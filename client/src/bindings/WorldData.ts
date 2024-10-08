// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { AirportData } from "./AirportData";
import type { Flight } from "./Flight";
import type { PlaneData } from "./PlaneData";
import type { Waypoint } from "./Waypoint";

export interface WorldData {
  classes: string[][];
  airports: AirportData[];
  flights: Flight[] | null;
  planes: PlaneData[];
  waypoints: Waypoint[];
}
