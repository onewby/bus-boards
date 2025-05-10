<script lang="ts">
    import Fa from "svelte-fa";
    import {faBus, faChevronLeft} from "@fortawesome/free-solid-svg-icons";

    interface Props {
        showBack?: boolean;
        buttons?: import('svelte').Snippet;
        children?: import('svelte').Snippet;
    }

    let { showBack = false, buttons, children }: Props = $props();
</script>

<div class="max-w-2xl flex flex-col sm:flex-row w-full justify-start items-center text-2xl sm:pt-4 pb-4">
    <div class="panel mr-4 text-xl flex flex-row items-center mb-6 sm:mb-0">
        <!-- Back button -->
        {#if showBack}
            <div class="border-r-black border-r cursor-pointer pl-4 pr-4 pt-2 pb-2 hover:bg-amber-700/5 dark:hover:bg-gray-500/20" onclick={() => window.history.back()}>
                <Fa icon={faChevronLeft} class="inline-block" />
            </div>
        {/if}
        <!-- Site name -->
        <a href="/" class="pl-4 pr-4 pt-2 pb-2 flex-shrink-0 w-fit whitespace-nowrap hover:bg-amber-700/5 dark:hover:bg-gray-500/20">
            <Fa icon={faBus} class="inline-block mr-2" /> Bus Boards
        </a>
        {@render buttons?.()}
    </div>
    {@render children?.()}
</div>