<script lang="ts">
    import Header from "../../header.svelte";
    import Stop from "./stop.svelte";
    import type {ServiceData} from "../../../api.type"

    import Fa from "svelte-fa";
    import {faBus, faChevronRight} from "@fortawesome/free-solid-svg-icons";

    import Map from "@svelte-parts/map/Map.svelte";
    import Tiles from "@svelte-parts/map/tiles/Tiles.svelte";
    import 'leaflet/dist/leaflet.css';
    import {page} from "$app/stores";
    import {onDestroy, onMount} from "svelte";
    import type {GeoJSONOptions} from "leaflet";
    import GeoJSON from "./GeoJSON.svelte";
    import L from "leaflet";
    import HTMLMarker from "./HTMLMarker.svelte";
    import {DateTime} from "luxon";

    export let data: ServiceData
    let expand = false
    let zoom = 15

    $: lon = data.realtime?.pos.longitude ?? data.stops[Math.floor(data.stops.length / 2)].long
    $: lat = data.realtime?.pos.latitude ?? data.stops[Math.floor(data.stops.length / 2)].lat
    $: rotation = data.realtime ? (data.realtime?.pos.bearing + 270) % 360 : 0
    $: flip = rotation >= 90 && rotation <= 270 ? 'scaleY(-1)' : ''

    $: {
        if(data.stops.length < 20) data.stops.forEach(stop => stop.major = true)
    }

    let realtimeData
    $: expand, data.realtime, realtimeData = getRealtimePct()

    let tickerNumber
    onMount(() => {
        if(data.realtime) {
            tickerNumber = setInterval(async () => {
                const resp = await fetch(`/api/service?id=` + $page.params['service'])
                if(!resp.ok) return
                data = await resp.json()
            }, 30000)
        }
    })

    onDestroy(() => {
        if(tickerNumber) clearInterval(tickerNumber)
    })

    $: geoData = {
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "properties": {},
                "geometry": {
                    "coordinates": data.route,
                    "type": "LineString"
                }
            }
        ]
    }

    const geoOptions: GeoJSONOptions = {
        style: function(feature) {
            return {
                stroke: true,
                color: "rgb(100, 83, 9)",
                weight: 4
            }
        },
        pointToLayer: function (feature, latlng) {
            let divIcon = L.divIcon({
                className: "bg-amber-500 h-full w-full rounded border-black border"
            })
            let marker = L.marker(latlng, {icon: divIcon})
            marker.bindPopup(`<b>${feature.properties.name} (${feature.properties.ind})</b><br>${feature.properties.loc}`)
            return marker
        }
    }

    function getRealtimePct() {
        let realtimePct = data.realtime?.pct
        if(data.realtime && !expand) {
            const stopIndex = data.realtime.stop
            const previousMajorStop = data.stops.slice(0, stopIndex).findLastIndex(stop => stop.major) ?? 0
            const nextMajorStop = data.stops.slice(stopIndex).findIndex(stop => stop.major) + stopIndex ?? data.stops.length - 1

            if(stopIndex > 0 && data.stops.slice(previousMajorStop, nextMajorStop).some(stop => !stop.major)) {
                const stopTime = toLuxon(data.stops[stopIndex].dep)
                const prevStopTime = toLuxon(data.stops[stopIndex - 1].dep)
                const prevMajorTime = toLuxon(data.stops[previousMajorStop].dep)
                const nextMajorTime = toLuxon(data.stops[nextMajorStop].dep)

                // get current time elapsed = (time elapsed previousMajorStop to last stop) + (pct elapsed * time elapsed from last stop to the upcoming stop)
                let elapsedTime = prevStopTime.diff(prevMajorTime, "milliseconds").milliseconds + (data.realtime.pct * stopTime.diff(prevStopTime, "milliseconds").milliseconds)

                // get time nextMajorStop - previousMajorStop
                let totalTime = nextMajorTime.diff(prevMajorTime, "milliseconds").milliseconds

                realtimePct = elapsedTime / totalTime
                return {pct: realtimePct, stop: nextMajorStop}
            }
        }
        return {pct: realtimePct, stop: data.realtime?.stop}
    }

    const timeFmt = (time: string) => (Number(time.substring(0, 2)) % 24).toString().padStart(2, "0") + time.substring(2, 5)

    function toLuxon(time: string) {
        let addDays = Math.floor(Number(time.substring(0, 2)) / 24)
        let timestamp = timeFmt(time)
        return DateTime.fromSQL(timestamp).plus({day: addDays})
    }
</script>

<div class="w-full h-fit overflow-scroll flex flex-col justify-start items-center max-w-full pt-4 pb-8 dark:text-white">
    <Header showBack={true}>
        <div class="text-2xl">
            <span class="font-bold">{data.service.code}</span> to <span class="font-semibold">{data.service.dest}</span>
        </div>
    </Header>

    <a href={data.operator.url} class="w-full max-w-2xl">
        <div class="panel w-full mt-2 pl-8 pr-8 pt-4 pb-4 flex flex-row items-center">
            <Fa icon={faBus} size="lg" class="mr-4" />
            <div class="flex-grow">Operated by {data.operator.name}</div>
            <Fa icon={faChevronRight} size="md" />
        </div>
    </a>

    {#if data.stops.some((stop) => !stop.major)}
        <div class="panel w-full mt-2 pl-8 pr-8 pt-4 pb-4 flex flex-row items-center justify-center">
            <label for="expand">Show minor stops</label>
            <input id="expand" type="checkbox" class="ml-2" bind:checked={expand}>
        </div>
    {/if}

    <div class="panel w-full mt-2 pt-4 pb-4 flex flex-col items-center">
        {#each data.stops as stop, i}
            {#if expand || stop.major || i === 0 || i === data.stops.length - 1}
                <Stop type={i === 0 ? "origin" : i === data.stops.length - 1 ? "destination" : "stop"} stop={stop}
                      realtime={i === realtimeData.stop ? realtimeData.pct && data.realtime : undefined} />
            {/if}
        {/each}
    </div>

    <div class="panel w-full mt-2 flex flex-col items-center">
        <Map width="100%" height="300px" lon={lon} lat={lat} bind:zoom>
            <Tiles />
            <GeoJSON data={geoData} options={geoOptions} />
            {#each data.stops as stop}
                <HTMLMarker lon={stop.long} lat={stop.lat} popup="{timeFmt(stop.dep)}<br><b>{stop.name}{stop.ind ? ` (${stop.ind})` : ''}</b><br>{stop.loc}"
                            divIcon={{ className: "bg-amber-400 rounded border border-black" }} zIndex={-1000}/>
            {/each}
            {#if data.realtime}
                <HTMLMarker lon={lon} lat={lat} divIcon={{
                        html: `<div class='bg-white border border-black h-full w-full rounded-tr-full' style='transform: rotate(${rotation}deg) ${flip}'></div>`,
                        className: "", iconSize: [20, 12] }} />
            {/if}
        </Map>
    </div>
</div>

<svelte:head>
    <title>{data ? timeFmt(data.stops[0].dep) + " " + data.service.code + " to " + data.service.dest : "Service"} - Bus Boards</title>
</svelte:head>