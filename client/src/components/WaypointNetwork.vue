<script lang="ts">
import { ref } from "vue";
import * as L from "leaflet";
import { getWorldData } from "@/staticData.ts";
import config from "@/config";
let showWaypointNetwork = ref(false);

let waypointNetworkFeatureGroup = L.featureGroup();
(async () => {
  const wd = await getWorldData();
  const existing: string[] = [];
  for (let u of wd.waypoints) {
    for (let vn of u.connections) {
      const [key1, key2] = [u.name + "-" + vn, vn + "-" + u.name];
      if (existing.includes(key1)) continue;
      existing.push(key1, key2);
      const v = wd.waypoints.find((v) => v.name === vn)!;
      L.polyline([config.world2map(u.pos), config.world2map(v.pos)], {
        color: "red",
        opacity: 0.5,
        interactive: false,
      }).addTo(waypointNetworkFeatureGroup);
    }
  }
})();
</script>
<script setup lang="ts">
import { watchEffect } from "vue";
import { rawMap } from "@/map.ts";

watchEffect(async () => {
  if (showWaypointNetwork.value) {
    waypointNetworkFeatureGroup.addTo(rawMap());
  } else {
    waypointNetworkFeatureGroup.remove();
  }
});
</script>

<template>
  <input
    id="showWaypointNetwork"
    v-model="showWaypointNetwork"
    type="checkbox"
  />
  <label for="showWaypointNetwork">Show Waypoint Network</label>
</template>
