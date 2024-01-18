<script lang="ts">
    import Header from "../header.svelte";
    import {page} from "$app/stores";
    import InfiniteLoading from "svelte-infinite-loading";
    import type {SearchResult} from "../../api.type";
    import SearchSuggestion from "../search_suggestion.svelte";

    let results: SearchResult[] = []
    let pageNum = 0
    enum QueryType { NONE, QUERY, LOCATION }
    let queryType = $page.url.searchParams.get("query") ? QueryType.QUERY :
        $page.url.searchParams.get("lat") && $page.url.searchParams.get("lon") ? QueryType.LOCATION : QueryType.NONE

    async function handle({detail: {loaded, complete, error}}) {
        const query = $page.url.searchParams.get("query")
        const lat = $page.url.searchParams.get("lat")
        const lon = $page.url.searchParams.get("lon")
        const resp = await fetch(
            queryType === QueryType.QUERY ? `/api/search?query=${query}&page=${pageNum}`
                : `/api/search?lat=${lat}&lon=${lon}&page=${pageNum}`)
        if(resp.ok) {
            const newResults: SearchResult[] = await resp.json()
            if(newResults.length) {
                pageNum++
                results.push(...newResults)
                results = results
                loaded()
            } else {
                complete()
            }
        } else {
            error()
        }
    }

    function infiniteHandler(event) {
        handle(event)
    }
</script>

<div class="w-full h-fit flex flex-col justify-start items-center max-w-full pt-4 pb-8 dark:text-white">
    <Header>
        <div class="text-2xl">
            Search {#if queryType === QueryType.QUERY} for <span class="font-semibold">{$page.url.searchParams.get("query")}</span>
            {:else if queryType === QueryType.LOCATION} for location <span class="font-semibold">{$page.url.searchParams.get("lat")}, {$page.url.searchParams.get("lon")}</span>{/if}
        </div>
    </Header>

    <div class="panel w-full mt-2 flex flex-col">
        {#each results as result, i}
            <SearchSuggestion result={result}/>
            {#if i !== results.length - 1}
                <hr class="mt-0 mb-0 ml-2 mr-2 border-gray-400 dark:border-white">
            {/if}
        {/each}
        <InfiniteLoading on:infinite={infiniteHandler}>
            <div slot="noMore"></div>
            <div slot="noResults" class="p-4">Could not find any results.</div>
        </InfiniteLoading>
    </div>
</div>

<svelte:head>
    <title>Search - Bus Boards</title>
</svelte:head>