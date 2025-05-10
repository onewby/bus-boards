<!-- Adapted from @svelte-parts/map -->
<script lang="ts">
    import { getContext } from 'svelte'
    import L, {type DivIconOptions} from 'leaflet'

    interface Props {
        lat?: number;
        lon?: number;
        popup?: string;
        divIcon?: DivIconOptions;
        zIndex?: number | undefined;
    }

    let {
        lat = 0,
        lon = 0,
        popup = "",
        divIcon = {},
        zIndex = undefined
    }: Props = $props();

    const { getMap } = getContext<{getMap: () => L.Map}>('leaflet_map')
    const map = getMap()
    const popupOptions = {
        maxWidth: 108,
        className: "mapPopup"
    }

    const icon = L.divIcon(divIcon)
    const d = L.marker([lat, lon], {icon: icon, zIndexOffset: zIndex})
    if (popup) { d.bindPopup(popup, popupOptions) }
    if (map) { d.addTo(map) }

    $effect.pre(() => {
        lat;
        lon;
        d.setLatLng({lat: lat, lng: lon})
    });
</script>