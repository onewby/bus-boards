<script lang="ts">
    import Header from "../../header.svelte";
    import Service from "./service.svelte";
    import { page } from '$app/stores';
    import Fa from "svelte-fa";
    import type {StopData} from "../../../api.type";

    // let departures = true
    export let data: StopData;

    let ct = new Date(Date.now())
    const dateNum = (num: number) => num.toString().padStart(2, "0")
    let time = $page.url.searchParams.get("date") ?? `${ct.getFullYear()}-${dateNum(ct.getMonth() + 1)}-${dateNum(ct.getDate())}T${dateNum(ct.getHours())}:${dateNum(ct.getMinutes())}`

    let operators = {}
    $: {
        operators = {}
        data.times.forEach(time => {
            if(operators[time.operator_name] === undefined) operators[time.operator_name] = new Set()
            operators[time.operator_name].add(time.operator_id)
        })
    }
    $: operatorNames = Object.keys(operators).sort((a, b) => a[1]?.localeCompare(b[1]) ?? 0)
    let collator = new Intl.Collator([], {numeric: true, sensitivity: 'base'})

    let operatorFilter = $page.url.searchParams.get("operator") ?? ""
    let stanceFilter = $page.url.searchParams.get("stance") ?? ""

    let filterURL = ""
    $: {
        let params = new URLSearchParams()
        params.set("date", time)
        if(operatorFilter !== "") params.set("operator", operatorFilter)
        if(stanceFilter !== "") params.set("stance", stanceFilter)
        filterURL = `/stop/${$page.params.stop}?${params.toString()}`
    }

    $: filteredTimes = data.times.filter(stop =>
        (operatorFilter === "" || (operators[operatorFilter] !== undefined && operators[operatorFilter].has(stop.operator_id)))
        && (stanceFilter === "" || stop.indicator === stanceFilter))
</script>

<div class="w-full h-fit overflow-scroll flex flex-col justify-start items-center text-center max-w-full pt-4 pb-8 dark:text-white">
    <Header>
        <div>{#if data.stop.locality_name}{data.stop.locality_name} › {/if}<span class="font-semibold">{data.stop.name}</span></div>
    </Header>

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
        <div class="flex flex-row items-center">
            <input type="datetime-local" bind:value={time} class="dark:bg-gray-800">
            <a href={filterURL}>
                <button class="bg-blue-600 hover:bg-blue-700 dark:bg-blue-800 dark:hover:bg-blue-900 transition-colors rounded-md text-white pl-4 pr-4 pt-2 pb-2 ml-2">Go</button>
            </a>
        </div>
    </div>

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