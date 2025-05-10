<script lang="ts">
    import {
        faChevronUp,
        faChevronDown,
        faStar as faStarActive,
        faTrain
    } from "@fortawesome/free-solid-svg-icons";
    import {faStar as faStarInactive} from "@fortawesome/free-regular-svg-icons";
    import Fa from "svelte-fa";
    import type {SearchResult} from "../api.type";
    import {starredStops} from "../stores";
    import {goto} from "$app/navigation";

    interface Props {
        result: SearchResult;
        moveable?: boolean;
    }

    let { result, moveable = false }: Props = $props();

    let isHovered = $state(false);

    function toggle_pin(event: Event) {
        event.preventDefault()
        event.stopPropagation()
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

    function move_up(e: Event) {
        e.stopPropagation()
        if(pinned) {
            let index = $starredStops.indexOf(result)
            $starredStops.splice(index, 1)
            $starredStops.splice(index == 0 ? $starredStops.length : index - 1, 0, result)
            $starredStops = $starredStops
        }
    }

    function move_down(e: Event) {
        e.stopPropagation()
        if(pinned) {
            let index = $starredStops.indexOf(result)
            $starredStops.splice(index, 1)
            $starredStops.splice(index == $starredStops.length ? 0 : index + 1, 0, result)
            $starredStops = $starredStops
        }
    }

    function navigate(e: Event) {
        e.stopPropagation()
        goto(`/stop/${result.locality}/${encodeURIComponent(result.name)}`)
    }

    let pinned = $derived($starredStops.some((stop: SearchResult) =>
        stop.name === result.name && stop.parent === result.parent && stop.locality === result.locality && stop.qualifier === result.qualifier))
</script>

<tr class="cursor-pointer transition-colors hover:bg-gray-50 dark:hover:bg-slate-900"
    class:border-b-gray-400={moveable} class:dark:border-b-white={moveable}
    class:hover-bg-amber-700-5={true} class:dark-hover-bg-gray-500-20={true}
    role="link"
    onclick={navigate}>
    <td class="pl-4 pr-2 pt-2 pb-2 w-full">
        <div class="flex flex-row w-full align-middle items-center">
            <div class="w-full flex-grow">
                <div class="text-lg">{result.name}</div>
                <div class="text-sm">{result.parent}</div>
            </div>
            {#if result.station}
                <Fa icon={faTrain} class="inline-block pr-4 text-xl" />
            {/if}
        </div>
    </td>
    <td class="align-middle text-right text-xl pr-4">
        <button onmouseover={() => isHovered = true} onfocus={() => isHovered = true}
             onmouseout={() => isHovered = false} onblur={() => isHovered = false}
             onclick={toggle_pin} title={(pinned ? "Unfavourite " : "Favourite ") + result.name} tabindex="0">
            <Fa icon={(pinned && !isHovered) || (!pinned && isHovered) ? faStarActive : faStarInactive} class="inline-block" />
        </button>
    </td>
    {#if moveable}
    <td class="align-middle">
        <div class="flex flex-col items-center pr-4">
            <button onclick={move_up} title={"Move up " + result.name} tabindex="0" class="hover:text-amber-500">
                <Fa icon={faChevronUp} />
            </button>
            <button onclick={move_down} title={"Move down " + result.name} tabindex="0" class="hover:text-amber-500">
                <Fa icon={faChevronDown} />
            </button>
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