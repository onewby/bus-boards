pub const LOCALITY_CHANGES: [(&str, &str, &str, Option<&str>); 10] = [
    ("N0077860", "Park Lane (Tyne and Wear Metro Station)", "E0057917", None),
    ("E0057948", "Bradford Interchange Rail Station", "N0077005", Some("Bradford")),
    ("E0057974", "Leeds Rail Station", "N0077039", Some("Leeds")),
    ("ES003919", "Dundee Rail Station", "ES000536", Some("Dundee")),
    ("E0034956", "London Victoria Coach Station", "E0034917", Some("Victoria")),
    ("E0057190", "Luton Rail Station", "N0071638", Some("Luton")),
    ("N0078022", "Bus Station", "ES002978", Some("Glasgow")),
    ("E0039083", "Rail Station Entrance", "N0071638", Some("Luton")),
    ("E0033284", "Wellington Bridge St Real Time Tracking", "N0077039", Some("Leeds")),
    ("E0057149", "Rail Station", "N0077769", Some("Bournemouth"))
];

pub const MANUAL_RENAMES: [(&str, &str, &str); 66] = [
    ("N0078622", "^Edinburgh Airport \\(Edinburgh Trams\\)$", "Airport"),
    ("ES001737", "^Haymarket \\(Edinburgh Trams\\)$", "Rail Station"),
    ("ES001737", "^Haymarket Station$", "Rail Station"),
    ("E0049583", "^Stand 4$", "Bus Station"),  // Tadcaster
    ("ES000536", "^Bus Station$", "Seagate Bus Station"),  // Dundee
    ("N0078275", "^Edinburgh Park Station$", "Rail Station"),
    ("N0078275", "^Edinburgh Park Station \\(Edinburgh Trams\\)$", "Rail Station"),
    ("E0057900", "^Newcastle Rail Station$", "Newcastle Central Rail Station"),
    ("E0057900", "^Central Station \\(Tyne and Wear Metro Station\\)$", "Newcastle Central Rail Station"),
    ("E0057900", "^Central Station Bewick Street$", "Newcastle Central Rail Station"),
    ("E0057900", "^Central Stn Clayton St$", "Newcastle Central Rail Station"),
    ("E0057900", "^Central Stn Neville St$", "Newcastle Central Rail Station"),
    ("E0057900", "^Central Stn Westgate Rd$", "Newcastle Central Rail Station"),
    ("E0057900", "^Central Station Westgate Road$", "Newcastle Central Rail Station"),
    ("E0057900", "^Central Rail Station$", "Newcastle Central Rail Station"),
    ("E0057900", "^Central Stn$", "Newcastle Central Rail Station"),
    ("N0078208", "^ Ponteland Road - Newcastle Airport$", "Newcastle Airport Ponteland Road"),
    ("N0078208", "^Newcastle Airport Metro Station$", "Newcastle Airport (Tyne and Wear Metro Station)"),
    ("E0055009", "^Hull Rail Station$", "Paragon Interchange (Rail Station)"),
    ("E0055009", "^Hull Interchange$", "Paragon Interchange (Rail Station)"),
    ("N0077005", "^Bradford Interchange Rail Station$", "Interchange"),
    ("E0057917", "^Sunderland Interchange$", "Park Lane Interchange"),
    ("E0057917", "^Park Lane \\(Tyne and Wear Metro Station\\)$", "Park Lane Interchange"),
    ("E0057917", "^Sunderland \\(Tyne and Wear Metro Station\\)$", "Rail Station"),
    ("ES000536", "^Station$", "Rail Station"),
    ("E0050224", "^East Midlands Parkway Station$", "East Midlands Parkway Rail Station"),
    ("E0030375", "^Meadowhall Rail Station$", "Meadowhall Interchange"),
    ("E0030375", "^Meadowhall Interchange \\(S Yorks Supertram\\)$", "Meadowhall Interchange"),
    ("N0077854", "^Metrocentre Rail Station$", "Metrocentre Interchange"),
    ("N0077039", "^Leeds BS Ent Real Time Tracking$", "Bus Station"),
    ("N0077039", "^Station A$", "Rail Station A"),
    ("N0077039", "^Station B$", "Rail Station B"),
    ("N0077039", "^Station C$", "Rail Station C"),
    ("N0077039", "^Station D$", "Rail Station D"),
    ("N0077039", "^Station E$", "Rail Station E"),
    ("N0077039", "^Station F$", "Rail Station F"),
    ("N0077039", "^Leeds Station Interchange$", "Rail Station"),
    ("E0039258", "^Bus Station Express Lounge$", "Bus Station"),
    ("E0033527", "^Bus Station stand D$", "Seacroft Bus Station"),
    ("N0075057", "^Rail Station$", "Manchester Airport Rail Station"),
    ("N0075057", "^Manchester Airport The Station$", "Manchester Airport Rail Station"),
    ("N0075057", "Manchester Airport The Station", "Rail Station"),
    ("N0075057", "^Manchester Airport \\(Manchester Metrolink\\)$", "Manchester Airport Rail Station"),
    ("E0034917", "^London Victoria Coach Station$", "Victoria Coach Station"),
    ("N0073334", "Park and Ride Stance C", "Park and Ride"),
    ("N0073334", "Broxden Park\\+Ride", "Park and Ride"),
    ("N0071638", "^Luton Rail Station$", "Rail Station Interchange"),
    ("N0071638", "^Luton Station Interchange$", "Rail Station Interchange"),
    ("N0071638", "^Rail Station Entrance$", "Rail Station Interchange"),
    ("E0056332", "^Bus Station Arrive$", "Bus Station"),
    ("E0015874", "^Arrival Stand$", "Bus Station"),
    ("ES002978", "^Bus Station$", "Partick Station Interchange"),
    ("ES002978", "^Partick Rail Station$", "Partick Station Interchange"),
    ("ES002978", "^Partick SPT Subway Station$", "Partick Station Interchange"),
    ("ES002978", "^Partick Interchange$", "Partick Station Interchange"),
    ("ES003486", "Stance", "Rail Station"), // could be Goosecroft Road - bit of an editorial decision
    ("ES001670", "Stance", "Bus Station"),
    ("ES000097", "Stance", "Bus Station"),
    ("ES001470", "^Stance 8$", "Transport Interchange"),
    ("ES000923", "^Town Centre Stances$", "Town Centre"),
    ("ES000923", "^Town Centre stances$", "Town Centre"),
    ("N0077005", "^Nelson Street Real Time Tracking$", "Nelson Street"),
    ("N0078288", "^St Partick Square$", "St Patrick Square"),
    ("E0049949", "^Rail station$", "Rail Station"),
    ("ES003921", "^Games Shuttle$", "Buchanan Bus Station"),
    ("N0080902", "^Coach Stop$", "Bond Street")
];

pub const MANUAL_ARRIVALS: [&str; 4] = [
    "6400L00040",
    "6090117",
    "64803493",
    "6490IM002"
];

pub const NAPTAN_OVERRIDES: [(&str, &str); 2] = [
    ("E0055323", "E0043654"),
    ("N0072212", "E0041739")
];