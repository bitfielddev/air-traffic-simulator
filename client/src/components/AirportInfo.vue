<script setup lang="ts">
import type { Plane } from "@/bindings/Plane";
import * as plane from "@/plane";
import socket from "@/socket";
import { computed, ref, watch } from "vue";
import PlaneLink from "./PlaneLink.vue";
import AirportLink from "./AirportLink.vue";
import type { AirportState } from "@/airport";

const { airportState } = defineProps<{ airportState: AirportState }>();
const airport = computed(() => airportState.info?.airport);
const departurePlanes = ref<Plane[]>();
const arrivalPlanes = ref<Plane[]>();

watch(
  airport,
  async () => {
    if (airport.value === undefined) return;

    const departurePlaneIds: string[] = await socket.value
      .timeout(5000)
      .emitWithAck("airport_departures", airport.value.code);
    departurePlanes.value = await Promise.all(
      departurePlaneIds.map((a) => plane.getPlaneInfo(a)),
    );
    const arrivalPlaneIds: string[] = await socket.value
      .timeout(5000)
      .emitWithAck("airport_arrivals", airport.value.code);
    arrivalPlanes.value = await Promise.all(
      arrivalPlaneIds.map((a) => plane.getPlaneInfo(a)),
    );
  },
  { immediate: true },
);
</script>
<template>
  <template v-if="airport !== undefined">
    <div style="text-align: center">
      <b style="font-size: 3em">{{ airport.code }}</b
      ><br />{{ airport.name }}<br /><br />
      <b>Runways</b>
      <table border="0">
        <thead>
          <tr>
            <th>Name</th>
            <th>Start</th>
            <th>End</th>
            <th>Alt</th>
            <th>Class</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="runway in airport.runways" :key="runway.name">
            <td>
              <b>{{ runway.name }}</b>
            </td>
            <td>{{ runway.start[0] }} {{ runway.start[1] }}</td>
            <td>{{ runway.end[0] }} {{ runway.end[1] }}</td>
            <td>{{ runway.altitude }}</td>
            <td>{{ runway.class }}</td>
          </tr>
        </tbody>
      </table>
      <template v-if="departurePlanes !== undefined">
        <b>Departures:</b><br />
        <table border="0">
          <thead>
            <tr>
              <th>Airline</th>
              <th>Flight</th>
              <th>To</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="planeInfo in departurePlanes" :key="planeInfo.id">
              <td>{{ planeInfo.flight.airline }}</td>
              <td>{{ planeInfo.flight.code }}</td>
              <td><AirportLink :airport-id="planeInfo.flight.to" /></td>
              <td>
                <PlaneLink :plane-id="planeInfo.id"
                  ><button>Select</button></PlaneLink
                >
              </td>
            </tr>
          </tbody>
        </table>
      </template>
      <span v-else>Loading departures...</span>
      <template v-if="arrivalPlanes !== undefined">
        <b>Arrivals:</b><br />
        <table border="0">
          <thead>
            <tr>
              <th>Airline</th>
              <th>Flight</th>
              <th>From</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="planeInfo in arrivalPlanes" :key="planeInfo.id">
              <td>{{ planeInfo.flight.airline }}</td>
              <td>{{ planeInfo.flight.code }}</td>
              <td><AirportLink :airport-id="planeInfo.flight.from" /></td>
              <td>
                <PlaneLink :plane-id="planeInfo.id"
                  ><button>Select</button></PlaneLink
                >
              </td>
            </tr>
          </tbody>
        </table>
      </template>
      <span v-else>Loading arrivals...</span>
    </div>
  </template>
  <template v-else> Loading... </template>
</template>
