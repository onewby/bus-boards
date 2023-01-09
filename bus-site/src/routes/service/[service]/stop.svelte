<script lang="ts">
    import type {ServiceStopData} from "../../../api.type";

    type StopType = "stop" | "origin" | "destination"

    export let type: StopType = "stop"
    export let stop: ServiceStopData
    export let realtime: number | undefined = undefined

    export const sub1hr = (time: string) => ((Number(time.substring(0, 2)) + 23) % 24).toString().padStart(2, "0") + time.substring(2, time.length)
    export const timeFmt = (time: string) => (Number(time.substring(0, 2)) % 24).toString().padStart(2, "0") + time.substring(2, 5)

    let fmtDep = timeFmt(stop.dep)
    let fmtArr = stop.arr ? timeFmt(stop.arr) : undefined

    let ct = new Date(Date.now())
    const dateNum = (num: number) => num.toString().padStart(2, "0")
    let apiTime = `${ct.getFullYear()}-${dateNum(ct.getMonth() + 1)}-${dateNum(ct.getDate())}T${sub1hr(stop.dep)}`
</script>

<a href="/stop/{stop.id}?date={apiTime}" class="w-full relative">
    {#if realtime !== undefined}<div class="bg-blue-400 h-[0.8rem] w-[0.8rem] absolute left-[2.6rem] rounded-full z-10 outline outline-2 outline-white shadow-md" style="top: calc(50% - 0.4rem - {((1-realtime) * 100).toFixed(0)}%)"></div>{/if}
    <div class="flex flex-row w-full items-center pl-8 pr-8 hover:bg-amber-700/5 dark:hover:bg-gray-500/10 items-stretch">
        <div class="flex flex-col">
            {#if type !== "origin"}<div class="flex-auto mr-4 flex justify-center"><div class="h-full w-1 bg-black dark:bg-white"></div></div>{/if}
            <div class="h-8 w-8 max-h-8 max-w-8 flex-shrink-0 flex-grow bg-cover mr-4 bg-black dark:bg-white" style="mask-image: url('/stop/{type}.svg'); -webkit-mask-image: url('/stop/{type}.svg');"></div>
            {#if type !== "destination"}<div class="flex-auto mr-4 flex justify-center"><div class="h-full w-1 bg-black dark:bg-white"></div></div>{/if}
        </div>
        <div class="flex items-center"><div>{#if stop.loc}<span class="font-medium">{stop.loc}</span>{/if} {stop.name}{#if stop.ind}<span class="text-gray-600 dark:text-gray-300">&nbsp;({stop.ind})</span>{/if}</div></div>
        <div class="flex-grow"></div>
        <div class="text-right flex items-center flex-shrink-0">
            <div>
                {#if stop.arr && fmtArr !== fmtDep}
                    arr. {fmtArr}<br>dep. {fmtDep}
                {:else}
                    {fmtDep}
                {/if}
            </div>
        </div>
    </div>
</a>