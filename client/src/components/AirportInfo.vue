<script setup lang="ts">
import type { Airport } from "@/bindings/Airport";

const { airportState } = defineProps<{ airportState: Airport }>();
</script>
<template>
  <div style="text-align: center">
    <b style="font-size: 3em">{{ airportState.airport.code }}</b
    ><br />{{ airportState.airport.name }}<br />
    <!-- <b>Departures:</b><br />
    {departures}
    <b>Arrivals:</b><br />
    {arrivals} -->
  </div>
</template>
