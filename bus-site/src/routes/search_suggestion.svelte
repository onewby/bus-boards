<script lang="ts">
    import {faChevronUp, faChevronDown, faStar as faStarActive} from "@fortawesome/free-solid-svg-icons";
    import {faStar as faStarInactive} from "@fortawesome/free-regular-svg-icons";
    import Fa from "svelte-fa";
    import type {SearchResult} from "../api.type";
    import {starredStops} from "../stores";

    export let result: SearchResult;
    $: pinned = $starredStops.indexOf(result) !== -1
    export let moveable = false;

    let isHovered = false;

    function toggle_pin() {
        if(pinned) {
            let index = $starredStops.indexOf(result)
            if(index >= 0) {
                $starredStops.splice(index, 1)
                $starredStops = $starredStops
            }
        } else {
            $starredStops.push(result)
            $starredStops = $starredStops
        }
    }

    function move_up() {
        if(pinned) {
            let index = $starredStops.indexOf(result)
            $starredStops.splice(index, 1)
            $starredStops.splice(index == 0 ? $starredStops.length : index - 1, 0, result)
            $starredStops = $starredStops
        }
    }

    function move_down() {
        if(pinned) {
            let index = $starredStops.indexOf(result)
            $starredStops.splice(index, 1)
            $starredStops.splice(index == $starredStops.length ? 0 : index + 1, 0, result)
            $starredStops = $starredStops
        }
    }
</script>

<tr class="cursor-pointer transition-colors hover:bg-gray-50 dark:hover:bg-slate-900"
    class:border-b-gray-400={moveable} class:dark:border-b-white={moveable}
    class:hover-bg-amber-700-5={moveable} class:dark-hover-bg-gray-500-20={moveable}
    on:click={() => window.location.href=`/stop/${result.locality}/${result.name}`}>
    <td class="pl-4 pr-2 pt-2 pb-2 w-full">
        <div class="text-lg">{result.name}</div>
        <div class="text-sm">{result.parent}</div>
    </td>
    <td class="align-middle text-right text-xl pr-4">
        <div on:mouseover={() => isHovered = true} on:focus={() => isHovered = true}
             on:mouseout={() => isHovered = false} on:blur={() => isHovered = false}
             on:click|stopPropagation={toggle_pin} on:keypress={toggle_pin} role="button" tabindex="-1">
            <Fa icon={(pinned && !isHovered) || (!pinned && isHovered) ? faStarActive : faStarInactive} class="inline-block" />
        </div>
    </td>
    {#if moveable}
    <td class="align-middle">
        <div class="flex flex-col items-center pr-4">
            <div on:click|stopPropagation={move_up} class="hover:text-amber-500"><Fa icon={faChevronUp} /></div>
            <div on:click|stopPropagation={move_down} class="hover:text-amber-500"><Fa icon={faChevronDown} /></div>
        </div>
    </td>
    {/if}
</tr>

<style lang="postcss">
    .hover-bg-amber-700-5 {
        @apply hover:bg-amber-700/5
    }

    .dark-hover-bg-gray-500-20 {
        @apply dark:hover:bg-gray-500/20
    }
</style>