import soap from "soap"

export class DarwinAPI {

    _token: string
    client: soap.Client | undefined
    clientPromise: Promise<soap.Client>

    constructor(token: string) {
        if(token == "") console.error("Invalid Darwin API token - ensure the DARWIN_API_KEY environment variable is set.")
        this._token = token
        this.client = undefined
        this.clientPromise = this.getClient()
    }

    async getClient(): Promise<soap.Client> {
        if(this.client === undefined) {
            if(this.clientPromise !== undefined) {
                return this.clientPromise
            }
            this.client = await soap.createClientAsync(
                "https://lite.realtime.nationalrail.co.uk/OpenLDBWS/wsdl.aspx?ver=2021-11-01"
            )
            this.client.addSoapHeader({"AccessToken": {"TokenValue": this._token}}, undefined, "typ", "http://thalesgroup.com/RTTI/2013-11-28/Token/types")
        }
        return this.client
    }

    static _fixCallingPoints(callingPoints: any) {
        if(!(callingPoints.callingPointList instanceof Array)) {
            callingPoints.callingPointList = [callingPoints.callingPointList]
        }
        if(callingPoints.callingPointList.length > 0 && !(callingPoints.callingPointList[0].callingPoint instanceof Array)) {
            callingPoints.callingPointList[0].callingPoint = [callingPoints.callingPointList[0].callingPoint]
        }
    }

    async getDepartureBoard(options: GetBoardOptions): Promise<ServiceBoard> {
        return (await (await this.getClient()).GetDepartureBoardAsync(options))[0].GetStationBoardResult
    }

    async getServiceDetails(service: string): Promise<ServiceDetails> {
        let client = await this.getClient()
        let details: ServiceDetails = (await client.GetServiceDetailsAsync({serviceID: service}))[0].GetServiceDetailsResult
        if(!details.previousCallingPoints) details.previousCallingPoints = {callingPointList: []}
        if(!details.subsequentCallingPoints) details.subsequentCallingPoints = {callingPointList: []}
        DarwinAPI._fixCallingPoints(details.previousCallingPoints)
        DarwinAPI._fixCallingPoints(details.subsequentCallingPoints)
        details.isCancelled = !!details.isCancelled
        return details
    }
}

type GetBoardOptions = {
    numRows?: number,
    crs: string,
    filterCrs?: string,
    filterType?: "to" | "from",
    timeOffset?: number,
    timeWindow?: number
}

export type Endpoint = {
    locationName: string,
    crs: string
}

type Endpoints = {
    location: Endpoint[]
}

export type Service = {
    std?: string,
    etd?: string,
    sta?: string,
    eta?: string,
    platform: string,
    operator: string,
    operatorCode: string,
    serviceType: string,
    serviceID: string,
    rsid: string,
    origin: Endpoints,
    destination: Endpoints
}

export type ServiceBoard = {
    generatedAt: string,
    locationName: string,
    crs: string,
    platformAvailable: boolean,
    trainServices?: {
        service: Service[]
    }
}

export type ServiceDetails = {
    generatedAt: string,
    serviceType: string,
    locationName: string,
    crs: string,
    operator: string,
    operatorCode: string,
    rsid: string,
    delayReason?: string,
    cancelReason?: string
    isCancelled: boolean,
    platform: string,
    length: string,
    sta?: string,
    eta?: string,
    ata?: string,
    std?: string,
    etd?: string,
    atd?: string,
    previousCallingPoints: CallingPoints,
    subsequentCallingPoints: CallingPoints
}

export type CallingPoint = Endpoint & {
    st: string,
    et?: string,
    at?: string
}

type CallingPoints = {
    callingPointList: {
        callingPoint: CallingPoint[]
    }[]
}