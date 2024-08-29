<script setup lang="ts">
import * as plane from "@/plane";
import * as airport from "@/airport";
import { computed } from "vue";
const planeState = computed(() =>
  plane.selected.value === undefined
    ? undefined
    : plane.markers.get(plane.selected.value.id),
);
const waypointList = computed(() => {
  const past =
    planeState.value?.info?.pos.planner.past_route
      .map((a) => a.name)
      .join(", ") ?? "";
  const present = planeState.value?.info?.pos.planner.route.at(0)?.name ?? "";
  const future =
    planeState.value?.info?.pos.planner.route.map((a) => a.name).join(", ") ??
    "";
  return { past, present, future };
});

let showWaypoints: boolean;

const airportState = computed(() =>
  airport.selectedAirport.value === undefined
    ? undefined
    : airport.airportMarkers.get(airport.selectedAirport.value),
);
</script>

<template>
  <aside id="aside">
    <div v-if="planeState !== undefined">
      <div style="text-align: center">
        <b style="font-size: 3em"
          >{{ planeState.info?.flight.from }} →
          {{ planeState.info?.flight.to }}</b
        ><br />
        operated by <b>{{ planeState.info?.flight.airline }}</b
        ><br />
        on a(n) <b>{{ planeState.info?.model.name }}</b
        ><br />
        <!--<b>{departTime}</b> --(<i>{duration}</i>)→ <b>{arrivalTime}</b><br />
      {distance} blocks-->
        <b>Coords:</b> {{ Math.round(planeState.s[0]) }}
        {{ Math.round(planeState.s[1]) }} <b>Alt:</b>
        {{ Math.round(planeState.s[2]) }} <br /><br />
      </div>
      <small>
        <span
          >Waypoints: {{ waypointList.past }}
          <b> {{ waypointList.present }} </b> {{ waypointList.future }}</span
        >
        <input id="showWaypoints" v-model="showWaypoints" type="checkbox" />
        <label for="showWaypoints">Show Waypoints</label>
      </small>
      <br />
      <small>ID: {{ planeState.info?.id }}</small>
    </div>
    <div v-else-if="airportState !== undefined">
      <div style="text-align: center">
        <b style="font-size: 3em">{{ airportState.airport.code }}</b
        ><br />{{ airportState.airport.name }}<br />
        <!-- <b>Departures:</b><br />
      {departures}
      <b>Arrivals:</b><br />
      {arrivals} -->
      </div>
    </div>
    <div v-else>Select an airport, runway or flight...</div>
  </aside>
</template>

<style scoped>
aside div {
  padding: 1em;
}
</style>
