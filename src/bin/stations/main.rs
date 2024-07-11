use std::error::Error;
use spex::parsing::XmlReader;
use BusBoardsServer::download_if_old;

const NAPTAN_NS: &str = "http://www.naptan.org.uk/";

/// Generate crs.csv to map ATCO codes to station CRS codes
fn main() -> Result<(), Box<dyn Error>> {
    let mut csv = csv::Writer::from_path("crs.csv")?;
    csv.write_record(&["ATCOCode", "CrsRef"])?;

    let naptan_file = download_if_old("https://naptan.api.dft.gov.uk/v1/access-nodes?dataFormat=xml", "NaPTAN.xml")?;
    let xml = XmlReader::parse_auto(naptan_file)?;

    xml.root().pre_ns(NAPTAN_NS).all("StopPoints").all("StopPoint")
        .iter().filter_map(|point| {
            let atco = point.pre_ns(NAPTAN_NS).req("AtcoCode").text().ok()?;
            let crs = point.pre_ns(NAPTAN_NS)
                .req("StopClassification")
                .req("OffStreet")
                .req("Rail")
                .req("AnnotatedRailRef")
                .req("CrsRef")
                .text().ok()?;
            Some([atco, crs])
        }).for_each(|r| csv.write_record(r).unwrap());

    Ok(())
}