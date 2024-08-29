import { reactive, ref, type Ref } from "vue";
import type { Plane } from "./bindings/Plane";
import socket from "./socket";
import { escape } from "./util";
import { stringify as uuidStringify } from "uuid";
import * as map from "@/map";
import "Leaflet.MultiOptionsPolyline";

export interface SelectedPlane {
  id: string;
  path: L.MultiOptionsPolyline;
}

interface PlaneState {
  s: [number, number, number];
  angle: number;
  v: [number, number];
  marker: L.CircleMarker;
  info?: Plane;
}

export const markers = reactive(new Map<string, PlaneState>());
export const selected = ref<SelectedPlane>();

export function updateSelect(latLng: L.LatLng) {
  selected.value?.path.setLatLngs(
    selected.value?.path.getLatLngs().concat([latLng]),
  );
}
export function deselect() {
  selected.value?.path.remove();
  selected.value = undefined;
}
export async function select(
  id: string,
  e: L.PopupEvent,
  map: Ref<L.Map | undefined>,
) {
  deselect();
  const plane: Plane = await socket.value
    .timeout(5000)
    .emitWithAck("plane", id);
  e.popup.setContent(
    `${escape(plane.flight.code)}: ${escape(plane.flight.from)} → ${escape(plane.flight.to)}`,
  );

  const state = markers.get(id)!;
  state.info = plane;

  selected.value = {
    path: L.multiOptionsPolyline(
      plane.pos.planner.past_pos.map((a) => L.latLng(...a)),
      {
        multiOptions: {
          optionIdxFn: (latLng) => {
            const altThresholds = [10, 20, 30, 40, 50, 60, 70, 80, 90];

            for (let i = 0; i < altThresholds.length; ++i) {
              if (latLng.alt <= altThresholds[i]) {
                return i;
              }
            }
            return altThresholds.length;
          },
          options: [
            { color: "#aaaaaa" },
            { color: "#aaaa00" },
            { color: "#00aa00" },
            { color: "#00aaaa" },
            { color: "#0000aa" },
            { color: "#aa00aa" },
            { color: "#aa0000" },
            { color: "#000000" },
            { color: "#000000" },
          ],
        },
      },
    ).addTo(map.value!),
    id,
  };
}

export function handleStateUpdates() {
  socket.value.on("state", (removed, bin) => {
    for (const remove of removed) {
      markers.get(remove)?.marker.remove();
      markers.delete(remove);
    }
    for (let off = 0; off < bin.byteLength; off += 40) {
      const id = uuidStringify(new Uint8Array(bin.slice(off, off + 16)));
      const view = new DataView(bin);
      const sx = view.getFloat32(off + 16, true);
      const sy = view.getFloat32(off + 20, true);
      const sz = view.getFloat32(off + 24, true);
      const angle = view.getFloat32(off + 28, true);
      const vx = view.getFloat32(off + 32, true);
      const vy = view.getFloat32(off + 36, true);

      const state = markers.get(id);
      if (state === undefined) {
        markers.set(id, {
          s: [sx, sy, sz],
          angle,
          v: [vx, vy],
          marker: L.circleMarker([sx, sy], { radius: 5 })
            .on("popupopen", (e) => select(id, e, map.map))
            .on("popupclose", () => deselect())
            .bindPopup("Loading...")
            .addTo(map.map.value!),
        });
      } else {
        state.s = [sx, sy, sz];
        state.angle = angle;
        state.v = [vx, vy];
        state.marker.setLatLng([sx, sy]);
      }

      if (selected.value?.id === id) {
        updateSelect(L.latLng(sx, sy, sz));
      }
    }
  });
}

export function updatePositions(dt: number) {
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
