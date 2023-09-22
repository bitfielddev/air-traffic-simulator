use std::{cmp::Ordering, path::Path, sync::Arc};

use eyre::{eyre, Result};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use crate::ty::{AirportCode, Class, FlightCode, PlaneModelId, Pos2, Pos3, WaypointId};

#[derive(Clone, Serialize, Deserialize)]
pub struct WorldData {
    pub classes: Arc<[Arc<[Class]>]>,
    pub airports: Arc<[Arc<Airport>]>,
    pub flights: Option<Arc<[Arc<Flight>]>>,
    pub planes: Arc<[Arc<PlaneModel>]>,
    pub waypoints: Arc<[Arc<Waypoint>]>,
}

impl WorldData {
    pub fn cmp_class(&self, c1: &Class, c2: &Class) -> Option<Ordering> {
        for class_list in &*self.classes {
            let Some(pos1) = class_list.iter().find(|a| *a == c1) else {
                continue;
            };
            let Some(pos2) = class_list.iter().find(|a| *a == c2) else {
                continue;
            };
            return Some(pos1.cmp(pos2));
        }
        None
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Airport {
    pub name: SmolStr,
    pub code: AirportCode,
    pub runways: Arc<[Runway]>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Runway {
    pub start: Pos2,
    pub end: Pos2,
    pub altitude: f32,
    pub class: Class,
}

impl Runway {
    #[must_use]
    pub fn len(&self) -> f32 {
        (self.start - self.end).length()
    }
    #[must_use]
    pub const fn start3(&self) -> Pos3 {
        self.start.extend(self.altitude)
    }
    #[must_use]
    pub const fn end3(&self) -> Pos3 {
        self.end.extend(self.altitude)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Flight {
    pub airline: SmolStr,
    pub code: FlightCode,
    pub from: AirportCode,
    pub to: AirportCode,
    pub plane: Arc<[PlaneModelId]>,
}

impl Flight {
    pub fn from(&self, wd: &WorldData) -> Result<Arc<Airport>> {
        let out = wd
            .airports
            .iter()
            .find(|a| a.code == self.from)
            .ok_or_else(|| eyre!("No airport `{}`", self.from))?;
        Ok(Arc::clone(out))
    }
    pub fn to(&self, wd: &WorldData) -> Result<Arc<Airport>> {
        let out = wd
            .airports
            .iter()
            .find(|a| a.code == self.to)
            .ok_or_else(|| eyre!("No airport `{}`", self.to))?;
        Ok(Arc::clone(out))
    }
    pub fn plane(&self, wd: &WorldData) -> Result<Arc<[Arc<PlaneModel>]>> {
        self.plane
            .iter()
            .map(|p| {
                wd.planes
                    .iter()
                    .find(|m| m.id == *p)
                    .ok_or_else(|| eyre!("No plane model `{p}`"))
            })
            .map_ok(Arc::clone)
            .collect::<Result<_, _>>()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlaneModel {
    pub id: PlaneModelId,
    pub name: SmolStr,
    pub manufacturer: SmolStr,
    pub class: Class,
    pub motion: ModelMotion,
    pub icon: Option<Arc<Path>>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ModelMotion {
    max_hor_vel: f32,
    hor_accel: f32,
    max_ver_vel: f32,
    ver_accel: f32,
    turning_radius: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Waypoint {
    pub name: WaypointId,
    pub pos: Pos2,
}
