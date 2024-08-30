<script setup lang="ts">
import Map from "./components/Map.vue";
import Sidebar from "./components/Sidebar.vue";
</script>

<template>
  <Sidebar class="sect" />
  <Map class="sect" />
</template>

<style scoped>
.sect {
  float: left;
  height: 100vh;
  display: block;
}

#aside {
  width: 20vw;
  background-color: lightgray;
}

#map {
  width: 80vw;
}

@media only screen and (max-width: 600px) {
  #aside {
    width: 100vw;
    z-index: 2000;
    position: absolute;
    transition: transform 0.5s ease-in-out;
    transform: translateX(-100vw);
  }

  #aside.visible {
    transform: translateX(0vw);
  }

  #map {
    width: 100vw;
  }
}
</style>
