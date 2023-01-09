export type StopDeparture = {
    trip_id: string,
    trip_headsign: string,
    departure_time: string,
    indicator: string,
    route_short_name: string,
    colour: string,
    operator_id: string,
    operator_name: string
}

export type StopData = {
    stop: {
        id: number,
        name: string,
        locality_name: string
    },
    stances: {
        code: string,
        indicator: string
    }[],
    times: StopDeparture[]
}

export type ServiceStopData = {
    id: string,
    name: string,
    loc: string,
    ind: string,
    arr: string,
    dep: string,
    puo: boolean,
    doo: boolean,
    major: boolean,
    long: number,
    lat: number
}

export type ServiceData = {
    service: {
        code: string,
        dest: string
    }
    operator: {
        name: string,
        url: string
    },
    stops: ServiceStopData[],
    realtime?: {
        stop: number,
        pos: {
            latitude: number,
            longitude: number,
            bearing: number,
            odometer: number,
            speed: number
        },
        pct: number
    },
    route: [number, number][]
}

export type SearchResult = {
    id: string,
    name: string,
    parent: string,
    qualifier: string
}

export type SearchData = {
    results: SearchResult[]
}