<script setup lang="ts">
import "leaflet/dist/leaflet.css";
import * as L from "leaflet";
import { onMounted, onUnmounted, ref } from "vue";
import { stringify as uuidStringify } from "uuid";
import type { WorldData } from "@/bindings/WorldData";
import socket from "@/socket";
import type { Plane } from "@/bindings/Plane";

interface PlaneState {
  s: [number, number];
  angle: number;
  v: [number, number];
  marker: L.CircleMarker;
}

interface SelectedPlane {
  path: L.Polyline;
  id: string;
}

const map = ref<L.Map>();
const markers = new Map<string, PlaneState>();
let selected: SelectedPlane | undefined;

async function drawRunways() {
  let wd: WorldData = await socket.value
    .timeout(5000)
    .emitWithAck("world_data");
  for (let airport of wd.airports) {
    for (let runway of airport.runways) {
      console.log(runway);
      L.polyline([runway.start, runway.end], {
        color: "red",
        weight: 10,
        opacity: 0.5,
      })
        .bindPopup(
          `${airport.name} (${airport.code})<br>Altitude: ${runway.altitude}`,
        )
        .addTo(map.value!);
    }
  }
}

function handleStateUpdates() {
  socket.value.on("state", (removed, bin) => {
    for (let remove of removed) {
      markers.get(remove)?.marker.remove();
      markers.delete(remove);
    }
    for (let off = 0; off < bin.byteLength; off += 36) {
      const id = uuidStringify(new Uint8Array(bin.slice(off, off + 16)));
      const view = new DataView(bin);
      const sx = view.getFloat32(off + 16, true);
      const sy = view.getFloat32(off + 20, true);
      const angle = view.getFloat32(off + 24, true);
      const vx = view.getFloat32(off + 28, true);
      const vy = view.getFloat32(off + 32, true);

      let state = markers.get(id);
      if (state === undefined) {
        markers.set(id, {
          s: [sx, sy],
          angle,
          v: [vx, vy],
          marker: L.circleMarker([sx, sy], { radius: 5 })
            .on("popupopen", (e) => select(id, e))
            .on("popupclose", () => deselect())
            .bindPopup(id)
            .addTo(map.value!),
        });
      } else {
        state.s = [sx, sy];
        state.angle = angle;
        state.v = [vx, vy];
        state.marker.setLatLng([sx, sy]);
      }

      if (selected?.id === id) {
        selected.path.addLatLng([sx, sy]);
      }
    }
  });
}

async function select(id: string, e: L.PopupEvent) {
  selected?.path.remove();
  const plane: Plane = await socket.value
    .timeout(5000)
    .emitWithAck("plane", id);
  const escape = (text: string) =>
    text
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll("'", "&#39;")
      .replaceAll('"', "&quot;");
  e.popup.setContent(
    `${escape(plane.flight.code)}: ${escape(plane.flight.from)} -> ${escape(plane.flight.to)}`,
  );
  selected = {
    path: L.polyline(plane.pos.planner.past_pos.map((a) => [a[0], a[1]])).addTo(
      map.value!,
    ),
    id,
  };
}

function deselect() {
  selected?.path.remove();
  selected = undefined;
}

function updatePositions(dt: number) {
  const start = Date.now();
  for (const state of markers.values()) {
    state.s[0] += state.v[0] * Math.cos(state.angle) * (dt / 1000);
    state.s[1] += state.v[0] * Math.sin(state.angle) * (dt / 1000);
    state.marker.setLatLng(state.s);
  }
  setTimeout(() => {
    updatePositions(Date.now() - start);
  }, 0);
}

onMounted(() => {
  map.value = L.map("map", {
    preferCanvas: true,
    crs: L.CRS.Simple,
  }).setView([0, 0], 0);

  L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
    maxZoom: 19,
    attribution:
      '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>',
  }).addTo(map.value);

  drawRunways();
  handleStateUpdates();
  updatePositions(0);
});

onUnmounted(() => {
  socket.value.close();
});
</script>

<template>
  <div id="map" style="width: 100vw; height: 100vh"></div>
</template>
