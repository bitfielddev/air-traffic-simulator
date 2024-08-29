import { ref } from "vue";

export const map = ref<L.Map>();

export function initMap() {
  map.value = L.map("map", {
    preferCanvas: true,
    crs: L.CRS.Simple,
  }).setView([0, 0], 0);

  L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
    maxZoom: 19,
    attribution:
      '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>',
  }).addTo(map.value);
}
