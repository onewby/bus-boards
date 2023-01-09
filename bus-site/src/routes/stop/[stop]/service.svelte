<script lang="ts">
    import type {StopDeparture} from "../../../api.type.js";
    export let service: StopDeparture

    export const timeFmt = (time: string) => (Number(time.substring(0, 2)) % 24).toString().padStart(2, "0") + time.substring(2, 5)
</script>

<a href="/service/{service.trip_id}">
    <div class="w-full flex flex-row gap-x-2 text-left hover:bg-amber-700/5 dark:hover:bg-gray-500/20 pt-2 pb-2">
        <div class="w-1 ml-2" style="background-color: {service.colour}">&nbsp;</div>
        <div class="pt-2 pb-2 ml-1 min-w-[4ch]">{service.departure_time}</div>
        <div class="font-bold pt-2 pb-2 ml-2 min-w-[3.5ch]">{service.route_short_name}</div>
        <div class="flex-grow pt-2 pb-2">{service.trip_headsign}</div>
        <div class="mr-4 pt-2 pb-2">{#if service.indicator}{service.indicator}{/if}</div>
    </div>
</a>