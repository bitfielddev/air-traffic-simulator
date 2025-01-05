use std::{
    cmp::Ordering,
    collections::{HashMap, VecDeque},
    path::Path,
    sync::Arc,
};

use eyre::{eyre, Result};
use glam::Vec2;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use tracing::{trace, warn};
use ts_rs::TS;

use crate::util::{
    pos::Pos2Angle, ray::Ray, AirportCode, Class, FlightCode, PlaneModelId, Pos2, Pos3, WaypointId,
};

#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WorldData {
    #[ts(as = "Arc<[Arc<[String]>]>")]
    pub classes: Arc<[Arc<[Class]>]>,
    pub airports: Arc<[Arc<AirportData>]>,
    pub flights: Option<Arc<[Arc<Flight>]>>,
    pub planes: Arc<[Arc<PlaneData>]>,
    pub waypoints: Arc<[Arc<Waypoint>]>,
}

impl WorldData {
    #[must_use]
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
    #[must_use]
    pub fn airport(&self, code: &AirportCode) -> Option<&Arc<AirportData>> {
        self.airports.iter().find(|a| a.code == *code)
    }
    #[must_use]
    pub fn waypoint(&self, name: &WaypointId) -> Option<&Arc<Waypoint>> {
        self.waypoints.iter().find(|a| a.name == *name)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AirportData {
    #[ts(as = "String")]
    pub name: SmolStr,
    #[ts(as = "String")]
    pub code: AirportCode,
    pub runways: Arc<[Arc<Runway>]>,
}

impl AirportData {
    #[must_use]
    pub fn centre(&self) -> Pos2 {
        self.runways
            .iter()
            .flat_map(|a| [a.start, a.end])
            .sum::<Pos2>()
            / self.runways.len() as f32
            / 2.0
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Runway {
    #[ts(as = "String")]
    pub name: SmolStr,
    #[ts(as = "(f32, f32)")]
    pub start: Pos2,
    #[ts(as = "(f32, f32)")]
    pub end: Pos2,
    pub altitude: f32,
    #[ts(as = "String")]
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
    #[must_use]
    pub fn ray(&self) -> Ray<Vec2> {
        Ray::new(self.start, self.end)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Flight {
    #[ts(as = "String")]
    pub airline: SmolStr,
    #[ts(as = "String")]
    pub code: FlightCode,
    #[ts(as = "String")]
    pub from: AirportCode,
    #[ts(as = "String")]
    pub to: AirportCode,
    #[ts(as = "Arc<[String]>")]
    pub plane: Arc<[PlaneModelId]>,
}

impl Flight {
    pub fn from(&self, wd: &WorldData) -> Result<Arc<AirportData>> {
        let out = wd
            .airports
            .iter()
            .find(|a| a.code == self.from)
            .ok_or_else(|| eyre!("No airport `{}`", self.from))?;
        Ok(Arc::clone(out))
    }
    pub fn to(&self, wd: &WorldData) -> Result<Arc<AirportData>> {
        let out = wd
            .airports
            .iter()
            .find(|a| a.code == self.to)
            .ok_or_else(|| eyre!("No airport `{}`", self.to))?;
        Ok(Arc::clone(out))
    }
    pub fn plane(&self, wd: &WorldData) -> Result<Arc<[Arc<PlaneData>]>> {
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

#[derive(Clone, Debug, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PlaneData {
    #[ts(as = "String")]
    pub id: PlaneModelId,
    #[ts(as = "String")]
    pub name: SmolStr,
    #[ts(as = "String")]
    pub manufacturer: SmolStr,
    #[ts(as = "String")]
    pub class: Class,
    pub motion: ModelMotion,
    pub icon: Option<Arc<Path>>,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ModelMotion {
    #[ts(as = "(f32, f32)")]
    pub max_v: Vec2,
    #[ts(as = "(f32, f32)")]
    pub max_a: Vec2,
    pub turning_radius: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Waypoint {
    #[ts(as = "String")]
    pub name: WaypointId,
    #[ts(as = "(f32, f32)")]
    pub pos: Pos2,
    #[ts(as = "Arc<[String]>")]
    pub connections: Arc<[WaypointId]>,
}

impl WorldData {
    #[tracing::instrument(skip(self))]
    pub fn find_waypoint_route(&self, from: Pos2Angle, to: Pos2) -> VecDeque<Arc<Waypoint>> {
        let Some((from_waypoint, _)) = self
            .waypoints
            .iter()
            .map(|a| {
                (
                    a,
                    a.pos.distance(from.0)
                        / if (a.pos - from.0).dot(to - from.0) > 0.0 {
                            5.0
                        } else {
                            1.0
                        },
                )
            })
            .sorted_by(|(_, a), (_, b)| a.total_cmp(b))
            .next()
        else {
            return VecDeque::new();
        };
        let Some((to_waypoint, _)) = self
            .waypoints
            .iter()
            .map(|a| {
                (
                    a,
                    a.pos.distance(to)
                        / if (a.pos - to).dot(from.0 - to) > 0.0 {
                            5.0
                        } else {
                            1.0
                        },
                )
            })
            .sorted_by(|(_, a), (_, b)| a.total_cmp(b))
            .next()
        else {
            return VecDeque::new();
        };
        trace!(?from_waypoint, ?to_waypoint);

        let h = |w: &Arc<Waypoint>| w.pos.distance(to_waypoint.pos);
        let mut came_from = HashMap::<&WaypointId, &Arc<Waypoint>>::new();
        let mut g_score = HashMap::from([(&from_waypoint.name, 0.0)]);
        let mut f_score = HashMap::from([(&from_waypoint.name, h(from_waypoint))]);

        while let Some((current_name, _)) = f_score
            .iter()
            .map(|(a, b)| (*a, *b))
            .min_by(|(_, v1), (_, v2)| v1.total_cmp(v2))
        {
            let mut current = self.waypoint(current_name).unwrap();
            if current == to_waypoint {
                let mut total_path = VecDeque::from([Arc::clone(current)]);
                while let Some(new_current) = came_from.get(&current.name) {
                    current = new_current;
                    total_path.push_front(Arc::clone(current));
                }
                trace!(?total_path, "Found path");
                return total_path;
            }
            f_score.remove(&current.name);

            for neighbour_name in current.connections.iter() {
                let Some(neighbour) = self.waypoint(neighbour_name) else {
                    continue;
                };
                let tent_g = *g_score.get(&current.name).unwrap_or(&f32::INFINITY)
                    + current.pos.distance(neighbour.pos);
                if tent_g < *g_score.get(&neighbour.name).unwrap_or(&f32::INFINITY) {
                    came_from.insert(&neighbour.name, current);
                    g_score.insert(&neighbour.name, tent_g);
                    f_score.insert(&neighbour.name, tent_g + h(neighbour));
                }
            }
        }

        warn!("Cannot find path");
        VecDeque::new()
    }
}
