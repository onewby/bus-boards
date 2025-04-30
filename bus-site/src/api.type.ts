import type {DateTime} from "luxon";

export type StopDeparture = {
    trip_id: string,
    trip_headsign: string,
    departure_time: string[],
    indicator: string[],
    route_short_name: string,
    colour: string,
    operator_id: string,
    operator_name: string,
    type: "bus" | "train",
    status?: string,
    _timestamp: DateTime,
    seq?: number,
    then_headsign?: string
}

export type StopAlert = {
    header?: string,
    description?: string,
    url?: string
}

export type StopInfo = {
    id: number,
    name: string,
    locality_name: string,
    locality_code: string
}

export type StanceInfo = {
    code: string,
    street: string,
    indicator: string,
    lat: number,
    long: number
}

export type BasicStopData = {
    stop: StopInfo,
    stances: StanceInfo[],
    filter: SearchResult | undefined
}

export type StopData = {
    stop: StopInfo,
    stances: StanceInfo[],
    times: StopDeparture[],
    alerts: StopAlert[],
    filter: SearchResult | undefined
}

export type ServiceStopData = {
    locality: string,
    name: string,
    display_name: string,
    loc?: string,
    ind?: string,
    arr: string,
    dep: string,
    puo: boolean,
    doo: boolean,
    major: boolean,
    long: number,
    lat: number,
    status?: string,
    seq: number
}

export type ServiceBranch = {
    dest: string,
    stops: ServiceStopData[],
    realtime?: {
        stop: number,
        pos?: {
            latitude: number,
            longitude: number,
            bearing: number,
            odometer: number,
            speed: number
        },
        pct?: number,
        on_previous: boolean,
        vehicle: {
            license?: string,
            name?: string,
            occupancy_pct?: number
        }
    },
    route: string,
    connections: Connections
}

export type ServiceInfo = {
    code: string,
    dest: string,
    cancelled: boolean
}

export type ServiceData = {
    service: ServiceInfo
    operator: {
        name: string,
        url: string
    },
    branches: ServiceBranch[],
    alerts: StopAlert[]
}

export type Connections = {
    from?: LinkedService,
    to?: LinkedService
}

export type LinkedService = {
    trip_id: string,
    location: string,
    dep_time: string
}

export type SearchResult = {
    name: string,
    parent: string,
    qualifier: string,
    locality: string,
    station?: string
}

export type SearchData = {
    results: SearchResult[]
}

export type NameLoc = {
    name: string,
    loc: string
}

export type StopVisits = {
    _links:    StopVisitsLinks;
    _embedded: Embedded;
}

export type Embedded = {
    "timetable:visit": TimetableVisit[];
}

export type TimetableVisit = {
    direction:              string;
    destinationName:        string;
    aimedArrivalTime:       string | null;
    aimedDepartureTime:     string;
    isRealTime:             boolean;
    cancelled:              boolean;
    sources:                Source[];
    expectedArrivalTime?:   string;
    expectedDepartureTime?: string;
    _links:                 TimetableVisitLinks;
    displayTime:            string;
}

export type TimetableVisitLinks = {
    "transmodel:line":   TransmodelLine;
    "timetable:journey": TimetableJourney;
}

export type TimetableJourney = {
    id:   string;
    href: string;
    date: string;
}

export type TransmodelLine = {
    name:        string;
    title:       string;
    description: string;
    colors:      Colors;
    operator:    string;
    href:        string;
}

export type Colors = {
    background: string;
    foreground: string;
}

export enum Source {
    Monitored = "monitored",
    Timetable = "timetable",
}

export type StopVisitsLinks = {
    "naptan:stop": NaptanStop;
    streetview:    Streetview[];
}

export type NaptanStop = {
    commonName:   string;
    localityName: null;
    atcoCode:     string;
    stopType:     string;
    location:     Location;
    indicator:    string;
    bearing:      string;
    _links:       NaptanStopLinks;
    href:         string;
}

export type NaptanStopLinks = {
    self: {
        href: string;
    };
}

export type Location = {
    type:        string;
    coordinates: number[];
}

export type Streetview = {
    href: string;
    type: string;
}

export type Vehicles = {
    type:     string;
    features: Feature[];
}

export type Feature = {
    type:       string;
    geometry:   Geometry;
    properties: Properties;
    _embedded:  Embedded;
    _links:     Links;
}

export type Links = {
    topups: Topups;
}

export type Topups = {
    href:  string;
    title: string;
}

export type Geometry = {
    type:        string;
    coordinates: number[];
}

export type Properties = {
    direction:         string;
    line:              string;
    operator:          string;
    vehicle:           string;
    href:              string;
    meta:              Meta;
    bearing?:          number;
    compassDirection?: string;
    destination:       string;
}

export type Meta = {
    number_plate:             string;
    fleet_number:             string;
    type:                     string;
    make:                     string;
    model:                    string;
    power_usb?:               boolean;
    wifi?:                    boolean;
    payments_contactless:     boolean;
    wheelchair_capacity:      number;
    low_floor?:               boolean;
    double_glazing:           boolean;
    zero_emission_engine?:    boolean;
    tenant:                   string;
    next_stop_announcements?: boolean;
    next_stop_display?:       boolean;
    low_emission_engine?:     boolean;
    name?:                    string;
    luggage_racks?:           boolean;
    reading_lights?:          boolean;
}

export type PolarTimetable = {
    _links:    PolarTimetableLinks;
    _embedded: PolarTimetableEmbedded;
    date:      string;
}

export type PolarTimetableEmbedded = {
    "transmodel:line":      EmbeddedTransmodelLine[];
    "transmodel:direction": TransmodelDirection;
    "timetable:waypoint":   TimetableWaypoint[];
    "timetable:journey":    PolarTimetableJourney[];
}

export type PolarTimetableJourney = {
    id:        string;
    _embedded: TimetableJourneyEmbedded;
    _links:    TimetableJourneyLinks;
}

export type TimetableJourneyEmbedded = {
    "timetable:visit": PolarTimetableVisit[];
}

export type PolarTimetableVisit = {
    aimedArrivalTime:   string;
    aimedDepartureTime: string | null;
    _links:             PolarTimetableVisitLinks;
}

export type PolarTimetableVisitLinks = {
    "timetable:waypoint": Self;
}

export type Self = {
    href: string;
}

export type TimetableJourneyLinks = {
    "transmodel:line": LinksTransmodelLine;
}

export type LinksTransmodelLine = {
    name:        string;
    description: string;
    colors:      Colors;
    href:        string;
}

export type TimetableWaypoint = {
    principle: boolean;
    major:     boolean;
    _links:    TimetableWaypointLinks;
}

export type TimetableWaypointLinks = {
    self:          Self;
    "naptan:stop": NaptanStop;
}

export type TransmodelDirection = {
    name:        string;
    origin:      string;
    destination: string;
    vias:        any[];
    _links?:     NaptanStopLinks;
    href?:       string;
}

export type EmbeddedTransmodelLine = {
    id:          string;
    name:        string;
    description: string;
    detail:      string | null;
    colors:      Colors;
    _embedded:   TransmodelLineEmbedded;
    _links:      NaptanStopLinks;
    weighting:   number | null;
}

export type TransmodelLineEmbedded = {
    "transmodel:operator": TransmodelOperator;
}

export type TransmodelOperator = {
    code: string;
    name: string;
}

export type PolarTimetableLinks = {
    "transmodel:line":      LinksTransmodelLine[];
    "transmodel:direction": TransmodelDirection[];
    self:                   Self;
    switch:                 Switch;
}

export type Switch = {
    href:      string;
    templated: boolean;
}

export type PolarLines = {
    _embedded: PolarLinesEmbedded;
}

export type PolarLinesEmbedded = {
    "transmodel:line": EmbeddedTransmodelLine[];
}

export type FirstVehicles = {
    jsonrpc: string;
    method:  string;
    params:  Params;
}

export type Params = {
    resource: Resource;
}

export type Resource = {
    member: Member[];
}

export type Member = {
    dir:             string;
    line:            string;
    line_name:       string;
    operator:        string;
    operator_name:   string;
    origin_atcocode: string;
    request_time:    string;
    status:          Status;
    stops:           Stop[];
}

export type Status = {
    bearing:                number;
    location:               FirstLocation;
    occupancy:              Occupancy;
    progress_between_stops: ProgressBetweenStops;
    recorded_at_time:       string;
    stops_index:            StopsIndex;
    vehicle_id:             string;
}

export type FirstLocation = {
    coordinates: number[];
    type:        string;
}

export type Occupancy = {
    types: Type[];
}

export type Type = {
    capacity: number;
    name:     string;
    occupied: number;
}

export type ProgressBetweenStops = {
    value: number;
}

export type StopsIndex = {
    type:  string;
    value: number;
}

export type Stop = {
    aimed:        Aimed;
    atcocode:     string;
    bearing:      string;
    date:         string;
    indicator:    string;
    latitude:     number;
    locality:     string;
    longitude:    number;
    name:         string;
    smscode:      string;
    stop_name:    string;
    time:         string;
    timing_point: boolean;
}

export type Aimed = {
    arrival:   Arrival;
    departure: Arrival;
}

export type Arrival = {
    date: null | string;
    time: null | string;
}

export type FirstWebSocketInfo = {
    data:  Data;
    links: Links;
}

export type Data = {
    url:            string;
    "access-token": string;
}

export type MegabusVehicles = {
    code:    number;
    message: string;
    routes:  MegabusRoute[];
}

export type MegabusRoute = {
    metadata:                 Metadata;
    chronological_departures: ChronologicalDeparture[];
}

export type ChronologicalDeparture = {
    trip:           MegabusTrip;
    active_vehicle: ActiveVehicle | null;
    stop:           MegabusStop;
    tracking:       Tracking;
    coachtracker:   Coachtracker;
}

export type ActiveVehicle = {
    current_wgs84_latitude_degrees:   number;
    current_wgs84_longitude_degrees:  number;
    current_forward_azimuth_degrees:  number;
    current_speed_mph:                number | null;
    last_update_time_unix:            number;
    engine_is_currently_on:           boolean;
    engine_is_currently_idling:       boolean;
    last_update_time_formatted_local: string;
}

export type Coachtracker = {
    is_earlier_departure: boolean;
    is_later_departure:   boolean;
}

export type MegabusStop = {
    sequence:                                 number;
    original_source_sequence:                 number;
    scheduled_arrival_time_unix:              number;
    scheduled_departure_time_unix:            number;
    live_arrival_time_unix:                   number | null;
    live_departure_time_unix:                 number | null;
    estimated_arrival_time_unix:              number | null;
    estimated_departure_time_unix:            number | null;
    scheduled_arrival_time_formatted_local:   string;
    scheduled_departure_time_formatted_local: string;
    live_arrival_time_formatted_local:        null | string;
    live_departure_time_formatted_local:      null | string;
    estimated_arrival_time_formatted_local:   null | string;
    estimated_departure_time_formatted_local: null | string;
}

export type Tracking = {
    current_delay_seconds:       number | null;
    total_distance_km:           number;
    is_future_trip:              boolean;
    is_cancelled:                boolean;
    is_completed:                boolean;
    has_no_tracking:             boolean;
    has_no_vehicle:              boolean;
    has_no_gps:                  boolean;
    is_stationary:               boolean;
    is_arrived:                  boolean;
    is_arrived_at_current_stop:  boolean;
    is_moving:                   boolean;
    is_moving_to_current_stop:   boolean;
    has_departed_current_stop:   boolean;
    has_moved_past_current_stop: boolean;
    has_bypassed_current_stop:   boolean;
}

export type MegabusTrip = {
    id:                             string;
    operator_code:                  string;
    operator_name:                  string;
    source_operator_code:           null;
    source_operator_name:           null;
    class_code:                     string;
    class_name:                     string;
    route_id:                       string;
    short_name:                     string;
    direction:                      Direction;
    pattern_code:                   string;
    duplicate_service:              boolean;
    duplicate_of_trip_id:           null;
    departure_time_unix:            number;
    arrival_time_unix:              number;
    departure_location_name:        string;
    arrival_location_name:          string;
    departure_locale:               string;
    arrival_locale:                 string;
    duration_seconds:               number;
    departure_time_formatted_local: string;
    arrival_time_formatted_local:   string;
}

export enum Direction {
    I = "I",
    O = "O",
}

export type Metadata = {
    route_id:                string;
    short_name:              string;
    departure_location_name: string;
    arrival_location_name:   string;
}

export type LothianRoutes = {
    server:      string;
    timeElapsed: number;
    networkTime: string;
    groups:      LothianGroup[];
}

export type LothianGroup = {
    id:          string;
    name:        string;
    description: null | string;
    routes:      LothianRoute[];
}

export type LothianRoute = {
    id:          string;
    name:        string;
    description: string;
    transitMode: null;
    color:       string;
    textColor:   string;
}

export type LothianPatterns = {
    server:      string;
    timeElapsed: number;
    networkTime: string;
    route:       LothianRoute;
    patterns:    LothianPattern[];
}

export type LothianPattern = {
    id:          string;
    routeName:   string;
    direction:   null;
    origin:      string;
    destination: string;
    polyline:    string;
    stops:       LothianStop[];
}

export type LothianStop = {
    id:               string;
    name:             string;
    transitMode:      null;
    coordinate:       Coordinate;
    stopCode:         string;
    compassDirection: null;
    bearing:          number;
    indicator:        null | string;
    routes:           any[];
}

export type Coordinate = {
    latitude:  number;
    longitude: number;
}

export type LothianLiveVehicles = {
    vehicles: LothianVehicle[];
}

export type LothianVehicle = {
    vehicle_id:   string;
    vehicle_type: string;
    journey_id:   string;
    latitude:     number;
    longitude:    number;
    destination:  string;
    heading:      number;
    service_name: string;
    next_stop_id: string;
}

export type LothianTimetables = {
    server:      string;
    timeElapsed: number;
    networkTime: string;
    timetable:   LothianTimetable;
}

export type LothianTimetable = {
    routePattern: LothianPattern;
    trips:        LothianTrip[];
}

export type LothianTrip = {
    tripID:     string;
    departures: LothianDeparture[];
}

export type LothianDeparture = {
    stopID:        string;
    time:          string;
    isTimingPoint: boolean;
    scheduledFor:  ScheduledFor;
    sequence:      string;
}

export type ScheduledFor = {
    unixTime:    number;
    isoTime:     string;
    displayTime: string;
}

export type LothianEvents = {
    events: LothianEvent[];
}

export type LothianEvent = {
    id:              string;
    created:         string;
    last_updated:    null | string;
    cause:           string;
    effect:          string;
    severity:        string;
    title:           Description;
    description:     Description;
    time_ranges:     TimeRange[];
    url:             string;
    webarticle_html: string;
    routes_affected: RoutesAffected[];
}

export type Description = {
    en: string;
}

export type RoutesAffected = {
    name:       string;
    diversions: any[];
}

export type TimeRange = {
    start:  string;
    finish?: string;
}

export type PolarDisruptions = {
    _embedded: PolarDisruptionsEmbedded;
}

export type PolarDisruptionsEmbedded = {
    alert: PolarAlert[];
}

export type PolarAlert = {
    id:            string;
    header:        string;
    description:   string;
    cause:         string;
    effect:        string;
    created:       string;
    type:          string;
    activePeriods: PolarActivePeriod[];
    _embedded:     PolarAlertEmbedded;
    _links?:       PolarAlertLinks;
}

export type PolarAlertEmbedded = {
    operator?: PolarOperator[];
    line?:     EmbeddedTransmodelLine[];
}

export type PolarActivePeriod = {
    start:              string;
    time_range_display: string;
    end?:               string;
}

export type PolarOperator = {
    id:     string;
    code:   string;
    name:   string;
    _links: OperatorLinks;
    tenant: string;
}

export type OperatorLinks = {
    self: PolarInfo;
}

export type PolarAlertLinks = {
    info: PolarInfo;
}

export type PolarInfo = {
    href: string;
}

export type SiriSx = {
    Siri: Siri;
}

export type Siri = {
    ServiceDelivery: ServiceDelivery;
}

export type ServiceDelivery = {
    ResponseTimestamp:         string;
    ProducerRef:               string;
    ResponseMessageIdentifier: string;
    SituationExchangeDelivery: SituationExchangeDelivery;
}

export type SituationExchangeDelivery = {
    ResponseTimestamp: string;
    Situations:        Situations;
}

export type Situations = {
    PtSituationElement: PtSituationElement[];
}

export type PtSituationElement = {
    CreationTime:         string;
    ParticipantRef:       string;
    SituationNumber:      string;
    Source:               SiriSource;
    Progress:             string;
    ValidityPeriod:       PublicationWindow[] | PublicationWindow;
    PublicationWindow:    PublicationWindow;
    MiscellaneousReason?: string;
    Planned:              boolean;
    Summary:              string;
    Description:          string;
    Consequences:         Consequences;
    InfoLinks?:           SiriInfoLinks;
    EquipmentReason?:     string;
}

export type Consequences = {
    Consequence: PurpleConsequence[] | PurpleConsequence;
}

export type ConsequenceElement = {
    Condition: string;
    Severity:  string;
    Affects:   PurpleAffects;
    Advice:    Advice;
    Blocking:  Blocking;
}

export type Advice = {
    Details: string;
}

export type PurpleAffects = {
    Networks:    PurpleNetworks;
    StopPoints?: PurpleStopPoints;
    Operators?:  SiriOperators;
}

export type PurpleNetworks = {
    AffectedNetwork: PurpleAffectedNetwork;
}

export type PurpleAffectedNetwork = {
    VehicleMode:   string;
    AffectedLine?: PurpleAffectedLine[] | PurpleAffectedLine;
    AllLines?:     string;
}

export type PurpleAffectedLine = {
    AffectedOperator: SiriAffectedOperator;
    LineRef:          number | string;
}

export type SiriAffectedOperator = {
    OperatorRef:  string;
    OperatorName: string;
}

export type SiriOperators = {
    AllOperators: string;
    AffectedOperators: SiriAffectedOperator | SiriAffectedOperator[] | undefined
}

export type PurpleStopPoints = {
    AffectedStopPoint: PurpleAffectedStopPoint[] | PurpleAffectedStopPoint;
}

export type PurpleAffectedStopPoint = {
    StopPointRef:  number;
    StopPointName: string;
    Location:      SiriLocation;
    AffectedModes: AffectedModes;
}

export type AffectedModes = {
    Mode: SiriMode;
}

export type SiriMode = {
    VehicleMode: string;
}

export type SiriLocation = {
    Longitude: number;
    Latitude:  number;
}

export type Blocking = {
    JourneyPlanner: boolean;
}

export type PurpleConsequence = {
    Condition: string;
    Severity:  string;
    Affects:   FluffyAffects;
    Advice?:   Advice;
    Blocking?: Blocking;
    Delays?:   Delays;
}

export type FluffyAffects = {
    Networks:    FluffyNetworks;
    StopPoints?: FluffyStopPoints;
    Operators?:  SiriOperators;
}

export type FluffyNetworks = {
    AffectedNetwork: FluffyAffectedNetwork;
}

export type FluffyAffectedNetwork = {
    VehicleMode:   string;
    AffectedLine?: FluffyAffectedLine[] | FluffyAffectedLine;
    AllLines?:     string;
}

export type FluffyAffectedLine = {
    AffectedOperator: SiriAffectedOperator;
    LineRef:          number | string;
    Direction?:       SiriDirection;
}

export type SiriDirection = {
    DirectionRef: string;
}

export type FluffyStopPoints = {
    AffectedStopPoint: FluffyAffectedStopPoint[] | FluffyAffectedStopPoint;
}

export type FluffyAffectedStopPoint = {
    StopPointRef:   number | string;
    StopPointName:  string;
    Location:       SiriLocation;
    AffectedModes?: AffectedModes;
}

export type Delays = {
    Delay: string;
}

export type SiriInfoLinks = {
    InfoLink: SiriInfoLink;
}

export type SiriInfoLink = {
    Uri: string;
}

export type PublicationWindow = {
    StartTime: string;
    EndTime?:  string;
}

export type SiriSource = {
    SourceType:          string;
    TimeOfCommunication: string;
}

export type StopsQuery = {
    name: string,
    display_name: string,
    locality: string,
    ind?: string,
    arr: number,
    dep: number,
    loc?: string,
    major: boolean,
    puo: boolean,
    doo: boolean,
    long: number,
    lat: number,
    seq: number,
    full_loc: string
}

// stops.name, stops.name as display_name, stops.locality, indicator as ind, arrival_time as arr,
//                 departure_time as dep, l.name as loc, timepoint as major, drop_off_type as doo, pickup_type as puo,
//                 stances.lat as lat, stances.long as long, stop_sequence as seq, stops.locality_name AS full_loc