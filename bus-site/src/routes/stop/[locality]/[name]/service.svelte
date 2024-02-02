<script lang="ts">
    import type {StopDeparture} from "../../../../api.type.js";
    import Fa from "svelte-fa";
    import {faTrain} from "@fortawesome/free-solid-svg-icons";

    export let service: StopDeparture
</script>

<a href="/service/{service.type}/{service.trip_id}">
    <div class="w-full flex flex-row gap-x-2 text-left hover:bg-amber-700/5 dark:hover:bg-gray-500/20 pt-2 pb-2">
        <div class="w-1 ml-2" style="background-color: {service.colour}">&nbsp;</div>
        <div class="pt-2 pb-2 ml-1 min-w-[4ch] flex flex-col justify-center">{service.departure_time}</div>
        <div class="font-bold min-w-[3.5ch] flex flex-col justify-center">
            {#if service.type === "bus"}
                <div class="ml-2 pt-2 pb-2">
                    {service.route_short_name}
                </div>
            {:else}
                <div class="text-center">
                    <Fa icon={faTrain} class="block mx-auto" style="height: 1.25em;" />
                </div>
            {/if}
        </div>
        <div class="flex-grow pt-2 pb-2 flex flex-col justify-center">{service.trip_headsign}</div>
        <div class="mr-4 flex flex-col justify-center text-right">
            {#if service.indicator && service.status !== 'Cancelled'}{service.indicator}{/if}
            {#if service.status}
                {#if service.indicator && service.status !== 'Cancelled'}<br>{/if}
                {@const skipped = service.status === 'Skipped'}
                {@const delayed = service.status.startsWith('Exp')}
                {@const cancelled = service.status === 'Cancelled'}
                {@const onTime = service.status === 'On time'}
                {@const other = !(skipped || delayed || cancelled || onTime)}
                <span class="text-sm"
                      class:text-red-600={delayed || cancelled} class:dark:text-red-400={delayed || cancelled}
                      class:text-green-600={onTime} class:dark:text-green-300={onTime}
                      class:text-gray-700={other} class:dark:text-gray-300={other}
                      class:text-cyan-600={skipped} class:dark:text-cyan-400={skipped}
                      class:text-base={cancelled || skipped}>
                    {service.status}
                </span>
            {/if}
        </div>
    </div>
</a>