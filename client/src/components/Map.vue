<script setup lang="ts">
import "leaflet/dist/leaflet.css";
import { onMounted, onUnmounted } from "vue";
import socket from "@/socket";
import * as plane from "@/plane";
import * as map from "@/map";
import * as airport from "@/airport";

onMounted(() => {
  map.initMap();
  airport.drawRunways();
  plane.handleStateUpdates();
  plane.updatePositions(0);
});

onUnmounted(() => {
  socket.value.close();
});
</script>

<template>
  <div id="map"></div>
</template>
