<script>
    import "../app.css";
    import Copyright from "./copyright.svelte";
    import {onMount} from "svelte";
    import { pwaInfo } from 'virtual:pwa-info';

    onMount(async () => {
        if (pwaInfo) {
            const { registerSW } = await import('virtual:pwa-register')
            registerSW({
                immediate: true
            })
        }
    })

    $: webManifest = pwaInfo ? pwaInfo.webManifest.linkTag : ''
</script>

<svelte:head>
    {@html webManifest}
</svelte:head>

<div class="min-w-screen min-h-screen w-full h-fit bg-gray-800 bg-center bg-cover bg-fixed transition-colors -z-40">
    <div class="min-w-screen min-h-screen w-full h-fit bg-gradient-to-r from-amber-500/90 to-amber-300/90 dark:bg-none dark:bg-slate-800 -z-30 backdrop-blur flex flex-col">
        <div class="flex flex-grow">
            <slot />
        </div>
        <Copyright />
    </div>
</div>

<style>
    :global(.mapPopup) {
        @apply font-sans;
    }
</style>