// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { FlightPlanner } from "./FlightPlanner";
import type { Kinematics } from "./Kinematics";
import type { Pos3Angle } from "./Pos3Angle";

export interface PlanePos {
  pos_ang: Pos3Angle;
  kinematics: Kinematics;
  planner: FlightPlanner;
}