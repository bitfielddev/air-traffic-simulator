<script setup lang="ts">
import { getWorldData } from "@/staticData.ts";
import { computed, onMounted } from "vue";
import { planeStates } from "@/plane.ts";

let num_airports = 0;
let num_runways = 0;
onMounted(async () => {
  const wd = await getWorldData();
  num_airports = wd.airports.filter((a) => a.runways.length).length;
  num_runways = wd.airports
    .map((a) => a.runways.length)
    .reduce((a, b) => a + b);
});
const num_planes = computed(() => planeStates.size);
</script>

<template>
  <b>Airports:</b> {{ num_airports }}<br />
  <b>Runways:</b> {{ num_runways }}<br />
  <b>Planes:</b> {{ num_planes }}<br />
</template>

<style scoped></style>
