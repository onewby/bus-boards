pub struct Source<'a> {
    pub name: &'a str,
    pub prefix: &'a str,
    pub url: &'a str,
    pub path: &'a str
}

pub const SOURCES: [Source; 2] = [
    Source {
        name: "BODS (approx ~550MB)",
        prefix: "",
        url: "https://data.bus-data.dft.gov.uk/timetable/download/gtfs-file/all/",
        path: "gtfs/itm_all_gtfs.zip",
    },
    Source {
        name: "Ember",
        prefix: "E",
        url: "https://api.ember.to/v1/gtfs/static/",
        path: "gtfs/ember.zip",
    }
];