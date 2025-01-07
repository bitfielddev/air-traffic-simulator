<script setup lang="ts">
import { formatDuration } from "@/util.ts";
import type { PlaneState } from "@/plane.ts";
import { computed, onUnmounted, ref, watchEffect } from "vue";
import { getWorldData } from "@/staticData.ts";
import { airportCoords } from "@/airport.ts";

const { planeState } = defineProps<{ planeState: PlaneState }>();

const fromCoords = ref<[number, number]>([0, 0]);
const toCoords = ref<[number, number]>([0, 0]);
watchEffect(async () => {
  const wd = await getWorldData();
  fromCoords.value = airportCoords(
    wd.airports.find((a) => a.code === planeState.info?.flight.from)!,
  );
  toCoords.value = airportCoords(
    wd.airports.find((a) => a.code === planeState.info?.flight.to)!,
  );
});

const startTime = computed(
  () => new Date(Number(planeState.info?.start_time) * 1000),
);
const totalDuration = computed(() => {
  let waypoints = planeState.info?.pos.planner.route.map((a) => a.pos)!;
  waypoints.unshift(fromCoords.value);
  waypoints.push(toCoords.value);

  let distance = 0;
  for (let i = 0; i < waypoints.length - 1; i++) {
    let [x1, z1] = waypoints[i];
    let [x2, z2] = waypoints[i + 1];
    distance += Math.sqrt(Math.pow(x1 - x2, 2) + Math.pow(z1 - z2, 2));
  }

  return distance / planeState.info?.model.motion.max_v[0]!;
});

const currentDuration = ref(0);
const remainingDuration = ref(0);
async function updateDuration() {
  currentDuration.value =
    new Date().valueOf() / 1000 - Number(planeState.info?.start_time);

  const wd = await getWorldData();
  let [x1, , z1] = planeState.s;
  let [x2, z2] = airportCoords(
    wd.airports.find((a) => a.code === planeState.info?.flight.to)!,
  );
  let method1 =
    Math.sqrt(Math.pow(x1 - x2, 2) + Math.pow(z1 - z2, 2)) / planeState.v[0];

  let method2 = totalDuration.value - currentDuration.value;
  remainingDuration.value = Math.max(method1, method2);
}
updateDuration();
const durationUpdater = setInterval(updateDuration, 1000);

onUnmounted(() => {
  clearInterval(durationUpdater);
});
</script>

<template>
  <b>Departed:</b> {{ startTime.toLocaleTimeString() }} ({{
    formatDuration(currentDuration)
  }}
  ago) <br />
  <b>Est. Remaining:</b> {{ formatDuration(remainingDuration) }}
</template>
