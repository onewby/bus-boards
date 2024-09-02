<script lang="ts">
    import type {LinkedService} from "../../../../api.type";

    type ConnectionType = "from" | "to"

    export let service: LinkedService
    export let type: ConnectionType
    export let on_previous = false;
</script>

<a data-sveltekit-reload href="/service/bus/{service.trip_id}" class="w-full relative">
    <div class="flex flex-row w-full pl-8 pr-8 hover:bg-amber-700/5 dark:hover:bg-gray-500/10 text-gray-600 dark:text-gray-300 items-stretch">
        <div class="flex flex-col">
            <div class="flex-auto mr-4 flex justify-center" class:invisible={type === "from"}><div class="h-full w-1 bg-black dark:bg-white"></div></div>
            <div class="h-8 w-8 max-h-8 max-w-8 flex-shrink-0 flex-grow bg-cover mr-4 bg-black dark:bg-white" style="mask-image: url('/connection/{type}.svg'); -webkit-mask-image: url('/connection/{type}.svg');"></div>
            <div class="flex-auto mr-4 flex justify-center" class:invisible={type === "to"}><div class="h-full w-1 bg-black dark:bg-white"></div></div>
        </div>
        <div class="flex items-center">
            <div>
                Links {type} the
                <span class="font-medium">{service.dep_time}</span>
                 {type}
                <span class="font-medium">{service.location}</span>
                {#if on_previous} (vehicle currently on this service){/if}
            </div>
        </div>
    </div>
</a>