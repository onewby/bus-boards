<script lang="ts">
    import type {ServiceStopData} from "../../../../api.type";

    type StopType = "stop" | "origin" | "destination"

    export let type: StopType = "stop"
    export let stop: ServiceStopData
    export let realtime: number | undefined = undefined
    export let divider = true

    export const sub1hr = (time: string) => ((Number(time.substring(0, 2)) + 23) % 24).toString().padStart(2, "0") + time.substring(2, time.length)
    export const timeFmt = (time: string) => (Number(time.substring(0, 2)) % 24).toString().padStart(2, "0") + time.substring(2, 5)

    let fmtDep = timeFmt(stop.dep)
    let fmtArr = stop.arr ? timeFmt(stop.arr) : undefined

    let ct = new Date(Date.now())
    const dateNum = (num: number) => num.toString().padStart(2, "0")
    let apiTime = `${ct.getFullYear()}-${dateNum(ct.getMonth() + 1)}-${dateNum(ct.getDate())}T${sub1hr(stop.dep)}`
</script>

<a data-sveltekit-reload href={stop.locality ? `/stop/${stop.locality}/${stop.name}?date=${apiTime}` : undefined} class="w-full relative">
    {#if realtime !== undefined}<div class="bg-blue-400 h-[0.8rem] w-[0.8rem] absolute left-[2.6rem] rounded-full z-10 ring ring-white shadow-md" style="top: calc(50% - 0.4rem - {((1-realtime) * 100).toFixed(0)}%)"></div>{/if}
    <div class="flex flex-row w-full pl-8 pr-8 hover:bg-amber-700/5 dark:hover:bg-gray-500/10 items-stretch">
        <div class="flex flex-col">
            <div class="flex-auto mr-4 flex justify-center" class:invisible={type === "origin"}><div class="h-full w-1 bg-black dark:bg-white"></div></div>
            <div class="h-8 w-8 max-h-8 max-w-8 flex-shrink-0 flex-grow bg-cover mr-4 bg-black dark:bg-white" style="mask-image: url('/stop/{type}.svg'); -webkit-mask-image: url('/stop/{type}.svg');"></div>
            <div class="flex-auto mr-4 flex justify-center" class:invisible={type === "destination"}><div class="h-full w-1 bg-black dark:bg-white"></div></div>
        </div>
        <div class="flex items-center" class:divider>
            <div>
                {#if stop.loc}<span class="font-medium">{stop.loc}</span>{/if}
                {stop.display_name}
                {#if stop.ind}<span class="text-gray-600 dark:text-gray-300">&nbsp;({stop.ind})</span>{/if}
            </div>
        </div>
        <div class="flex-grow" class:divider></div>
        <div class="flex flex-col text-right items-end justify-around flex-shrink-0" class:py-1={stop.status} class:divider>
            <div>
                {#if stop.arr && fmtArr !== fmtDep}
                    arr. {fmtArr}<br>dep. {fmtDep}
                {:else}
                    {fmtDep}
                {/if}
            </div>
            {#if stop.status}
                <div class="text-sm text-gray-700 dark:text-gray-300 {stop.status !== 'On time' ? 'text-red-600 dark:text-red-400' : ''}"
                     class:font-medium={stop.status === "Cancelled"}>
                    {stop.status}
                </div>
            {/if}
        </div>
    </div>
</a>

<style lang="postcss">
    .divider {
        /*border-top: 1px rgba(128, 128, 0, 0.4) solid;*/
        @apply border-t border-t-amber-900/20 dark:border-t-gray-200/20;
    }
</style>