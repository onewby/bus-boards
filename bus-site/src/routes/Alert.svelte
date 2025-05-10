<script lang="ts">
    import {faExclamationCircle, faExclamationTriangle} from "@fortawesome/free-solid-svg-icons";
    import Fa from "svelte-fa";
    import type {StopAlert} from "../api.type.ts";

    interface Props {
        alert: StopAlert;
        major?: boolean;
    }

    let { alert, major = false }: Props = $props();
</script>

<a class="panel w-full pl-8 pr-8 pt-4 pb-4 flex flex-row items-center mt-2 text-left" href={alert.url}>
    <Fa icon={major ? faExclamationCircle : faExclamationTriangle} size="lg" class="mr-4" />
    <div class="flex-grow">
        {#if alert.header}<b>{alert.header}{#if alert.description}: {/if}</b>{/if}
        {alert.description ?? ""}
    </div>
</a>