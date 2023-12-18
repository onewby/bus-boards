import type {DateTime} from "luxon";

export type StopDeparture = {
    trip_id: string,
    trip_headsign: string,
    departure_time: string,
    indicator: string,
    route_short_name: string,
    colour: string,
    operator_id: string,
    operator_name: string,
    type: "bus" | "train",
    status?: string,
    _timestamp?: DateTime,
    seq?: number
}

export type StopData = {
    stop: {
        id: number,
        name: string,
        locality_name: string,
        locality_code: string
    },
    stances: {
        code: string,
        street: string,
        indicator: string,
        lat: number,
        long: number
    }[],
    times: StopDeparture[],
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
        pct?: number
    },
    route: [number, number][]
}

export type ServiceInfo = {
    code: string,
    dest: string,
    cancelled: boolean,
    message: string
}

export type ServiceData = {
    service: ServiceInfo
    operator: {
        name: string,
        url: string
    },
    branches: ServiceBranch[]
}

export type SearchResult = {
    name: string,
    parent: string,
    qualifier: string,
    locality: string
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
