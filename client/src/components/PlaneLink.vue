<script setup lang="ts">
import * as plane from "@/plane";

defineProps<{ planeId: string }>();
</script>

<template>
  <button @click="plane.planeMarkers.get(planeId)?.marker.fireEvent('click')">
    <slot />
  </button>
</template>

<style scoped>
button {
  all: unset;
}
</style>
