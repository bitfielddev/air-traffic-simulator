// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { AirportData } from "./AirportData";
import type { AirportEvent } from "./AirportEvent";

export interface Airport {
  id: string;
  airport: AirportData;
  events: AirportEvent[];
}