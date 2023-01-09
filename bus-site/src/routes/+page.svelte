<script>
    import Fa from "svelte-fa";
    import {faBus} from "@fortawesome/free-solid-svg-icons";
    import SearchSuggestion from "./search_suggestion.svelte";

    let input = ""
    let results = []

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
            window.location.href = `/search?query=${encodeURIComponent(input.trim())}`
        }
    }
</script>

<div class="w-full h-full overflow-scroll prose dark:prose-invert flex flex-col justify-center items-center self-center text-center max-w-full pt-16 pb-16">
    <h1 class="text-5xl text-gray-50/90 drop-shadow"><Fa icon={faBus} class="inline-block mr-2" /> Bus Boards</h1>
    <div class="panel w-full mt-2 p-8">
        <form on:submit={onSubmit}>
            <label class="lead dark:text-gray-100 w-full" for="input-from">Where are you travelling from?</label>
            <input required bind:value={input} on:input={onInput} type="text" autocomplete="street-address" class="w-full mt-4 p-4 bg-gray-50/75 dark:bg-slate-900/75" placeholder="Search for a location..." id="input-from">
            <table class="bg-white dark:bg-slate-800 border dark:border-gray-500 text-left m-0 border-collapse">
                <tbody>
                {#each results as result}
                    <SearchSuggestion title={result.name} subtitle={result.parent} on:click={window.location.href=`/stop/${result.id}`} />
                {/each}
                </tbody>
            </table>
            <button class="bg-blue-600 hover:bg-blue-700 dark:bg-blue-800 dark:hover:bg-blue-900 transition-colors rounded-md text-white pl-4 pr-4 pt-2 pb-2 mt-8" type="submit">Search</button>
        </form>
    </div>
    <!--<div class="max-w-2xl flex flex-row space-x-4 w-full h-fit">
        <button class="btn mt-4 pl-8 pr-8 pt-4 pb-4 text-2xl font-bold w-full">
            Operators
        </button>
        <button class="btn mt-4 pl-8 pr-8 pt-4 pb-4 text-2xl font-bold w-full">
            Locations
        </button>
    </div>-->
    <!--<div class="max-w-2xl flex flex-row flex-wrap w-full h-fit gap-x-2">
        {#each operators as operator}
            <button class="btn mt-2 pl-4 pr-4 pt-2 pb-2 text-lg">
                {operator}
            </button>
        {/each}
    </div>-->
</div>

<svelte:head>
    <title>Bus Boards</title>
</svelte:head>