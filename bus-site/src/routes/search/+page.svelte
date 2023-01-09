<script lang="ts">
    import Header from "../header.svelte";
    import {page} from "$app/stores";
    import InfiniteLoading from "svelte-infinite-loading";
    import type {SearchResult} from "../../api.type";

    let results = []
    let pageNum = 0

    async function handle({detail: {loaded, complete, error}}) {
        const query = $page.url.searchParams.get("query")
        const resp = await fetch(`/api/search?query=${query}&page=${pageNum}`)
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
            Search {#if $page.url.searchParams.has("query")} for <span class="font-semibold">{$page.url.searchParams.get("query")}</span>{/if}
        </div>
    </Header>

    <div class="panel w-full mt-2 flex flex-col">
        {#each results as result, i}
            <a class="pl-4 pr-4 pt-2.5 pb-2.5 transition-colors hover:bg-gray-50 dark:hover:bg-slate-900" href="/stop/{result.id}">
                <p class="text-lg">{result.name}</p>
                {#if result.parent}
                    <p class="text-sm">{result.parent}</p>
                {/if}
            </a>
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