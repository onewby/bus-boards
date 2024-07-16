use rusqlite::{Connection, params};
use std::error::Error;
use piz::ZipArchive;
use piz::read::{as_tree, FileTree};
use std::io::BufReader;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use std::io;
use rustls::{Certificate, ClientConfig, DigitallySignedStruct, ServerName};
use std::sync::Arc;
use suppaftp::{RustlsConnector, RustlsFtpStream};
use rustls::client::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use std::time::SystemTime;
use itertools::Itertools;

#[derive(Debug, Deserialize, Serialize)]
struct ServiceReportRecord {
    #[serde(rename = "NationalOperatorCode")]
    national_operator_code: String,
    #[serde(rename = "LineName")]
    line_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct AgencyInfo {
    agency_id: String,
    agency_name: String,
    routes: String,
    code: Option<String>,
    website: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PublicNameRecord {
    #[serde(rename = "PubNmId")]
    pub_nm_id: String,
    #[serde(rename = "Website")]
    website: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct NOCTableRecord {
    #[serde(rename = "NOCCODE")]
    noccode: String,
    #[serde(rename = "PubNmId")]
    pub_nm_id: String,
    #[serde(rename = "OperatorPublicName")]
    operator_public_name: String,
    #[serde(rename = "VOSA_PSVLicenseName")]
    vosa_psv_license_name: String,
}

fn get_service_report() -> Result<io::Cursor<Vec<u8>>, Box<dyn Error>> {
    let ftp_host = "ftp.tnds.basemap.co.uk";
    let ftp_user = std::env::var("TNDS_USERNAME")?;
    let ftp_password = std::env::var("TNDS_PASSWORD")?;

    // Create a connection to an FTP server and authenticate to it.
    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(DangerousCertVerifier {}))
        .with_no_client_auth();
    let config = Arc::new(config);
    let mut ftp_stream = RustlsFtpStream::connect((ftp_host, 21))
        .unwrap()
        .into_secure(RustlsConnector::from(Arc::clone(&config)), ftp_host)
        .unwrap();

    ftp_stream.login(&ftp_user, &ftp_password)?;
    println!("Connected to FTP");

    // Download the file from FTP
    let service_report_file = ftp_stream.retr_as_buffer("servicereport.csv")?;
    ftp_stream.quit()?;
    Ok(service_report_file)
}

struct DangerousCertVerifier {}

impl ServerCertVerifier for DangerousCertVerifier {
    fn verify_server_cert(&self, end_entity: &Certificate, intermediates: &[Certificate], server_name: &ServerName, scts: &mut dyn Iterator<Item=&[u8]>, ocsp_response: &[u8], now: SystemTime) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(&self, message: &[u8], cert: &Certificate, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(&self, message: &[u8], cert: &Certificate, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }
}

pub fn download_noc(conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    let service_report_file = get_service_report()?;
    println!("Downloaded TNDS service report");

    // Download Traveline zip file
    let traveline_url = "https://www.travelinedata.org.uk/wp-content/themes/desktop/nocadvanced_download.php?reportFormat=csvFlatFile&allTable%5B%5D=table_noc_table&allTable%5B%5D=table_public_name&submit=Submit";
    let traveline_zip = reqwest::blocking::get(traveline_url)?.bytes()?;
    let archive = ZipArchive::new(traveline_zip.as_ref())?;
    let dir = as_tree(archive.entries())?;
    let pnr_file = archive.read(dir.lookup("PublicName.csv")?)?;
    let noc_file = archive.read(dir.lookup("NOCTable.csv")?)?;

    let public_name_records = csv::Reader::from_reader(BufReader::new(pnr_file)).deserialize::<PublicNameRecord>().flatten().collect_vec();
    let mut noc_records = csv::Reader::from_reader(BufReader::new(noc_file));
    let noc_table: HashMap<_, _> = noc_records.deserialize::<NOCTableRecord>().flatten()
        .group_by(|r| r.noccode.clone()).into_iter().map(|(k, mut v)| (k, v.next().unwrap())).collect();

    // Read service report records
    let service_report_records: Vec<ServiceReportRecord> = csv::Reader::from_reader(BufReader::new(service_report_file)).deserialize().flatten().collect_vec();
    let nocs = service_report_records.iter().into_group_map_by(|r| r.national_operator_code.clone());
    let route_nocs: HashMap<_, _> = nocs.iter().map(|(noc, records)|
        (records.iter().map(|&r| r.line_name.clone()).sorted().collect::<Vec<_>>().join(","), noc))
        .into_group_map()
        .iter().filter_map(|(routes, nocs)| {
            if routes.len() == 0 {
                Some((routes.clone(), nocs[0].clone()))
            } else {
                None
            }
        }).collect();

    // Get agency info, with Traveline code where possible
    let mut assigned_codes = conn.prepare(
        "SELECT agency.agency_id, agency.agency_name, (SELECT group_concat(route_short_name) FROM routes WHERE routes.agency_id=agency.agency_id ORDER BY route_short_name) as routes FROM agency"
    )?.query_map([], |row| {
        Ok(AgencyInfo {
            agency_id: row.get("agency_id")?,
            agency_name: row.get("agency_name")?,
            routes: row.get("routes")?,
            code: route_nocs.get(row.get::<_, String>("routes")?.as_str()).map(|s| s.to_string()),
            website: None,
        })
    })?.flatten().collect_vec();

    let assigned_code_strs: HashSet<String> = assigned_codes.iter().filter_map(|a| a.code.clone()).collect();
    let remaining_agencies: Vec<_> = assigned_codes.iter_mut().filter(|a| a.code.is_none()).collect();
    let remaining_traveline: Vec<(String, String)> = nocs.iter()
        .filter(|(k, _)| !assigned_code_strs.contains(*k))
        .map(|(k, v)| {
            (
                k.clone(),
                v.iter().map(|r| r.line_name.clone()).sorted().collect::<Vec<_>>().join(",")
            )
        })
        .collect();

    for agency in remaining_agencies {
        // Try assigning by NOC table name
        let cands: Vec<_> = remaining_traveline.iter()
            .filter(|(t_k, _)| agency.agency_name == noc_table.get(t_k).map_or("", |r| &r.operator_public_name))
            .collect();

        if cands.is_empty() {
            // If no match, try assigning by operator public name
            let last_ditch_cands: Vec<_> = noc_table.values()
                .filter(|t| agency.agency_name == t.operator_public_name)
                .collect();
            if last_ditch_cands.len() == 1 {
                agency.code = Some(last_ditch_cands[0].noccode.clone());
            } else {
                println!("Could not map {} to Traveline data", agency.agency_name);
            }
            continue;
        }

        // Assign Traveline code to the agency with the closest routes match
        let closest = cands.iter().min_by(|(_, t_v1), (_, t_v2)| {
            let sim1 = sorensen::distance(agency.routes.as_bytes(), t_v1.as_bytes());
            let sim2 = sorensen::distance(agency.routes.as_bytes(), t_v2.as_bytes());
            sim2.partial_cmp(&sim1).unwrap()
        });
        if let Some((t_k, _)) = closest {
            agency.code = Some(t_k.clone());
        }
    }

    assigned_codes
        .iter_mut()
        .filter(|a| a.code.is_some())
        .for_each(|assignment| {
            if let Some(noc) = noc_table.get(&assignment.code.clone().unwrap()) {
                if let Some(pnr) = public_name_records.iter().find(|t| t.pub_nm_id == noc.pub_nm_id) {
                    let first_index = pnr.website.find('#');
                    let last_index = pnr.website.rfind('#');
                    assignment.website = if let (Some(first), Some(last)) = (first_index, last_index) {
                        if first != last {
                            Some(pnr.website[first + 1..last].to_string())
                        } else {
                            Some(pnr.website.clone())
                        }
                    } else {
                        Some(pnr.website.clone())
                    }
                }
            }
        });

    // Distinguish duplicates
    assigned_codes.iter_mut().filter(|a| a.code.is_some()).into_group_map_by(|a| a.code.clone().unwrap())
        .into_iter().for_each(|(code, mut rows)| {
            rows.iter_mut().enumerate().skip(1).for_each(|(i, row)| {
                row.code = Some(format!("{code}{i}"));
            })
    });

    // Insert into SQLite database
    conn.execute("DELETE FROM traveline", params![])?;

    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare("INSERT INTO traveline (code, agency_id, website) VALUES (?1, ?2, ?3)")?;
        for record in assigned_codes.iter().filter(|c| c.code.is_some()) {
            stmt.execute(params![
                &record.code,
                &record.agency_id,
                &record.website
            ])?;
        }
    }
    tx.commit()?;

    Ok(())
}
