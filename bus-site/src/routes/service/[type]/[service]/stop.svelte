<script lang="ts">
    import type {ServiceStopData} from "../../../../api.type.ts";

    type StopType = "stop" | "origin" | "destination"

    interface Props {
        type?: StopType;
        stop: ServiceStopData;
        realtime?: number | undefined;
        divider?: boolean;
    }

    let {
        type = "stop",
        stop,
        realtime = undefined,
        divider = true
    }: Props = $props();

    export const sub1hr = (time: string) => ((Number(time.substring(0, 2)) + 23) % 24).toString().padStart(2, "0") + time.substring(2, time.length)
    export const timeFmt = (time: string) => (Number(time.substring(0, 2)) % 24).toString().padStart(2, "0") + time.substring(2, 5)

    let fmtDep = timeFmt(stop.dep)
    let fmtArr = stop.arr ? timeFmt(stop.arr) : undefined

    let ct = new Date(Date.now())
    const dateNum = (num: number) => num.toString().padStart(2, "0")
    let apiTime = `${ct.getFullYear()}-${dateNum(ct.getMonth() + 1)}-${dateNum(ct.getDate())}T${sub1hr(stop.dep)}`

    let thisStop: HTMLAnchorElement | undefined = $state()

    let prevSiblingHeight = $state(0)
    $effect(() => {
        let sibling = thisStop?.previousElementSibling
        if(sibling?.classList.contains("stop")) {
            prevSiblingHeight = sibling.getBoundingClientRect().height
            return
        }
        prevSiblingHeight = 0
    })
</script>

<a data-sveltekit-reload href={stop.locality ? `/stop/${stop.locality}/${encodeURIComponent(stop.name)}?date=${apiTime}` : undefined} class="w-full relative stop" bind:this={thisStop}>
    {#if realtime !== undefined}
        {@const totalHeight = prevSiblingHeight / 2 + (thisStop?.getBoundingClientRect().height ?? 0) / 2}
        <div class="bg-blue-400 h-[0.8rem] w-[0.8rem] absolute left-[2.6rem] rounded-full z-10 ring ring-white shadow-md"
             style="top: calc(-0.4rem - {prevSiblingHeight / 2}px + {totalHeight * realtime}px)"></div>
    {/if}
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
                {@const skipped = stop.status === 'Skipped'}
                {@const delayed = stop.status.startsWith('Exp')}
                {@const cancelled = stop.status === 'Cancelled'}
                {@const onTime = stop.status === 'On time'}
                {@const other = !(skipped || delayed || cancelled || onTime)}
                <div class="text-sm"
                     class:text-red-600={delayed || cancelled} class:dark:text-red-400={delayed || cancelled}
                     class:text-green-600={onTime} class:dark:text-green-300={onTime}
                     class:text-gray-700={other} class:dark:text-gray-300={other}
                     class:text-cyan-600={skipped} class:dark:text-cyan-400={skipped}
                     class:font-medium={cancelled || skipped}>
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