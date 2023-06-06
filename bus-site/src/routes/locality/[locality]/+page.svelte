<script>
    import Header from "../../header.svelte";
    import {faMap} from "@fortawesome/free-solid-svg-icons";
    import Fa from "svelte-fa";
    import {slide} from "svelte/transition";

    import Map from "@svelte-parts/map/Map.svelte";
    import Tiles from "@svelte-parts/map/tiles/Tiles.svelte";
    import 'leaflet/dist/leaflet.css';
    import GeoJSON from "../../service/[type]/[service]/GeoJSON.svelte";
    import L from "leaflet";
    import {browser} from "$app/environment";

    export let data

    let showMap = false
    let zoom = 15

    function childToGeoJSON(child, type) {
        return {
            "type": "Feature",
            "properties": {
                "type": type,
                "id": child.id,
                "name": child.name
            },
            "geometry": {
                "coordinates": [child.long, child.lat],
                "type": "Point"
            }
        };
    }

    $: geoData = {
        "type": "FeatureCollection",
        "features": data?.children.map(child => childToGeoJSON(child, "locality")).concat(
            data.results.map(child => childToGeoJSON(child, "stop")))
    }

    const popupOptions = {
        maxWidth: 108,
        className: "mapPopup"
    }

    const geoOptions = {
        style: function(feature) {
            return {
                stroke: true,
                color: "rgb(100, 83, 9)",
                weight: 4
            }
        },
        pointToLayer: function (feature, latlng) {
            let divIcon = L.divIcon({
                className: "h-full w-full border-black border " + (feature.properties.type === "stop" ? "bg-amber-500 rounded": "bg-slate-800 rounded-full")
            })
            let marker = L.marker(latlng, {icon: divIcon})
            marker.bindPopup(`${feature.properties.type === "stop" ? "Stop" : "Locality"}<br><b><a href="/${feature.properties.type}/${feature.properties.id}">${feature.properties.name}</a></b>`, popupOptions)
            return marker
        }
    }
</script>

<div class="w-full h-fit flex flex-col justify-start items-center max-w-full pt-4 pb-8 dark:text-white">
    <Header>
        <div class="text-2xl">
            {#if data.parent.id}<a href="/locality/{data.parent.id}" class="hover:underline">{data.parent.name}</a> › {/if}<span class="font-semibold">{data.name}</span>
        </div>
        <slot slot="buttons">
            {#if data.results || data.children}
            <div class="border-l-black border-l cursor-pointer pl-4 pr-4 pt-2 pb-2 hover:bg-amber-700/5 dark:hover:bg-gray-500/20" on:click={() => showMap = !showMap}>
                <Fa icon={faMap} class="inline-block" />
            </div>
            {/if}
        </slot>
    </Header>

    {#key data}
    {#if browser && showMap && data}
        {@const combined = data?.children?.concat(data.results) ?? data?.results ?? []}
        {@const avgLon = combined.map(s => s.long).reduce((a, b) => a + b) / combined.length}
        {@const avgLat = combined.map(s => s.lat).reduce((a, b) => a + b) / combined.length}
        <div class="panel w-full mt-2 flex flex-row flex-wrap items-center justify-evenly gap-x-2 gap-y-2" transition:slide>
            <Map width="100%" height="300px" lon={avgLon} lat={avgLat} bind:zoom>
                <Tiles />
                <GeoJSON data={geoData} options={geoOptions} />
            </Map>
        </div>
    {/if}
    {/key}

    <div class="panel w-full mt-2 flex flex-col">
        {#each data.children as result, i}
            <a class="pl-4 pr-4 pt-2.5 pb-2.5 transition-colors hover:bg-gray-50 dark:hover:bg-slate-900" href="/locality/{result.id}">
                <p class="text-lg">{result.name}</p>
            </a>
            {#if i !== data.children.length - 1}
                <hr class="mt-0 mb-0 ml-2 mr-2 border-gray-400 dark:border-white">
            {/if}
        {/each}
    </div>

    <div class="panel w-full mt-2 flex flex-col">
        {#each data.results as result, i}
            <a class="pl-4 pr-4 pt-2.5 pb-2.5 transition-colors hover:bg-gray-50 dark:hover:bg-slate-900" href="/stop/{result.id}">
                <p class="text-lg">{result.name}</p>
            </a>
            {#if i !== data.results.length - 1}
                <hr class="mt-0 mb-0 ml-2 mr-2 border-gray-400 dark:border-white">
            {/if}
        {/each}
    </div>
</div>

<svelte:head>
    <title>{data.name} - Bus Boards</title>
</svelte:head>