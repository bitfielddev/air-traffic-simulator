<script setup lang="ts">
import { computed, onUnmounted, ref, watchEffect } from "vue";
import "leaflet-easybutton/src/easy-button.css";
import "leaflet-easybutton";
import type { PlaneState } from "@/plane";
import * as L from "leaflet";
import { getWorldData } from "@/staticData";
import * as map from "@/map";

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
        ?.map((name) => wd.waypoints.find((a) => a.name === name))
        .filter((a) => a !== undefined)
        .map((a) =>
          L.circleMarker(a.pos, { radius: 5, color: "#ff0000" }).bindTooltip(
            a.name,
            { permanent: true, interactive: false },
          ),
        ) ?? [];
    const futureWaypoints =
      waypointList.value.future
        ?.map((name) => wd.waypoints.find((a) => a.name === name))
        .filter((a) => a !== undefined)
        .map((a) =>
          L.circleMarker(a.pos, { radius: 5, color: "#00ff00" }).bindTooltip(
            a.name,
            { permanent: true, interactive: false },
          ),
        ) ?? [];

    waypointFeatureGroup = L.featureGroup([
      ...pastWaypoints,
      ...futureWaypoints,
    ]).addTo(map.map.value!);
  }
});

onUnmounted(() => {
  waypointFeatureGroup?.remove();
});
</script>
<template>
  <div style="text-align: center">
    <b style="font-size: 3em"
      >{{ planeState.info?.flight.from }} → {{ planeState.info?.flight.to }}</b
    ><br />
    <b>{{ planeState.info?.flight.code }}</b>
    operated by <b>{{ planeState.info?.flight.airline }}</b
    ><br />
    on a(n) <b>{{ planeState.info?.model.name }}</b
    ><br />
    <!--<b>{departTime}</b> --(<i>{duration}</i>)→ <b>{arrivalTime}</b><br />
{distance} blocks-->
    <b>Coords:</b> {{ Math.round(planeState.s[0]) }}
    {{ Math.round(planeState.s[1]) }} <b>Alt:</b>
    {{ Math.round(planeState.s[2]) }} <br />
    <b>Velocity:</b> {{ Math.round(planeState.v[0]) }}
    {{ Math.round(planeState.v[1]) }} <br /><br />
  </div>
  <small>
    <span
      >Waypoints: <i>{{ waypointList.past?.join(", ").trim() ?? "" }}</i
      >{{ waypointList.past?.length && waypointList.future?.length ? ", " : ""
      }}<b>{{ waypointList.future?.join(", ").trim() ?? "" }}</b></span
    >
    <input id="showWaypoints" v-model="showWaypoints" type="checkbox" />
    <label for="showWaypoints">Show Waypoints</label>
  </small>
  <br />
  <small>ID: {{ planeState.info?.id }}</small>
</template>
