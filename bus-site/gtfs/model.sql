create table if not exists shapes
(
    shape_id          TEXT,
    shape_pt_lat      REAL,
    shape_pt_lon      REAL,
    shape_pt_sequence integer
);

create index if not exists shapes_shape_id_index
    on shapes (shape_id);

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
    monday     integer,
    tuesday    integer,
    wednesday  integer,
    thursday   integer,
    friday     integer,
    saturday   integer,
    sunday     integer,
    start_date  integer,
    end_date    integer,
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
create unique index if not exists calendar_dates_service_id_exception_type_date_uindex
    on calendar_dates (service_id, exception_type, date);

create table if not exists trips
(
    trip_id       TEXT,
    route_id      TEXT,
    service_id    TEXT,
    trip_headsign TEXT,
    max_stop_seq  integer,
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
    constraint stop_times_pk
        primary key (trip_id, stop_sequence),
    constraint stop_times_trips_source_trip_id_fk
        foreign key (trip_id) references trips
);

create index if not exists stop_times_trip_id_stop_sequence_index
    on stop_times (trip_id asc, stop_sequence desc);

create index if not exists stop_times_stop_id_index
    on stop_times (stop_id);

create table if not exists file_hashes
(
    source TEXT PRIMARY KEY,
    hash TEXT
);