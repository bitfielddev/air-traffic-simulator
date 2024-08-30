// @ts-expect-error
import * as configImport from "./config.js";

interface ClientConfig {
  tileLayer: L.TileLayer;
  socketUri: string;
  altitudeColours: [number, string][];
}

const config: ClientConfig = configImport;

export default config;
