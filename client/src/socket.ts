import { io, Socket } from "socket.io-client";
import { ref, type Ref } from "vue";
import type { Plane } from "./bindings/Plane";
import type { Airport } from "./bindings/Airport";
import type { WorldData } from "./bindings/WorldData";
import type { Config } from "./bindings/Config";
import config from "./config";

interface ServerToClientEvents {
  state: (removed: string[], bin: ArrayBuffer) => void;
}

interface ClientToServerEvents {
  plane: (id: string, cb: (a: Plane) => void) => void;
  airport: (id: string, cb: (a: Airport) => void) => void;
  world_data: (cb: (a: WorldData) => void) => void;
  config: (cb: (a: Config) => void) => void;
  airport_departures: (code: string, cb: (a: string[]) => void) => void;
  airport_arrivals: (code: string, cb: (a: string[]) => void) => void;
}

export default ref(
  io(
    import.meta.env.DEV
      ? "0.0.0.0:3000"
      : (config.socketUri ?? window.location.href),
  ),
) as Ref<Socket<ServerToClientEvents, ClientToServerEvents>>;
