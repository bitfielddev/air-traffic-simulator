import { reactive, ref } from "vue";
import type { Airport } from "./bindings/Airport";
import * as map from "./map";
import socket from "./socket";
import { escape } from "./util";
import { getWorldData } from "./staticData";
import config from "./config";
import type { AirportData } from "@/bindings/AirportData.ts";

export interface AirportState {
  info?: Airport;
  marker: L.CircleMarker;
}

export const airportMarkers = reactive(new Map<string, AirportState>());
export const selectedAirport = ref<string>();

export function airportCoords(airport: AirportData): [number, number] {
  return airport.runways
    .flatMap((a) => [a.start, a.end])
    .reduce(
      ([px, py], [x, y]) => [
        px + x / (2 * airport.runways.length),
        py + y / (2 * airport.runways.length),
      ],
      [0, 0],
    );
}

export function deselectAirport() {
  selectedAirport.value = undefined;
}
export async function selectAirport(id: string) {
  deselectAirport();
  await getAirportInfo(id, true);
  selectedAirport.value = id;
}

export async function getAirportInfo(id: string, force?: boolean) {
  const cache = airportMarkers.get(id)?.info;
  if (cache !== undefined && !force) return cache as Airport;
  const info: Airport = await socket.value
    .timeout(5000)
    .emitWithAck("airport", id);
  const state = airportMarkers.get(id)!;
  if (state !== undefined) state.info = info;
  return info;
}

export async function drawAirports() {
  const wd = await getWorldData();
  for (const airport of wd.airports) {
    for (const runway of airport.runways) {
      L.polyline(
        [config.world2map(runway.start), config.world2map(runway.end)],
        {
          color: "red",
          weight: 10,
          opacity: 0.5,
        },
      )
        .bindPopup(
          `${escape(airport.name)} (${escape(airport.code)})<br>Runway ${escape(runway.name)}<br>Altitude: ${escape(runway.altitude.toString())}`,
        )
        .on("popupopen", () => selectAirport(airport.code))
        .on("popupclose", () => deselectAirport())
        .addTo(map.map.value!);
    }

    const centre = airportCoords(airport);
    const marker = L.circleMarker(config.world2map(centre), {
      radius: 10,
      color: "red",
    })
      .bindPopup(`${escape(airport.name)} (${airport.code})`)
      .on("popupopen", () => selectAirport(airport.code))
      .on("popupclose", () => deselectAirport())
      .addTo(map.map.value!);
    airportMarkers.set(airport.code, { marker });
  }
}
