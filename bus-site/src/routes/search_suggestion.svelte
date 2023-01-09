<script>
    export let title = "";
    export let subtitle = "";
</script>

<tr on:click class="cursor-pointer"><td class="pl-4 pr-4 pt-2 pb-2 transition-colors hover:bg-gray-50 dark:hover:bg-slate-900">
    <div class="text-lg">{title}</div>
    <div class="text-sm">{subtitle}</div>
</td></tr>