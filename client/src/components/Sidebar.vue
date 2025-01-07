<script setup lang="ts">
import * as airport from "@/airport";
import * as map from "@/map";
import * as plane from "@/plane";
import "@fortawesome/fontawesome-free/css/all.min.css";
import { faXmark } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/vue-fontawesome";
import * as L from "leaflet";
import "leaflet-easybutton";
import "leaflet-easybutton/src/easy-button.css";
import { computed, ref, watch } from "vue";
import AirportInfo from "./AirportInfo.vue";
import PlaneInfo from "./PlaneInfo.vue";
import { rawMap } from "@/map";
import Statistics from "@/components/Statistics.vue";
import WaypointNetwork from "@/components/WaypointNetwork.vue";

const planeState = computed(() =>
  plane.selectedPlane.value === undefined
    ? undefined
    : (plane.planeStates.get(plane.selectedPlane.value.id) as
        | plane.PlaneState
        | undefined),
);
const airportState = computed(() =>
  airport.selectedAirport.value === undefined
    ? undefined
    : (airport.airportStates.get(airport.selectedAirport.value) as
        | airport.AirportState
        | undefined),
);

const showInMobile = ref(false);
const sidebarButton = ref<L.Control.EasyButton>();

watch(map.map, () => {
  if (map.map.value === undefined || sidebarButton.value !== undefined) return;
  sidebarButton.value = L.easyButton("fa-bars", () => {
    showInMobile.value = true;
  }).addTo(rawMap());
  toggleSidebarButtonDisplay();
});

function toggleSidebarButtonDisplay() {
  const style = sidebarButton.value?.getContainer()?.style;
  if (style === undefined) return;
  style.display = window.matchMedia("(max-width: 600px)").matches
    ? "unset"
    : "none";
}

window.addEventListener("resize", toggleSidebarButtonDisplay);
</script>

<template>
  <aside id="aside" :class="{ visible: showInMobile }">
    <button id="close" @click="showInMobile = false">
      <FontAwesomeIcon :icon="faXmark" />
    </button>
    <div v-if="planeState !== undefined">
      <PlaneInfo :plane-state />
    </div>
    <div v-else-if="airportState !== undefined">
      <AirportInfo :airport-state />
    </div>
    <div v-else>
      Select an airport, runway or flight...
      <hr />
      <Statistics />
      <hr />
      <WaypointNetwork />
    </div>
  </aside>
</template>

<style scoped>
aside div {
  padding: 1em;
  overflow-x: clip;
}

#close {
  position: absolute;
  right: 1em;
  top: 1em;
  aspect-ratio: 1/1;
  font-size: 1em;
  border: none;
  background-color: unset;
}
</style>
