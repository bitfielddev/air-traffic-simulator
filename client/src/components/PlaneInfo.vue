<script setup lang="ts">
import "leaflet-easybutton/src/easy-button.css";
import "leaflet-easybutton";
import AirportLink from "./AirportLink.vue";
import Waypoints from "@/components/planeState/Waypoints.vue";
import Duration from "@/components/planeState/Duration.vue";
import { getPlaneInfo, type PlaneState } from "@/plane.ts";
import { onUnmounted } from "vue";

const { planeState } = defineProps<{ planeState: PlaneState }>();

const planeInfoUpdater = setInterval(() => {
  if (planeState.info === undefined) return;
  getPlaneInfo(planeState.info.id, true);
}, 5000);

onUnmounted(() => {
  clearInterval(planeInfoUpdater);
});
</script>
<template>
  <template v-if="planeState.info !== undefined">
    <div style="text-align: center">
      <b style="font-size: 2.5em"
        ><AirportLink :airport-id="planeState.info.flight.from" /> →
        <AirportLink :airport-id="planeState.info.flight.to" /></b
      ><br />
      <b>{{ planeState.info.flight.code }}</b>
      operated by <b>{{ planeState.info.flight.airline }}</b
      ><br />
      on a(n) <b>{{ planeState.info.model.name }}</b
      ><br />
      <b>Coords:</b> {{ Math.round(planeState.s[0]) }}
      {{ Math.round(planeState.s[1]) }} <b>Alt:</b>
      {{ Math.round(planeState.s[2]) }} <br />
      <b>Velocity:</b> {{ Math.round(planeState.v[0]) }}
      {{ Math.round(planeState.v[1]) }} <br />
      <Duration :plane-state /> <br /><br />
    </div>
    <Waypoints :plane-state />
    <br />
    <small>ID: {{ planeState.info.id }}</small>
  </template>
  <template v-else> Loading... </template>
</template>
