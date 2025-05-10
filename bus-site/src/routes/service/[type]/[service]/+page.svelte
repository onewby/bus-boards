<script lang="ts">
    import Header from "../../../header.svelte";
    import Stop from "./stop.svelte";
    import type {PageData} from "./$types"
    import Connection from "./connection.svelte";

    import Fa from "svelte-fa";
    import {faBus, faChevronRight} from "@fortawesome/free-solid-svg-icons";

    import Map from "../../../../map/Map.svelte";
    import Tiles from "../../../../map/Tiles.svelte";
    import 'leaflet/dist/leaflet.css';
    import {page} from "$app/state";
    import {onDestroy, onMount} from "svelte";
    import type {GeoJSONOptions} from "leaflet";
    import GeoJSON from "../../../../map/GeoJSON.svelte";
    import L from "leaflet";
    import HTMLMarker from "./HTMLMarker.svelte";
    import {DateTime} from "luxon";
    import {browser} from "$app/environment";
    import Alert from "../../../Alert.svelte";
    import polyline from "google-polyline";
    import {invalidateAll} from "$app/navigation";
    import type {GeoJsonObject} from "geojson";

    let { data } = $props();

    // map controls
    let expand = $state(false)
    let zoom = $state(15)
    // branch index
    let b = $state(0)

    // branch (for splitting trains)
    let branch = $derived(data.branches[b])
    // map positioning
    let lon = $derived(branch.realtime?.pos?.longitude ?? branch.stops[Math.floor(branch.stops.length / 2)].long)
    let lat = $derived(branch.realtime?.pos?.latitude ?? branch.stops[Math.floor(branch.stops.length / 2)].lat)
    // vehicle positioning
    let rotation = $derived(branch.realtime?.pos ? (branch.realtime.pos.bearing + 270) % 360 : 0)
    let flip = $derived(rotation >= 90 && rotation <= 270 ? 'scaleY(-1)' : '')
    // position the realtime dot
    let realtimeData = $derived.by(() => {
        branch.realtime; expand;
        let rtp = getRealtimePct()
        console.log(rtp)
        return rtp
    })

    let train = page.params.type === "train"

    // refresh every 30s if realtime data exists
    let tickerNumber: ReturnType<typeof setInterval>
    onMount(() => {
        if(branch.realtime) {
            tickerNumber = setInterval(async () => {
                await invalidateAll()
            }, 30000)
        }
    })

    onDestroy(() => {
        if(tickerNumber) clearInterval(tickerNumber)
    })

    // Map (Leaflet) config
    const popupOptions = {
        maxWidth: 108,
        className: "mapPopup"
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
            marker.bindPopup(`<b>${feature.properties.display_name} (${feature.properties.ind})</b><br>${feature.properties.loc}`, popupOptions)
            return marker
        }
    }

    let geoData = $derived({
        "type": "FeatureCollection",
        "features": data.branches.map(branch => ({
            "type": "Feature",
            "properties": {},
            "geometry": {
                "coordinates": polyline.decode(branch.route).map(([lat, lng]) => [lng, lat]),
                "type": "LineString"
            }
        }))
    } as GeoJsonObject)

    // Position the realtime dot
    function getRealtimePct() {
        let realtimePct = branch.realtime?.pct
        if(branch.realtime && !expand) {
            // calculate position when minor stops are hidden
            const stopIndex = branch.realtime.stop
            const previousMajorStop = Math.max(branch.stops.slice(0, stopIndex).findLastIndex(stop => stop.major), 0)
            let nextMajorStop = branch.stops.slice(stopIndex).findIndex(stop => stop.major) + stopIndex
            if(nextMajorStop === stopIndex - 1) nextMajorStop = branch.stops.length - 1

            // interpolate between the two major stops between the next stop
            if(stopIndex > 0 && branch.stops.slice(previousMajorStop, nextMajorStop).some(stop => !stop.major)) {
                const stopTime = toLuxon(branch.stops[stopIndex].dep)
                const prevStopTime = toLuxon(branch.stops[stopIndex - 1].dep)
                const prevMajorTime = toLuxon(branch.stops[previousMajorStop].dep)
                const nextMajorTime = toLuxon(branch.stops[nextMajorStop].dep)

                // get current time elapsed = (time elapsed previousMajorStop to last stop) + (pct elapsed * time elapsed from last stop to the upcoming stop)
                let elapsedTime = prevStopTime.diff(prevMajorTime, "milliseconds").milliseconds + (branch.realtime.pct! * stopTime.diff(prevStopTime, "milliseconds").milliseconds)

                // get time nextMajorStop - previousMajorStop
                let totalTime = nextMajorTime.diff(prevMajorTime, "milliseconds").milliseconds

                realtimePct = elapsedTime / totalTime
                return {pct: realtimePct, stop: nextMajorStop}
            }
        }

        return {pct: realtimePct, stop: branch.realtime?.stop}
    }

    // Some times may be >24h, format properly
    const timeFmt = (time: string) => (Number(time.substring(0, 2)) % 24).toString().padStart(2, "0") + time.substring(2, 5)

    // Convert string time to luxon DateTime
    function toLuxon(time: string) {
        let addDays = Math.floor(Number(time.substring(0, 2)) / 24)
        let timestamp = timeFmt(time)
        return DateTime.fromSQL(timestamp).plus({day: addDays})
    }
</script>

<div class="w-full h-fit flex flex-col justify-start items-center max-w-full pt-4 pb-8 dark:text-white">
    <Header showBack={true}>
        <div class="text-2xl">
            <span class="font-bold">{data.service.code}</span> to <span class="font-semibold">{data.service.dest}</span>
        </div>
    </Header>

    {#each data.alerts as alert}
        <Alert alert={alert} major={data.service.cancelled} />
    {/each}

    <a href={data.operator.url} class="w-full mt-2 max-w-2xl hover:bg-amber-700/5 dark:hover:bg-gray-500/20">
        <div class="panel w-full pl-8 pr-8 pt-4 pb-4 flex flex-row items-center">
            <Fa icon={faBus} size="lg" class="mr-4" />
            <div class="flex-grow">Operated by {data.operator.name}</div>
            <Fa icon={faChevronRight} size="1x" />
        </div>
    </a>

    {#if branch.stops.some((stop) => !stop.major)}
        <div class="panel w-full mt-2 pl-8 pr-8 pt-4 pb-4 flex flex-row items-center justify-center">
            <label for="expand">Show minor stops</label>
            <input id="expand" type="checkbox" class="ml-2" bind:checked={expand}>
        </div>
    {/if}

    <div class="panel w-full mt-2 py-4 flex flex-col items-center" class:py-4={!train} class:py-2={train && data.branches.length === 1}>
        {#if data.branches.length > 1}
            <div class="w-full mb-2 flex flex-row items-center justify-between text-center border-b border-b-amber-900/20 dark:border-b-gray-200/20">
                {#each data.branches as branch, i}
                    <a href="#" class="w-full px-8 py-4 hover:bg-amber-700/5 dark:hover:bg-gray-500/20 border-r-amber-900/20 dark:border-r-gray-200/20"
                       class:border-r={i !== data.branches.length - 1} class:selected={i === b} onclick={() => b = i}>{branch.dest}</a>
                {/each}
            </div>
        {/if}
        {#key branch.stops}
            {#if branch.connections.from}
                <Connection service={branch.connections.from} type="from" on_previous={branch.realtime?.on_previous ?? false}></Connection>
            {/if}
            {#each branch.stops as stop, i}
                {#if expand || stop.major || branch.stops.length < 20 || i === 0 || i === branch.stops.length - 1 || i === realtimeData.stop}
                    <Stop type={i === 0 && !branch.connections.from ? "origin" : i === branch.stops.length - 1 && !branch.connections.to ? "destination" : "stop"} stop={stop}
                          realtime={i === realtimeData.stop && branch.realtime ? realtimeData.pct : undefined}
                          divider={i !== 0 && train}  />
                {/if}
            {/each}
            {#if branch.connections.to}
                <Connection service={branch.connections.to} type="to"></Connection>
            {/if}
        {/key}
    </div>

    <div class="panel w-full mt-2 flex flex-col items-center">
        {#if browser && data}
            <Map width="100%" height="300px" lon={lon} lat={lat} bind:zoom>
                <Tiles />
                <GeoJSON data={geoData} options={geoOptions} />
                {#each data.branches.flatMap(br => br.stops) as stop}
                    <HTMLMarker lon={stop.long} lat={stop.lat} popup="{timeFmt(stop.dep)}<br><b>{stop.display_name}{stop.ind ? ` (${stop.ind})` : ''}</b><br>{stop.loc ? stop.loc : ''}"
                                divIcon={{ className: "bg-amber-400 rounded border border-black" }} zIndex={-1000}/>
                {/each}
                {#if branch.realtime?.pos}
                    <HTMLMarker lon={lon} lat={lat} divIcon={{
                            html: `<div class='bg-white border border-black h-full w-full rounded-tr-full' style='transform: rotate(${rotation}deg) ${flip}'></div>`,
                            className: "", iconSize: [20, 12] }}
                            popup="{branch.realtime.vehicle?.license ? '<b>' + branch.realtime.vehicle.license + '</b><br>' : ''}
                                   {branch.realtime.vehicle?.name ? '<i>' + branch.realtime.vehicle.name + '</i><br>' : ''}
                                   {(!branch.realtime.vehicle?.license && !branch.realtime.vehicle?.name && !branch.realtime.vehicle?.occupancy_pct) ? 'No vehicle information available' : ''}
                                   {branch.realtime.vehicle?.occupancy_pct ? 'Occupancy: ' + branch.realtime.vehicle.occupancy_pct + '%' : ''}" />
                {/if}
            </Map>
        {/if}
    </div>
</div>

<svelte:head>
    <title>{data ? timeFmt(branch.stops[0].dep) + (data.service.code === branch.stops[0].dep ? "" :  " " + data.service.code) + " to " + data.service.dest : "Service"} - Bus Boards</title>
</svelte:head>

<style lang="postcss">
    .selected {
        @apply bg-amber-700/[0.1] dark:bg-gray-500/[0.1];
    }
</style>