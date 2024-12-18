create index if not exists stop_times_trip_id_stop_sequence_index
    on stop_times (trip_id asc, stop_sequence desc);

create index if not exists stop_times_stop_id_index
    on stop_times (stop_id);

create unique index if not exists stops_name_locality_uindex
    on stops (name, locality);

create unique index if not exists calendar_dates_service_id_exception_type_date_uindex
    on calendar_dates (service_id, exception_type, date);

create index if not exists stances_stop_index
    on stances (stop);

create index if not exists polar_polar_index
    on polar (polar);

create unique index if not exists traveline_agency_id_uindex
    on traveline (agency_id);

create index if not exists trips_route_id_index
    on trips (route_id);