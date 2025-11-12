import config from "@/config";
import "Leaflet.MultiOptionsPolyline";
import { stringify as uuidStringify } from "uuid";
import { markRaw, reactive, ref } from "vue";
import type { Plane } from "./bindings/Plane";
import socket from "./socket";
import { escape } from "./util";
import { rawMap } from "@/map";
import { airportCoords, getAirportInfo } from "@/airport.ts";

export interface SelectedPlane {
  id: string;
  path: L.MultiOptionsPolyline;
  plane2dest: L.Polyline;
}

export interface PlaneState {
  s: [number, number, number];
  angle: number;
  v: [number, number];
  marker: L.CircleMarker;
  info?: Plane;
}

export const planeStates = reactive(new Map<string, PlaneState>());
export const selectedPlane = ref<SelectedPlane>();

export function updateSelectPlane(latLng: L.LatLng) {
  selectedPlane.value?.path?.setLatLngs(
    selectedPlane.value?.path?.getLatLngs().concat([latLng]),
  );

  selectedPlane.value?.plane2dest.setLatLngs(
    selectedPlane.value?.plane2dest?.getLatLngs().with(1, latLng as never),
  );
}
export function deselectPlane() {
  selectedPlane.value?.path?.remove();
  selectedPlane.value?.plane2dest?.remove();
  selectedPlane.value = undefined;
}

export async function getPlaneInfo(id: string, force?: boolean) {
  const cache = planeStates.get(id)?.info;
  if (cache !== undefined && !force) return cache as Plane;
  const info: Plane = await socket.value.timeout(5000).emitWithAck("plane", id);
  const state = planeStates.get(id)!;
  if (state !== undefined) state.info = info;
  return info;
}

export async function selectPlane(id: string, e: L.PopupEvent) {
  deselectPlane();

  const plane = await getPlaneInfo(id, true);
  const path = markRaw(
    L.multiOptionsPolyline(
      plane.pos.planner.past_pos.map((a) => L.latLng(...config.world2map3(a))),
      {
        multiOptions: {
          optionIdxFn: (latLng) => {
            const altThresholds = config.altitudeColours.map((a) => a[0]);

            for (const [i, altThreshold] of altThresholds.entries()) {
              if (latLng.alt <= altThreshold) {
                return i;
              }
            }
            return altThresholds.length;
          },
          options: config.altitudeColours.map((a) => ({ color: a[1] })),
        },
      },
    ).addTo(rawMap()),
  );

  const destLatLng = L.latLng(
    ...config.world2map(
      airportCoords((await getAirportInfo(plane.flight.to)).airport),
    ),
  );
  const planeLatLng = L.latLng(...config.world2map3(plane.pos.pos_ang[0]));
  const plane2dest = markRaw(
    L.polyline([destLatLng, planeLatLng], {
      color: config.plane2destColour ?? "#aaa",
      dashArray: "10 10",
    }).addTo(rawMap()),
  );

  e.popup.setContent(
    `${escape(plane.flight.code)}: ${escape(plane.flight.from)} → ${escape(plane.flight.to)}`,
  );
  selectedPlane.value = {
    id,
    path,
    plane2dest,
  };
}

export function handleStateUpdates() {
  socket.value.on("state", (removed, bin) => {
    for (const remove of removed) {
      planeStates.get(remove)?.marker.remove();
      planeStates.delete(remove);
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

      const state = planeStates.get(id);
      if (state === undefined) {
        planeStates.set(id, {
          s: [sx, sy, sz],
          angle,
          v: [vx, vy],
          marker: markRaw(
            L.circleMarker(config.world2map([sx, sy]), { radius: 5 })
              .on("popupopen", (e) => selectPlane(id, e))
              .on("popupclose", () => deselectPlane())
              .bindPopup("Loading...", { autoPan: false })
              .addTo(rawMap()),
          ),
        });
      } else {
        state.s = [sx, sy, sz];
        state.angle = angle;
        state.v = [vx, vy];
        state.marker.setLatLng(config.world2map([sx, sy]));
      }

      if (selectedPlane.value?.id === id) {
        updateSelectPlane(L.latLng(...config.world2map3([sx, sy, sz])));
      }
    }
  });
}

export function updatePositions(dt: number) {
  const start = Date.now();
  for (const state of planeStates.values()) {
    state.s[0] += state.v[0] * Math.cos(state.angle) * (dt / 1000);
    state.s[1] += state.v[0] * Math.sin(state.angle) * (dt / 1000);
    state.marker.setLatLng(config.world2map3(state.s));
  }
  setTimeout(() => {
    updatePositions(Date.now() - start);
  }, 0);
}
