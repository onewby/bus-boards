<script>
    import { page } from '$app/stores';
    import {faBus} from "@fortawesome/free-solid-svg-icons";
    import Fa from "svelte-fa";
</script>

<div class="flex flex-col w-full h-full min-w-screen min-h-screen justify-center items-center p-4">
    <div class="prose"><h1 class="text-5xl text-gray-50/90 drop-shadow"><Fa icon={faBus} class="inline-block mr-2" /> Bus Boards</h1></div>
    <div class="panel dark:text-white text-center p-8 mt-12">
        <h1 class="text-2xl font-bold">Oops!</h1>
        <p>
            {#if $page.status === 404}
                Looks like you either got a bit lost, or the resource at this page no longer exists.
            {:else}
                Something has went wrong!
            {/if}
        </p>
        <a href="/"><button class="bg-blue-600 hover:bg-blue-700 dark:bg-blue-800 dark:hover:bg-blue-900 transition-colors rounded-md text-white pl-4 pr-4 pt-2 pb-2 mt-8">Go back to the home page</button></a>
        <p class="text-sm mt-8">If you are reporting an error, provide this info:</p>
        <p class="text-sm">{$page.status}: {$page.error.message}</p>
    </div>
</div>