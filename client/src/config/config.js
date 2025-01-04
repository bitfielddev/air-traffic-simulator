import * as L from "leaflet";

export const tileLayer = L.tileLayer(
  "https://tile.openstreetmap.org/{z}/{x}/{y}.png",
  {
    maxZoom: 19,
    attribution:
      '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>',
  },
);

export const altitudeColours = [
  [10, "#aaaaaa"],
  [20, "#aaaa00"],
  [30, "#00aa00"],
  [40, "#00aaaa"],
  [50, "#0000aa"],
  [60, "#aa00aa"],
  [70, "#aa0000"],
  [80, "#000000"],
  [90, "#000000"],
];

export function world2map([x, y]) {
  return [y, x];
}

export function world2map3([x, y, z]) {
  return [y, x, z];
}
