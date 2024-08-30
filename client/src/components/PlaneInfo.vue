<script setup lang="ts">
import { computed } from "vue";
import "leaflet-easybutton/src/easy-button.css";
import "leaflet-easybutton";
import type { PlaneState } from "@/plane";

const { planeState } = defineProps<{ planeState: PlaneState }>();

const waypointList = computed(() => {
  const past =
    planeState.info?.pos.planner.past_route.map((a) => a.name).join(", ") ?? "";
  const present = planeState.info?.pos.planner.route.at(0)?.name ?? "";
  const future =
    planeState.info?.pos.planner.route.map((a) => a.name).join(", ") ?? "";
  return { past, present, future };
});

let showWaypoints: boolean;
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
      >Waypoints: {{ waypointList.past }} <b> {{ waypointList.present }} </b>
      {{ waypointList.future }}</span
    >
    <input id="showWaypoints" v-model="showWaypoints" type="checkbox" />
    <label for="showWaypoints">Show Waypoints</label>
  </small>
  <br />
  <small>ID: {{ planeState.info?.id }}</small>
</template>
