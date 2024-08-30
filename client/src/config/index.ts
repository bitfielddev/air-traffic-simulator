// @ts-expect-error
import * as configImport from "./config.js";

interface ClientConfig {
  tileLayer: L.TileLayer;
  socketUri: string;
  altitudeColours: [number, string][];
  world2map: (a: [number, number]) => [number, number];
  world2map3: (a: [number, number, number]) => [number, number, number];
}

const config: ClientConfig = configImport;

export default config;
