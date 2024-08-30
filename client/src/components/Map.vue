<script setup lang="ts">
import * as airport from "@/airport";
import * as map from "@/map";
import * as plane from "@/plane";
import "leaflet/dist/leaflet.css";
import { onMounted } from "vue";

onMounted(() => {
  map.initMap();
  airport.drawAirports();
  plane.handleStateUpdates();
  plane.updatePositions(0);
});
</script>

<template>
  <div id="map"></div>
</template>
