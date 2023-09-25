use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};

use crate::ty::{
    pos::PlanePos,
    world_data::{Airport, Flight, PlaneModel, Waypoint, WorldData},
    Pos3, Timestamp,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct State {
    pub planes: Vec<Arc<Plane>>,
    pub airports: Arc<[Arc<AirportControl>]>,
}
impl State {
    #[must_use]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Plane {
    pub pos: PlanePos,
    pub model: Arc<PlaneModel>,
    pub flight: Arc<Flight>,
    pub waypoint_route: VecDeque<Arc<Waypoint>>,
    pub route: Vec<(Timestamp, Pos3)>,
    pub events: Arc<RwLock<VecDeque<PlaneEvent>>>,
    pub phase: Phase,
}

impl Plane {
    #[must_use]
    pub fn new(pos: PlanePos, model: &Arc<PlaneModel>, flight: &Arc<Flight>) -> Self {
        Self {
            pos,
            model: Arc::clone(model),
            flight: Arc::clone(flight),
            waypoint_route: VecDeque::new(),
            route: Vec::new(),
            events: Arc::new(RwLock::default()),
            phase: Phase::default(),
        }
    }
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
pub struct AirportControl {
    pub airport: Arc<Airport>,
    pub events: Arc<RwLock<VecDeque<AirportEvent>>>,
}
impl AirportControl {
    #[must_use]
    pub fn new(airport: Arc<Airport>) -> Self {
        Self {
            airport,
            events: Arc::new(RwLock::default()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlaneEvent {
    pub from: Arc<Airport>,
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
