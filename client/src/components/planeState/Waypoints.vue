<script lang="ts">
import { ref } from "vue";
let showWaypoints = ref(false);
</script>
<script setup lang="ts">
import { computed, onUnmounted, watch } from "vue";
import * as L from "leaflet";
import { getWorldData } from "@/staticData.ts";
import config from "@/config";
import { rawMap } from "@/map.ts";
import type { PlaneState } from "@/plane.ts";

const { planeState } = defineProps<{ planeState: PlaneState }>();

const pastWaypoints = computed(() =>
  planeState.info!.pos.planner.past_route.map((a) => a.name),
);
const futureWaypoints = computed(() =>
  planeState.info!.pos.planner.route.map((a) => a.name),
);

let waypointFeatureGroup: L.FeatureGroup | undefined;
watch(
  [showWaypoints, pastWaypoints, futureWaypoints, planeState],
  async () => {
    waypointFeatureGroup?.remove();
    if (!showWaypoints.value) {
      waypointFeatureGroup = undefined;
      return;
    }
    const wd = await getWorldData();
    const pastWaypointMarkers =
      pastWaypoints.value
        .map((name) => wd.waypoints.find((a) => a.name === name)!)
        .map((a) =>
          L.circleMarker(config.world2map(a.pos), {
            radius: 5,
            color: "#ff0000",
          }).bindTooltip(a.name, { permanent: true, interactive: false }),
        ) ?? [];
    const futureWaypointMarkers =
      futureWaypoints.value
        .map((name) => wd.waypoints.find((a) => a.name === name)!)
        .map((a) =>
          L.circleMarker(config.world2map(a.pos), {
            radius: 5,
            color: "#00ff00",
          }).bindTooltip(a.name, { permanent: true, interactive: false }),
        ) ?? [];

    waypointFeatureGroup = L.featureGroup([
      ...pastWaypointMarkers,
      ...futureWaypointMarkers,
    ]).addTo(rawMap());
  },
  { deep: true },
);

onUnmounted(() => {
  waypointFeatureGroup?.remove();
});
</script>

<template>
  <small>
    <span
      >Waypoints: <i>{{ pastWaypoints.join(", ").trim() ?? "" }}</i
      >{{ pastWaypoints.length && futureWaypoints.length ? ", " : ""
      }}<b>{{ futureWaypoints.join(", ").trim() ?? "" }}</b></span
    >
    <input id="showWaypoints" v-model="showWaypoints" type="checkbox" />
    <label for="showWaypoints">Show Waypoints</label>
  </small>
</template>
