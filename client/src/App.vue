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
}

#map {
  width: 80vw;
}

@media only screen and (max-width: 600px) {
  #aside {
    width: 0vw;
    transition: width 1s;
  }

  #aside.visible {
    width: 80vw;
  }

  #map {
    width: 100vw;
  }
}
</style>
