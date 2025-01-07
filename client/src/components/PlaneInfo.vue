<script setup lang="ts">
import "leaflet-easybutton/src/easy-button.css";
import "leaflet-easybutton";
import AirportLink from "./AirportLink.vue";
import Waypoints from "@/components/planeState/Waypoints.vue";
import Duration from "@/components/planeState/Duration.vue";
import type { PlaneState } from "@/plane.ts";

const { planeState } = defineProps<{ planeState: PlaneState }>();
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
    <Duration :plane-state="planeState" /> <br /><br />
  </div>
  <Waypoints :plane-state="planeState" />
  <br />
  <small>ID: {{ planeState.info?.id }}</small>
</template>
