<script lang="ts">
    import Header from "../../../header.svelte";
    import Service from "./service.svelte";
    import { page } from '$app/stores';

    import Fa from "svelte-fa";
    import type {SearchResult, StopData} from "../../../../api.type";
    import {faExclamationTriangle, faMap, faXmark} from "@fortawesome/free-solid-svg-icons";
    import {slide} from "svelte/transition";

    import Map from "../../../../map/Map.svelte";
    import Tiles from "../../../../map/Tiles.svelte";
    import 'leaflet/dist/leaflet.css';
    import type {GeoJSONOptions} from "leaflet";
    import GeoJSON from "../../../../map/GeoJSON.svelte";
    import L from "leaflet";
    import {browser} from "$app/environment";
    import {createFloatingActions} from "svelte-floating-ui";
    import {size} from "@floating-ui/core";
    import Alert from "../../../Alert.svelte";

    // let departures = true
    export let data: StopData;

    let ct = new Date(Date.now())
    const dateNum = (num: number) => num.toString().padStart(2, "0")
    let time = $page.url.searchParams.get("date") ?? `${ct.getFullYear()}-${dateNum(ct.getMonth() + 1)}-${dateNum(ct.getDate())}T${dateNum(ct.getHours())}:${dateNum(ct.getMinutes())}`

    let operators: Record<string, Set<string>> = {}
    $: {
        operators = {}
        data.times.forEach(time => {
            if(operators[time.operator_name] === undefined) operators[time.operator_name] = new Set()
            operators[time.operator_name].add(time.operator_id)
        })
    }
    let collator = new Intl.Collator([], {numeric: true, sensitivity: 'base'})

    let operatorFilter = $page.url.searchParams.get("operator") ?? ""
    let stanceFilter = $page.url.searchParams.get("stance") ?? ""

    let filterURL = ""
    $: {
        let params = new URLSearchParams()
        params.set("date", time)
        if(operatorFilter !== "") params.set("operator", operatorFilter)
        if(stanceFilter !== "") params.set("stance", stanceFilter)
        if(data.filter) {
            params.set("filterLoc", data.filter.locality)
            params.set("filterName", data.filter.name)
        }
        filterURL = `/stop/${$page.params.locality}/${$page.params.name}?${params.toString()}`
    }

    $: filteredTimes = data.times.filter(stop =>
        (operatorFilter === "" || (operators[operatorFilter] !== undefined && operators[operatorFilter].has(stop.operator_id)))
        && (stanceFilter === "" || stop.indicator.find(ind => ind === stanceFilter) !== undefined))

    let showMap = false;
    let zoom = 20

    $: geoData = {
        "type": "FeatureCollection",
        "features": data?.stances.map(stance => ({
            "type": "Feature",
            "properties": {
                "street": stance.street,
                "indicator": stance.indicator
            },
            "geometry": {
                "coordinates": [stance.long, stance.lat],
                "type": "Point"
            }
        }))
    }

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
            marker.bindPopup(`<b>${feature.properties.street ?? data.stop.name}</b><br>${feature.properties.indicator}`, popupOptions)
            return marker
        }
    }

    const [ floatingRef, floatingContent ] = createFloatingActions({
        strategy: "absolute",
        placement: "bottom-start",
        middleware: [
            size({
                apply({rects, elements}) {
                    Object.assign(elements.floating.style, {
                        width: `${rects.reference.width}px`,
                    });
                },
            })
        ]
    });

    let destInput = ""
    let results: SearchResult[] = []

    async function onDestSearch(e) {
        if(e.target.value !== "") {
            let resp = await fetch(`/api/search?query=${encodeURIComponent(e.target.value.trim())}`)
            results = await resp.json()
        } else {
            results = []
        }
    }

    function bindDestination(result: SearchResult | undefined) {
        data.filter = result
        results = []
    }

    function clearDestination() {
        let name = data.filter!.name
        let loc = data.filter!.parent
        data.filter = undefined
        results = []
        destInput = loc + " " + name
        onDestSearch({target: {value: destInput}})
    }
</script>

<div class="w-full h-fit flex flex-col justify-start items-center text-center max-w-full pt-4 pb-8 dark:text-white">
    <Header>
        <div>{#if data.stop.locality_name}<a href="/locality/{data.stop.locality_code}" class="hover:underline">{data.stop.locality_name}</a> › {/if}<span class="font-semibold">{data.stop.name}</span></div>
        <slot slot="buttons">
            <div class="border-l-black border-l cursor-pointer pl-4 pr-4 pt-2 pb-2 hover:bg-amber-700/5 dark:hover:bg-gray-500/20" on:click={() => showMap = !showMap}>
                <Fa icon={faMap} class="inline-block" />
            </div>
        </slot>
    </Header>

    {#if browser && showMap && data}
        {@const avgLon = data.stances.map(s => s.long).reduce((a, b) => a + b) / data.stances.length}
        {@const avgLat = data.stances.map(s => s.lat).reduce((a, b) => a + b) / data.stances.length}
        <div class="panel w-full mt-2 flex flex-row flex-wrap items-center justify-evenly gap-x-2 gap-y-2" transition:slide>
            <Map width="100%" height="300px" lon={avgLon} lat={avgLat} bind:zoom>
                <Tiles />
                <GeoJSON data={geoData} options={geoOptions} />
            </Map>
        </div>
    {/if}

    <div class="panel w-full mt-2 pl-4 pr-4 pt-4 pb-4 flex flex-row flex-wrap items-center justify-evenly gap-x-2 gap-y-2">
        <!--<div class="flex flex-row items-center">
            <div>Arrivals</div>
            <div class="w-fit inline-block ml-4 mr-4"><Toggle hideLabel label="Toggle arrivals/departures" toggledColor="#f59e0b"></Toggle></div>
            <div>Departures</div>
        </div>-->
        {#if data}
            <div class="flex flex-col sm:flex-row gap-x-4 w-full">
                <div class="flex flex-row items-center w-full">
                    <label for="operator">Operator</label>
                    <select id="operator" class="ml-4 dark:bg-gray-800 w-full" bind:value={operatorFilter}>
                        <option name=""></option>
                        {#each Object.keys(operators) as operator}
                            <option value={operator}>{operator}</option>
                        {/each}
                    </select>
                </div>
                <div class="flex flex-row items-center w-full">
                    <label for="stance">Stance</label>
                    <select id="stance" class="ml-4 dark:bg-gray-800 w-full" bind:value={stanceFilter}>
                        <option name=""></option>
                        {#each [...new Set(data.stances.filter(st => st.indicator !== null).map(st => st.indicator))].sort(collator.compare) as stance}
                            <option value={stance}>{stance}</option>
                        {/each}
                    </select>
                </div>
            </div>
        {/if}
        <div class="flex flex-col sm:flex-row gap-x-4 w-full">
            <div class="flex flex-row items-center w-full gap-x-4">
                <label for="stop" class="whitespace-nowrap">Stops at</label>
                {#if data.filter}
                    <div class="flex flex-row items-center px-2 py-1 w-full cursor-pointer border border-gray-500 bg-white dark:bg-slate-800 hover:bg-gray-50 dark:hover:bg-slate-900" on:click={clearDestination}>
                        <div class="flex flex-col w-full items-start">
                            <div>{data.filter.name}</div>
                            <div class="text-xs">{data.filter.parent}</div>
                        </div>
                        <Fa icon={faXmark} class="inline-block" />
                    </div>
                {:else}
                    <input id="stop" class="bg-white dark:bg-gray-800 w-full" use:floatingRef on:input={onDestSearch} bind:value={destInput}>
                {/if}
            </div>
            <div class="flex flex-row items-start w-fit">
                <input type="datetime-local" bind:value={time} class="dark:bg-gray-800">
                <a href={filterURL}>
                    <button class="bg-blue-600 hover:bg-blue-700 dark:bg-blue-800 dark:hover:bg-blue-900 transition-colors rounded-md text-white pl-4 pr-4 pt-2 pb-2 ml-2">Go</button>
                </a>
            </div>
        </div>
    </div>

    {#if results.length > 0}
    <div class="flex flex-col w-full absolute z-50" use:floatingContent>
        {#each results as result}
            <div class="flex flex-col items-start px-2 py-1 w-full cursor-pointer border dark:border-gray-500 bg-white dark:bg-slate-800 hover:bg-gray-50 dark:hover:bg-slate-900" on:click={() => bindDestination(result)}>
                <div>{result.name}</div>
                <div class="text-xs">{result.parent}</div>
            </div>
        {/each}
    </div>
    {/if}

    {#each data.alerts as alert}
        <Alert alert={alert} />
    {/each}

    <div class="panel w-full mt-2 flex flex-col">
        {#if filteredTimes.length === 0}
            <div class="p-4">No services available at this stop.</div>
        {:else}
            {#each filteredTimes.slice(0, filteredTimes.length - 1) as service}
                <Service service={service} />
                <hr class="mt-0 mb-0 ml-2 mr-2 border-gray-400 dark:border-white">
            {/each}
            <Service service={filteredTimes[filteredTimes.length - 1]} />
        {/if}
    </div>
</div>

<svelte:head>
    <title>{data?.stop.name ?? "Stop"} - Bus Boards</title>
</svelte:head>