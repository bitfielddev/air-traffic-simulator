<script setup lang="ts">
import { computed, onUnmounted, ref, watchEffect } from "vue";
import "leaflet-easybutton/src/easy-button.css";
import "leaflet-easybutton";
import type { PlaneState } from "@/plane";
import * as L from "leaflet";
import { getWorldData } from "@/staticData";
import AirportLink from "./AirportLink.vue";
import config from "@/config";
import { airportCoords } from "@/airport.ts";
import { rawMap } from "@/map";
import { formatDuration } from "../util.ts";

const waypointList = computed(() => {
  const past = planeState.info?.pos.planner.past_route.map((a) => a.name);
  const future = planeState.info?.pos.planner.route.map((a) => a.name);
  return { past, future };
});

const { planeState } = defineProps<{ planeState: PlaneState }>();

const fromCoords = ref<[number, number]>([0, 0]);
const toCoords = ref<[number, number]>([0, 0]);
watchEffect(async () => {
  const wd = await getWorldData();
  fromCoords.value = airportCoords(
    wd.airports.find((a) => a.code === planeState.info?.flight.from)!,
  );
  toCoords.value = airportCoords(
    wd.airports.find((a) => a.code === planeState.info?.flight.to)!,
  );
});

const startTime = computed(
  () => new Date(Number(planeState.info?.start_time) * 1000),
);
const totalDuration = computed(() => {
  let waypoints = planeState.info?.pos.planner.route.map((a) => a.pos)!;
  waypoints.unshift(fromCoords.value);
  waypoints.push(toCoords.value);

  let distance = 0;
  for (let i = 0; i < waypoints.length - 1; i++) {
    let [x1, z1] = waypoints[i];
    let [x2, z2] = waypoints[i + 1];
    distance += Math.sqrt(Math.pow(x1 - x2, 2) + Math.pow(z1 - z2, 2));
  }

  return distance / planeState.info?.model.motion.max_v[0]!;
});

const currentDuration = ref(0);
const remainingDuration = ref(0);
async function updateDuration() {
  currentDuration.value =
    new Date().valueOf() / 1000 - Number(planeState.info?.start_time);

  const wd = await getWorldData();
  let [x1, , z1] = planeState.s;
  let [x2, z2] = airportCoords(
    wd.airports.find((a) => a.code === planeState.info?.flight.to)!,
  );
  let method1 =
    Math.sqrt(Math.pow(x1 - x2, 2) + Math.pow(z1 - z2, 2)) / planeState.v[0];

  let method2 = totalDuration.value - currentDuration.value;
  remainingDuration.value = Math.max(method1, method2);
}
updateDuration();
const durationUpdater = setInterval(updateDuration, 1000);

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
  clearInterval(durationUpdater);
});
</script>
<template>
  <div style="text-align: center">
    <b style="font-size: 3em"
      ><AirportLink :airport-id="planeState.info!.flight.from" /> →
      <AirportLink :airport-id="planeState.info!.flight.to" /></b
    ><br />
    <b>{{ planeState.info?.flight.code }}</b>
    operated by <b>{{ planeState.info?.flight.airline }}</b
    ><br />
    on a(n) <b>{{ planeState.info?.model.name }}</b
    ><br />
    <b>Coords:</b> {{ Math.round(planeState.s[0]) }}
    {{ Math.round(planeState.s[1]) }} <b>Alt:</b>
    {{ Math.round(planeState.s[2]) }} <br />
    <b>Velocity:</b> {{ Math.round(planeState.v[0]) }}
    {{ Math.round(planeState.v[1]) }} <br />
    <b>Departed:</b> {{ startTime.toLocaleTimeString() }} ({{
      formatDuration(currentDuration)
    }}
    ago) <br />
    <b>Est. Remaining:</b> {{ formatDuration(remainingDuration) }} <br /><br />
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
