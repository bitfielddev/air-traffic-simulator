import { ref } from "vue";
import config from "./config";

export const map = ref<L.Map>();

export function initMap() {
  map.value = L.map("map", {
    preferCanvas: true,
    crs: L.CRS.Simple,
  }).setView([0, 0], 0);

  config.tileLayer.addTo(map.value);
}
