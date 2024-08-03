use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use glam::Vec3Swizzles;
use serde::{Deserialize, Serialize};
use smol_str::ToSmolStr;
use uuid::Uuid;

use crate::{
    config::Config,
    plane_pos::{FlightInstruction, FlightPlanner, PlanePos},
    util::{angle::Angle, kinematics::Kinematics, pos::Pos3Angle, AirportStateId, PlaneStateId},
    world_data::{AirportData, Flight, PlaneData, Runway, WorldData},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct State {
    pub planes: Vec<Plane>,
    pub airports: Vec<Airport>,
}
impl State {
    #[must_use]
    pub fn new(wd: &WorldData) -> Self {
        Self {
            planes: Vec::default(),
            airports: wd
                .airports
                .iter()
                .map(|a| Airport::new(Arc::clone(a)))
                .collect(),
        }
    }
    #[must_use]
    pub fn plane(&self, id: &PlaneStateId) -> Option<&Plane> {
        self.planes.iter().find(|a| a.id == *id)
    }
    #[must_use]
    pub fn plane_mut(&mut self, id: &PlaneStateId) -> Option<&mut Plane> {
        self.planes.iter_mut().find(|a| a.id == *id)
    }
    #[must_use]
    pub fn airport(&self, id: &PlaneStateId) -> Option<&Airport> {
        self.airports.iter().find(|a| a.id == *id)
    }
    #[must_use]
    pub fn airport_mut(&mut self, id: &PlaneStateId) -> Option<&mut Airport> {
        self.airports.iter_mut().find(|a| a.id == *id)
    }
    pub fn tick(&mut self, config: &Config) {
        self.planes.retain_mut(|plane| plane.tick(config));
        for airport in &mut self.airports {
            airport.tick(config);
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Plane {
    pub id: PlaneStateId,
    pub pos: PlanePos,
    pub model: Arc<PlaneData>,
    pub flight: Arc<Flight>,
    pub phase: PhaseData,
    pub events: Arc<RwLock<VecDeque<PlaneEvent>>>,
}

impl Plane {
    #[must_use]
    pub fn new(model: &Arc<PlaneData>, flight: &Arc<Flight>, runway: &Arc<Runway>) -> Self {
        Self {
            id: Uuid::new_v4().to_smolstr(),
            pos: PlanePos {
                pos_ang: Pos3Angle(
                    runway.start3(),
                    Angle((runway.end - runway.start).to_angle()),
                ),
                kinematics: Kinematics {
                    target_vxz: Some(50.0),
                    ..Kinematics::default()
                },
                planner: FlightPlanner {
                    instructions: VecDeque::from([FlightInstruction::Straight(runway.ray())]),
                    ..FlightPlanner::default()
                },
            },
            model: Arc::clone(model),
            flight: Arc::clone(flight),
            events: Arc::new(RwLock::default()),
            phase: PhaseData::Takeoff {
                runway: Arc::clone(runway),
            },
        }
    }
    pub fn tick(&mut self, config: &Config) -> bool {
        if let Some(new_phase) = match &self.phase {
            PhaseData::Takeoff { runway } => {
                let plane_pos = self.pos.pos_ang.0.xy();
                let runway_progress =
                    plane_pos.distance(runway.start) / runway.end.distance(runway.start);
                (runway_progress >= 0.75).then(|| {
                    self.pos.kinematics.target_sy = Some(512.0);
                    PhaseData::Cruise
                })
            }
            PhaseData::Cruise => self
                .pos
                .planner
                .route
                .is_empty()
                .then(|| PhaseData::Descent),
            PhaseData::Descent => {
                todo!("ask airport for runway")
            }
            PhaseData::Landing { runway } => {
                todo!()
            }
        } {
            self.phase = new_phase;
        }
        self.pos.tick(config.tick_duration, self.model.motion);
        true
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PhaseData {
    Takeoff { runway: Arc<Runway> },
    Cruise,
    Descent,
    Landing { runway: Arc<Runway> },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Airport {
    pub id: AirportStateId,
    pub airport: Arc<AirportData>,
    pub events: Arc<RwLock<VecDeque<AirportEvent>>>,
}
impl Airport {
    #[must_use]
    pub fn new(airport: Arc<AirportData>) -> Self {
        Self {
            id: airport.code.clone(),
            airport,
            events: Arc::new(RwLock::default()),
        }
    }
    pub fn tick(&mut self, config: &Config) {}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlaneEvent {
    pub from: AirportStateId,
    pub payload: PlaneEventPayload,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PlaneEventPayload {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AirportEvent {
    pub from: PlaneStateId,
    pub payload: AirportEventPayload,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub enum AirportEventPayload {}
