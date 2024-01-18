create table if not exists localities
(
    code      TEXT
        constraint localities_pk
            primary key,
    name      TEXT,
    qualifier TEXT,
    parent    TEXT
        constraint localities_localities_code_fk
            references localities,
    lat       REAL,
    long      REAL
);

create table if not exists stops
(
    id            integer
        constraint stops_pk
            primary key autoincrement,
    name          text,
    locality      text
        constraint stops_localities_code_fk
            references localities,
    locality_name TEXT
);

create table if not exists stances
(
    code      TEXT
        constraint stances_pk
            primary key,
    street    TEXT,
    indicator TEXT,
    lat       REAL,
    long      REAL,
    stop      INTEGER
        constraint stances_stops_id_fk
            references stops,
    crs       TEXT
);

create table if not exists shapes
(
    shape_id          TEXT,
    shape_pt_lat      REAL,
    shape_pt_lon      REAL,
    shape_pt_sequence integer
);

create table if not exists agency
(
    agency_id       TEXT not null,
    agency_name     TEXT not null,
    agency_url      TEXT,
    agency_timezone TEXT,
    agency_lang     TEXT,
    constraint agency_pk
        primary key (agency_id)
);

create table if not exists routes
(
    route_id         TEXT,
    agency_id        TEXT,
    route_short_name TEXT,
    route_long_name  TEXT,
    route_type       TEXT,
    constraint routes_pk
        primary key (route_id),
    constraint routes_agency_agency_id_source_fk
        foreign key (agency_id) references agency (agency_id)
);

create table if not exists calendar
(
    service_id TEXT,
    start_date integer,
    end_date   integer,
    validity   integer,
    constraint calendar_pk
        primary key (service_id)
);

create table if not exists calendar_dates
(
    service_id     TEXT,
    date           integer,
    exception_type integer,
    constraint calendar_dates_pk
        primary key (service_id, date)
);

create table if not exists trips
(
    trip_id       TEXT,
    route_id      TEXT,
    service_id    TEXT,
    trip_headsign TEXT,
    max_stop_seq  integer,
    min_stop_seq  integer,
    shape_id      TEXT,
    constraint trips_pk
        primary key (trip_id),
    constraint trips_routes_route_id_source_fk
        foreign key (route_id) references routes
);

create table if not exists stop_times
(
    trip_id        TEXT,
    arrival_time   DATETIME,
    departure_time DATETIME,
    stop_id        TEXT,
    stop_headsign  TEXT,
    stop_sequence  integer,
    timepoint      integer,
    drop_off_type  integer,
    pickup_type    integer,
    /*constraint stop_times_pk
        primary key (trip_id, stop_sequence),*/
    constraint stop_times_trips_source_trip_id_fk
        foreign key (trip_id) references trips
);

create table if not exists file_hashes
(
    source TEXT PRIMARY KEY,
    hash TEXT
);

CREATE TABLE IF NOT EXISTS polar
(
    gtfs      TEXT
        constraint polar_pk
            primary key
        constraint polar_trips_trip_id_fk
            references trips,
    polar     TEXT,
    direction INT
);

CREATE TABLE IF NOT EXISTS lothian
(
    pattern TEXT,
    route   TEXT
        constraint lothian_routes_route_id_fk
            references routes
);

CREATE TABLE IF NOT EXISTS traveline
(
    code      TEXT
        constraint traveline_pk
            primary key,
    agency_id TEXT
        constraint traveline_agency_agency_id_fk
            references agency,
    website   TEXT
);

