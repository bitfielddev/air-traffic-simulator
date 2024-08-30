<script setup lang="ts">
import type { Airport } from "@/bindings/Airport";
import type { Plane } from "@/bindings/Plane";
import socket from "@/socket";
import { onMounted, ref } from "vue";
import * as plane from "@/plane";

const { airportState } = defineProps<{ airportState: Airport }>();
const departurePlanes = ref<Plane[]>();
const arrivalPlanes = ref<Plane[]>();

onMounted(async () => {
  const departurePlaneIds: string[] = await socket.value
    .timeout(5000)
    .emitWithAck("airport_departures", airportState.airport.code);
  console.log(departurePlaneIds);
  departurePlanes.value = await Promise.all(
    departurePlaneIds.map(plane.getPlaneInfo),
  );
  const arrivalPlaneIds: string[] = await socket.value
    .timeout(5000)
    .emitWithAck("airport_arrivals", airportState.airport.code);
  arrivalPlanes.value = await Promise.all(
    arrivalPlaneIds.map(plane.getPlaneInfo),
  );
});
</script>
<template>
  <div style="text-align: center">
    <b style="font-size: 3em">{{ airportState.airport.code }}</b
    ><br />{{ airportState.airport.name }}<br /><br />
    <b>Runways</b>
    <table border="0">
      <tr>
        <th>Name</th>
        <th>Start</th>
        <th>End</th>
        <th>Altitude</th>
        <th>Class</th>
      </tr>
      <tr v-for="runway in airportState.airport.runways" :key="runway.name">
        <td>
          <b>{{ runway.name }}</b>
        </td>
        <td>{{ runway.start[0] }} {{ runway.start[1] }}</td>
        <td>{{ runway.end[0] }} {{ runway.end[1] }}</td>
        <td>{{ runway.altitude }}</td>
        <td>{{ runway.class }}</td>
      </tr>
    </table>
    <template v-if="departurePlanes !== undefined">
      <b>Departures:</b><br />
      <table border="0">
        <tr>
          <td>Airline</td>
          <td>Flight</td>
          <td>To</td>
        </tr>
        <tr v-for="planeInfo in departurePlanes" :key="planeInfo.id">
          <td>{{ planeInfo.flight.airline }}</td>
          <td>{{ planeInfo.flight.code }}</td>
          <td>{{ planeInfo.flight.to }}</td>
          <td>
            <button
              @click="
                plane.markers.get(planeInfo.id)?.marker.fireEvent('click')
              "
            >
              Select
            </button>
          </td>
        </tr>
      </table>
    </template>
    <span v-else>Loading departures...</span>
    <template v-if="arrivalPlanes !== undefined">
      <b>Arrivals:</b><br />
      <table border="0">
        <tr>
          <td>Airline</td>
          <td>Flight</td>
          <td>To</td>
        </tr>
        <tr v-for="planeInfo in arrivalPlanes" :key="planeInfo.id">
          <td>{{ planeInfo.flight.airline }}</td>
          <td>{{ planeInfo.flight.code }}</td>
          <td>{{ planeInfo.flight.to }}</td>
          <td>
            <button
              @click="
                plane.markers.get(planeInfo.id)?.marker.fireEvent('click')
              "
            >
              Select
            </button>
          </td>
        </tr>
      </table>
    </template>
    <span v-else>Loading arrivals...</span>
  </div>
</template>
