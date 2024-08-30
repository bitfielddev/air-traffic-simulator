<script setup lang="ts">
import * as airport from "@/airport";

defineProps<{ airportId: string }>();
</script>

<template>
  <button
    @click="airport.airportMarkers.get(airportId)?.marker.fireEvent('click')"
  >
    <slot
      ><span>{{ airportId }}</span></slot
    >
  </button>
</template>

<style scoped>
button {
  all: unset;
}
span:hover {
  text-decoration: underline;
}
</style>
