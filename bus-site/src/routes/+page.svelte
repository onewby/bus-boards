<script lang="ts">
    import Fa from "svelte-fa";
    import {faBus, faLocationCrosshairs} from "@fortawesome/free-solid-svg-icons";
    import SearchSuggestion from "./search_suggestion.svelte";
    import type {SearchResult} from "../api.type";
    import {starredStops} from "../stores";
    import {browser} from "$app/environment";
    import {goto} from "$app/navigation";

    let input = ""
    let results: SearchResult[] = []

    async function onInput() {
        if(input !== "") {
            let resp = await fetch(`/api/search?query=${encodeURIComponent(input.trim())}`)
            // TODO show an error message if fetch fails
            results = await resp.json()
        } else {
            results = []
        }
    }

    function onSubmit() {
        if(input !== "") {
            goto(`/search?query=${encodeURIComponent(input.trim())}`)
        }
    }

    function geolocate() {
        return new Promise<GeolocationPosition>((resolve, reject) => {
            navigator.geolocation.getCurrentPosition(resolve, reject, {enableHighAccuracy: true})
        })
    }

    async function onGeolocate() {
        try {
            let result = await geolocate()
            goto(`/search?lat=${result.coords.latitude}&lon=${result.coords.longitude}`)
        } catch (e) {
            if(e instanceof GeolocationPositionError) {
                if(e.PERMISSION_DENIED) {
                    alert("Please allow finding your current location to enable geolocation.")
                } else {
                    alert("Could not find current location. Please try again!")
                }
            }
        }
    }
</script>

<div class="w-full h-full overflow-scroll prose dark:prose-invert flex flex-col justify-center items-center self-center text-center max-w-full pt-16 pb-16">
    <h1 class="text-5xl text-gray-50/90 drop-shadow"><Fa icon={faBus} class="inline-block mr-2" /> Bus Boards</h1>
    <div class="panel w-full mt-2 p-8">
        <form on:submit|preventDefault={onSubmit}>
            <label class="lead dark:text-gray-100 w-full" for="input-from">Where are you travelling from?</label>
            <div class="relative mt-4">
                <input required bind:value={input} on:input={onInput} type="text" autocomplete="street-address" class="w-full p-4 bg-gray-50/75 dark:bg-slate-900/75" placeholder="Search for a location..." id="input-from">
                <button class="absolute right-4 top-[calc(1rem+1px)]" class:hidden={!browser || !navigator?.geolocation}
                        on:click={onGeolocate} title="Search by current location">
                    <Fa icon={faLocationCrosshairs} size="lg" class="hover:text-amber-500" style="font-size: 1.5em;"></Fa>
                </button>
            </div>
            <table class="bg-white dark:bg-slate-800 border dark:border-gray-500 text-left m-0 border-collapse">
                <tbody>
                {#each results as result}
                    <SearchSuggestion result={result} />
                {/each}
                </tbody>
            </table>
            <button class="bg-blue-600 hover:bg-blue-700 dark:bg-blue-800 dark:hover:bg-blue-900 transition-colors rounded-md text-white pl-4 pr-4 pt-2 pb-2 mt-8" type="submit">Search</button>
        </form>
    </div>
    <table class="panel text-left m-0 border-collapse max-w-2xl w-full mt-4">
        <tbody>
        {#each $starredStops as stop}
            <SearchSuggestion result={stop} moveable={true} />
        {/each}
        </tbody>
    </table>
</div>

<svelte:head>
    <title>Bus Boards</title>
</svelte:head>