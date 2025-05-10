<script lang="ts">
  import { getContext } from 'svelte'
  import L from 'leaflet'

  interface Props {
    url?: string;
    maxZoom?: number;
    attribution?: string;
  }

  let { url = 'https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', maxZoom = 19, attribution = '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors' }: Props = $props();

  const { getMap } = getContext<{ getMap: () => L.Map }>('leaflet_map')
  const map = getMap()

  if (map) {
    L.tileLayer(url, {
      maxZoom,
      attribution,
    }).addTo(map)
  }
</script>