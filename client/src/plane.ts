import { reactive, ref, type Ref } from "vue";
import type { Plane } from "./bindings/Plane";
import socket from "./socket";
import { escape } from "./util";
import { stringify as uuidStringify } from "uuid";
import * as map from "@/map";
import "Leaflet.MultiOptionsPolyline";
import config from "@/config";

export interface SelectedPlane {
  id: string;
  path: L.MultiOptionsPolyline;
}

export interface PlaneState {
  s: [number, number, number];
  angle: number;
  v: [number, number];
  marker: L.CircleMarker;
  info?: Plane;
}

export const markers = reactive(new Map<string, PlaneState>());
export const selected = ref<SelectedPlane>();

export function updateSelectPlane(latLng: L.LatLng) {
  selected.value?.path.setLatLngs(
    selected.value?.path.getLatLngs().concat([latLng]),
  );
}
export function deselectPlane() {
  selected.value?.path.remove();
  selected.value = undefined;
}

export async function getPlaneInfo(id: string, force?: boolean) {
  const cache = markers.get(id)?.info;
  if (cache !== undefined && !force) return cache as Plane;
  const info: Plane = await socket.value.timeout(5000).emitWithAck("plane", id);
  const state = markers.get(id)!;
  if (state !== undefined) state.info = info;
  return info;
}

export async function selectPlane(
  id: string,
  e: L.PopupEvent,
  map: Ref<L.Map | undefined>,
) {
  deselectPlane();
  const plane = await getPlaneInfo(id, true);
  e.popup.setContent(
    `${escape(plane.flight.code)}: ${escape(plane.flight.from)} → ${escape(plane.flight.to)}`,
  );

  selected.value = {
    path: L.multiOptionsPolyline(
      plane.pos.planner.past_pos.map((a) => L.latLng(...a)),
      {
        multiOptions: {
          optionIdxFn: (latLng) => {
            const altThresholds = config.altitudeColours.map((a) => a[0]);

            for (let i = 0; i < altThresholds.length; ++i) {
              if (latLng.alt <= altThresholds[i]) {
                return i;
              }
            }
            return altThresholds.length;
          },
          options: config.altitudeColours.map((a) => ({ color: a[1] })),
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
            .on("popupopen", (e) => selectPlane(id, e, map.map))
            .on("popupclose", () => deselectPlane())
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
        updateSelectPlane(L.latLng(sx, sy, sz));
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
