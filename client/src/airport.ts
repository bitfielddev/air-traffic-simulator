import { reactive, ref } from "vue";
import type { Airport } from "./bindings/Airport";
import type { WorldData } from "./bindings/WorldData";
import * as map from "./map";
import socket from "./socket";
import { escape } from "./util";

export const airportMarkers = reactive(new Map<string, Airport>());
export const selectedAirport = ref<string>();

export function deselectAirport() {
  selectedAirport.value = undefined;
}
export async function selectAirport(id: string) {
  deselectAirport();
  const airport: Airport = await socket.value
    .timeout(5000)
    .emitWithAck("airport", id);

  airportMarkers.set(id, airport);
  selectedAirport.value = id;
}

export async function drawAirports() {
  const wd: WorldData = await socket.value
    .timeout(5000)
    .emitWithAck("world_data");
  for (const airport of wd.airports) {
    for (const runway of airport.runways) {
      L.polyline([runway.start, runway.end], {
        color: "red",
        weight: 10,
        opacity: 0.5,
      })
        .bindPopup(
          `${escape(airport.name)} (${escape(airport.code)})<br>Altitude: ${runway.altitude}`,
        )
        .on("popupopen", () => selectAirport(airport.code))
        .on("popupclose", () => deselectAirport())
        .addTo(map.map.value!);
    }

    const centre = airport.runways
      .flatMap((a) => [a.start, a.end])
      .reduce(
        ([px, py], [x, y]) => [
          px + x / (2 * airport.runways.length),
          py + y / (2 * airport.runways.length),
        ],
        [0, 0],
      );
    L.circleMarker(centre, { radius: 10, color: "red" })
      .bindPopup(`${escape(airport.name)} (${airport.code})`)
      .on("popupopen", () => selectAirport(airport.code))
      .on("popupclose", () => deselectAirport())
      .addTo(map.map.value!);
  }
}
