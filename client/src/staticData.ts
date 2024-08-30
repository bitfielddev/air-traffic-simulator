import type { Config } from "./bindings/Config";
import type { WorldData } from "./bindings/WorldData";
import socket from "./socket";

let worldData: WorldData | undefined;
export async function getWorldData() {
  if (worldData !== undefined) return worldData;
  worldData = await socket.value.timeout(5000).emitWithAck("world_data");
  return worldData!;
}

let config: Config | undefined;
export async function getEngineConfig() {
  if (config !== undefined) return config;
  config = await socket.value.timeout(5000).emitWithAck("config");
  return config!;
}
