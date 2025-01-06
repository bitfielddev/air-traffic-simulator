import { shallowRef, toRaw } from "vue";
import config from "./config";

export const map = shallowRef<L.Map>();

export function rawMap(): L.Map {
  return toRaw(map.value!);
}

export function initMap() {
  map.value = L.map("map", {
    preferCanvas: true,
    crs: L.CRS.Simple,
  }).setView([0, 0], 0);

  config.tileLayer.addTo(rawMap());
}
