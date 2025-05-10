<script lang="ts">
  import { getContext } from 'svelte'
  import L from 'leaflet'

  interface Props {
    lat?: number;
    lon?: number;
    popup?: string | undefined;
  }

  let { lat = 0, lon = 0, popup = undefined }: Props = $props();

  const { getMap } = getContext<{ getMap: () => L.Map }>('leaflet_map')
  const map = getMap()

  const d = L.marker([lat, lon])
  if (popup) { d.bindPopup(popup) }
  if (map) { d.addTo(map) }
</script>