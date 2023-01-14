<!-- Fixed and modified version of @svelte-parts/map GeoJSON -->
<script lang="ts">
    import { getContext } from 'svelte'
    import L, {GeoJSONOptions, Map} from 'leaflet';
    import {GeoJsonObject} from 'geojson';

    export let data: GeoJsonObject = {type: "FeatureCollection"}
    export let options: GeoJSONOptions = {}

    const { getMap } = getContext('leaflet_map')
    const map: Map = getMap()

    let geoJSON = L.geoJSON(data, options)
    geoJSON.addTo(map)
    map.fitBounds(geoJSON.getBounds(), {padding: [2, 2]})
</script>