use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    world_data::{Airport, Flight, PlaneModel, Waypoint, WorldData},
    Pos3, Timestamp,
};

#[derive(Clone, Deserialize, Serialize)]
pub struct State {
    pub planes: Vec<Arc<Plane>>,
    pub airports: Vec<Arc<AirportControl>>,
}
impl State {
    pub fn new(wd: &WorldData) -> Self {
        Self {
            planes: Vec::default(),
            airports: wd
                .airports
                .iter()
                .map(|a| Arc::new(AirportControl::new(Arc::clone(a))))
                .collect(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Plane {
    pub pos: PlanePos,
    pub model: Arc<PlaneModel>,
    pub flight: Arc<Flight>,
    pub waypoint_route: VecDeque<Arc<Waypoint>>,
    pub route: Vec<(Timestamp, Pos3)>,
    pub events: Arc<RwLock<VecDeque<PlaneEvent>>>,
    pub phase: Phase,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum Phase {
    Takeoff,
    Climb,
    Cruise,
    Descent,
    Landing,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct PlanePos {
    pub coords: Pos3,
    pub speed: Vec3,
    pub accel: Vec3,
}
impl PlanePos {
    pub fn tick(&mut self, cfg: &Config) {
        self.speed = self.accel * cfg.tick_duration;
        self.coords = self.speed * cfg.tick_duration;
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct AirportControl {
    pub airport: Arc<Airport>,
    pub events: Arc<RwLock<VecDeque<AirportEvent>>>,
}
impl AirportControl {
    pub fn new(airport: Arc<Airport>) -> Self {
        Self {
            airport,
            events: Arc::new(RwLock::default()),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PlaneEvent {
    pub from: Arc<Airport>,
    pub payload: PlaneEventPayload,
}

#[derive(Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PlaneEventPayload {}

#[derive(Clone, Deserialize, Serialize)]
pub struct AirportEvent {
    pub from: Arc<Plane>,
    pub payload: AirportEventPayload,
}

#[derive(Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub enum AirportEventPayload {}
