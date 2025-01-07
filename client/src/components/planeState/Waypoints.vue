<script setup lang="ts">
import { computed, onUnmounted, ref, watchEffect } from "vue";
import * as L from "leaflet";
import { getWorldData } from "@/staticData.ts";
import config from "@/config";
import { rawMap } from "@/map.ts";
import type { PlaneState } from "@/plane.ts";

const { planeState } = defineProps<{ planeState: PlaneState }>();

const waypointList = computed(() => {
  const past = planeState.info?.pos.planner.past_route.map((a) => a.name);
  const future = planeState.info?.pos.planner.route.map((a) => a.name);
  return { past, future };
});

let showWaypoints = ref(false);
let waypointFeatureGroup: L.FeatureGroup | undefined;
watchEffect(async () => {
  waypointFeatureGroup?.remove();
  if (!showWaypoints.value) {
    waypointFeatureGroup = undefined;
  } else {
    const wd = await getWorldData();
    const pastWaypoints =
      waypointList.value.past
        ?.map((name) => wd.waypoints.find((a) => a.name === name)!)
        .map((a) =>
          L.circleMarker(config.world2map(a.pos), {
            radius: 5,
            color: "#ff0000",
          }).bindTooltip(a.name, { permanent: true, interactive: false }),
        ) ?? [];
    const futureWaypoints =
      waypointList.value.future
        ?.map((name) => wd.waypoints.find((a) => a.name === name)!)
        .map((a) =>
          L.circleMarker(config.world2map(a.pos), {
            radius: 5,
            color: "#00ff00",
          }).bindTooltip(a.name, { permanent: true, interactive: false }),
        ) ?? [];

    waypointFeatureGroup = L.featureGroup([
      ...pastWaypoints,
      ...futureWaypoints,
    ]).addTo(rawMap());
  }
});

onUnmounted(() => {
  waypointFeatureGroup?.remove();
});
</script>

<template>
  <small>
    <span
      >Waypoints: <i>{{ waypointList.past?.join(", ").trim() ?? "" }}</i
      >{{ waypointList.past?.length && waypointList.future?.length ? ", " : ""
      }}<b>{{ waypointList.future?.join(", ").trim() ?? "" }}</b></span
    >
    <input id="showWaypoints" v-model="showWaypoints" type="checkbox" />
    <label for="showWaypoints">Show Waypoints</label>
  </small>
</template>
