use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};

use crate::{
    util::{pos::Pos3Angle, Pos3, Timestamp},
    world_data::{AirportData, Flight, PlaneData, Waypoint, WorldData},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct State {
    pub planes: Vec<Arc<Plane>>,
    pub airports: Arc<[Arc<Airport>]>,
}
impl State {
    #[must_use]
    pub fn new(wd: &WorldData) -> Self {
        Self {
            planes: Vec::default(),
            airports: wd
                .airports
                .iter()
                .map(|a| Arc::new(Airport::new(Arc::clone(a))))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Plane {
    pub pos: PlanePos,
    pub model: Arc<PlaneData>,
    pub flight: Arc<Flight>,
    pub waypoint_route: VecDeque<Arc<Waypoint>>,
    pub phase: Phase,
    pub events: Arc<RwLock<VecDeque<PlaneEvent>>>,
}

impl Plane {
    #[must_use]
    pub fn new(pos: PlanePos, model: &Arc<PlaneData>, flight: &Arc<Flight>) -> Self {
        Self {
            pos,
            model: Arc::clone(model),
            flight: Arc::clone(flight),
            waypoint_route: VecDeque::new(),
            events: Arc::new(RwLock::default()),
            phase: Phase::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlanePos {
    pos_ang: Pos3Angle,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub enum Phase {
    #[default]
    Takeoff,
    Climb,
    Cruise,
    Descent,
    Landing,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Airport {
    pub airport: Arc<AirportData>,
    pub events: Arc<RwLock<VecDeque<AirportEvent>>>,
}
impl Airport {
    #[must_use]
    pub fn new(airport: Arc<AirportData>) -> Self {
        Self {
            airport,
            events: Arc::new(RwLock::default()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlaneEvent {
    pub from: Arc<AirportData>,
    pub payload: PlaneEventPayload,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PlaneEventPayload {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AirportEvent {
    pub from: Arc<Plane>,
    pub payload: AirportEventPayload,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub enum AirportEventPayload {}
