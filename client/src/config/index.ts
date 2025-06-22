// @ts-expect-error
import * as configImport from "./config.js";

interface ClientConfig {
  tileLayer: L.TileLayer | L.FeatureGroup<L.TileLayer>;
  altitudeColours: [number, string][];
  socketUri?: string;
  world2map: (a: [number, number]) => [number, number];
  world2map3: (a: [number, number, number]) => [number, number, number];
  plane2destColour?: string;
}

const config: ClientConfig = configImport;

export default config;
