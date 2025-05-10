<script lang="ts">
  import { onMount, setContext } from 'svelte'
  import L from 'leaflet'

  setContext('leaflet_map', {
    getMap: () => map
  });

  interface Props {
    lat?: number;
    lon?: number;
    zoom?: number;
    width?: string;
    height?: string;
    children?: import('svelte').Snippet;
  }

  let {
    lat = 0,
    lon = 0,
    zoom = $bindable(5),
    width = '100%',
    height = '100px',
    children
  }: Props = $props();

  let style = $derived(`width:${width};height:${height};`)

  let container: HTMLDivElement = $state()
  let map: L.Map = $state()

  onMount(() => {
    map = L.map(container).setView([lat, lon], zoom)

    return () => {
      map.remove()
    }
  })
</script>

<div bind:this={container} style={style}>
  {#if map}
    {@render children?.()}
  {/if}
</div>