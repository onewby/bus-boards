<!-- Adapted from @svelte-parts/map -->
<script lang="ts">
    import { getContext } from 'svelte'
    import L, {DivIconOptions} from 'leaflet'

    export let lat = 0
    export let lon = 0
    export let popup = "";
    export let divIcon: DivIconOptions = {};
    export let zIndex: number | undefined = undefined;

    const { getMap } = getContext('leaflet_map')
    const map = getMap()

    const icon = L.divIcon(divIcon)
    const d = L.marker([lat, lon], {icon: icon, zIndexOffset: zIndex})
    if (popup) { d.bindPopup(popup) }
    if (map) { d.addTo(map) }

    $: lat, lon, d.setLatLng({lat: lat, lng: lon})
</script>