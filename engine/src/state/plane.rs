use std::{collections::VecDeque, sync::Arc, time::SystemTime};

use dubins_paths::DubinsPath;
use glam::Vec3Swizzles;
use serde::{Deserialize, Serialize};
use tracing::info;
use ts_rs::TS;
use uuid::Uuid;

use crate::{
    config::Config,
    state::{
        airport::{AirportEvent, AirportEventPayload},
        plane_pos::{FlightInstruction, FlightPlanner, PlanePos},
    },
    util::{
        angle::Angle,
        kinematics::{Kinematics, Target},
        pos::{Pos2Angle, Pos3Angle},
        ray::Ray,
        AirportStateId, PlaneStateId, Pos2,
    },
    world_data::{Flight, PlaneData, Runway, WorldData},
};

#[derive(
    Clone, Debug, Deserialize, Serialize, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive, TS,
)]
#[ts(export)]
pub struct Plane {
    pub id: PlaneStateId,
    pub pos: PlanePos,
    pub model: Arc<PlaneData>,
    pub flight: Arc<Flight>,
    pub phase: PhaseData,
    #[ts(as = "Vec<PlaneEvent>")]
    pub events: VecDeque<PlaneEvent>,
    pub start_time: u64,
}

struct PlaneEventsResult {
    landing_runway: Option<Arc<Runway>>,
}

enum PlanePhaseResult {
    NoChange,
    Remove,
    NewPhase(PhaseData),
}

impl Plane {
    #[must_use]
    pub fn new(
        model: &Arc<PlaneData>,
        flight: &Arc<Flight>,
        runway: &Arc<Runway>,
        wd: &WorldData,
    ) -> Self {
        let pos_ang_start = Pos3Angle(
            runway.start3(),
            Angle((runway.end - runway.start).to_angle()),
        );
        let pos_ang_end = Pos2Angle(runway.end, Angle((runway.end - runway.start).to_angle()));
        let mut s = Self {
            id: Uuid::new_v4(),
            pos: PlanePos {
                pos_ang: pos_ang_start,
                kinematics: Kinematics::default(),
                planner: FlightPlanner::new(
                    VecDeque::from([FlightInstruction::Straight(runway.ray())]),
                    wd.find_waypoint_route(
                        pos_ang_end,
                        wd.airport(&flight.to).map_or(Pos2::ZERO, |a| a.centre()),
                    ),
                ),
            },
            model: Arc::clone(model),
            flight: Arc::clone(flight),
            events: VecDeque::new(),
            phase: PhaseData::Takeoff {
                runway: Arc::clone(runway),
            },
            start_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        s.pos.kinematics.target_x(
            Some(s.model.motion.max_v.x),
            None,
            None,
            None,
            s.model.motion,
        );
        s
    }
    fn handle_events(&mut self) -> PlaneEventsResult {
        let mut landing_runway = None;
        for event in self.events.drain(..) {
            match event.payload {
                PlaneEventPayload::ClearForLanding(runway) => {
                    landing_runway = Some(runway);
                }
            }
        }

        PlaneEventsResult { landing_runway }
    }
    fn handle_takeoff_phase(&mut self, config: &Config, runway: &Arc<Runway>) -> PlanePhaseResult {
        let plane_pos = self.pos.pos_ang.0.xy();
        let runway_progress = plane_pos.distance(runway.start) / runway.end.distance(runway.start);

        if runway_progress < 0.75 {
            return PlanePhaseResult::NoChange;
        }

        let cruising_altitude = self
            .pos
            .planner
            .past_route
            .last()
            .or_else(|| self.pos.planner.route.front())
            .map(|a| a.pos)
            .map_or_else(
                || config.min_cruising_altitude(),
                |a| config.cruising_altitude(plane_pos, a),
            );
        self.pos.kinematics.target_y(
            Some(0.0),
            Some(cruising_altitude - self.pos.pos_ang.0.z),
            None,
            None,
            self.model.motion,
        );
        PlanePhaseResult::NewPhase(PhaseData::Cruise)
    }
    fn handle_cruise_phase(
        &mut self,
        send: &mut Vec<(AirportStateId, AirportEvent)>,
    ) -> PlanePhaseResult {
        if !self.pos.planner.route.is_empty() || !self.pos.planner.instructions.is_empty() {
            return PlanePhaseResult::NoChange;
        }
        send.push((
            self.flight.to.clone(),
            AirportEvent {
                from: self.id,
                payload: AirportEventPayload::RequestRunway,
            },
        ));
        self.pos.kinematics.target_x(
            Some(self.model.motion.max_v.x * 0.75),
            None,
            None,
            None,
            self.model.motion,
        );
        PlanePhaseResult::NewPhase(PhaseData::Descent)
    }
    fn handle_descent_phase(&mut self, ev_result: &PlaneEventsResult) -> PlanePhaseResult {
        let Some(landing_runway) = &ev_result.landing_runway else {
            return PlanePhaseResult::NoChange;
        };
        let landing_ray = Ray {
            tail: landing_runway.start - landing_runway.ray().vec,
            vec: landing_runway.ray().vec * 2.0,
        };
        let dubins = FlightInstruction::Dubins(
            DubinsPath::shortest_from(
                self.pos.pos_ang.to_2().into(),
                Pos2Angle(
                    landing_ray.tail,
                    Angle((landing_runway.end - landing_runway.start).to_angle()),
                )
                .into(),
                self.model.motion.turning_radius,
            )
            .unwrap(),
        );
        let straight = FlightInstruction::Straight(landing_ray);
        let touchdown_length = landing_runway.len() * 0.75;
        self.pos.planner.instructions.extend([dubins, straight]);

        let ds = self
            .pos
            .planner
            .instructions
            .iter()
            .map(FlightInstruction::length)
            .sum::<f32>()
            - touchdown_length;
        let dt = Target::sum_t(
            self.pos
                .kinematics
                .target_x(
                    Some(self.model.motion.max_v.x * 0.75),
                    Some(ds),
                    None,
                    None,
                    self.model.motion,
                )
                .iter(),
        );
        self.pos.kinematics.target_y(
            Some(0.0),
            Some(landing_runway.altitude - self.pos.pos_ang.0.z),
            Some(dt),
            None,
            self.model.motion,
        );
        self.pos.kinematics.x_target.push(Target {
            a: (self.model.motion.max_v.x * 0.75).mul_add(-(self.model.motion.max_v.x * 0.75), 1.0)
                / touchdown_length
                / 2.0,
            dt: 2.0 * touchdown_length / self.model.motion.max_v.x.mul_add(0.75, 1.0),
        }); // TODO
        PlanePhaseResult::NewPhase(PhaseData::Landing {
            runway: Arc::clone(landing_runway),
        })
    }
    fn handle_landing_phase(&self) -> PlanePhaseResult {
        if self.pos.planner.instructions.is_empty()
            || self.pos.kinematics.v.x < self.model.motion.max_v.x / 10.0
        {
            return PlanePhaseResult::Remove;
        }
        PlanePhaseResult::NoChange
    }
    #[tracing::instrument(skip_all, fields(%self.id, %self.model.id, %self.flight.code, %self.flight.from, %self.flight.to))]
    pub fn tick(&mut self, config: &Config) -> (bool, Vec<(AirportStateId, AirportEvent)>) {
        let mut send = vec![];
        let ev_result = self.handle_events();

        let phase_handle_result = match self.phase.clone() {
            PhaseData::Takeoff { runway } => self.handle_takeoff_phase(config, &runway),
            PhaseData::Cruise => self.handle_cruise_phase(&mut send),
            PhaseData::Descent => self.handle_descent_phase(&ev_result),
            PhaseData::Landing { runway: _runway } => self.handle_landing_phase(),
        };
        let remove = match phase_handle_result {
            PlanePhaseResult::NewPhase(new_phase) => {
                info!(phase=?new_phase.str(), "Changing phase");
                self.phase = new_phase;
                false
            }
            PlanePhaseResult::Remove => true,
            PlanePhaseResult::NoChange => false,
        };

        self.pos
            .tick(config.tick_duration, self.model.motion, config);
        (remove, send)
    }
}

#[derive(
    Clone, Debug, Deserialize, Serialize, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive, TS,
)]
#[ts(export)]
pub enum PhaseData {
    Takeoff { runway: Arc<Runway> },
    Cruise,
    Descent,
    Landing { runway: Arc<Runway> },
}

impl PhaseData {
    #[must_use]
    pub const fn str(&self) -> &'static str {
        match self {
            Self::Takeoff { .. } => "Takeoff",
            Self::Cruise => "Cruise",
            Self::Descent => "Descent",
            Self::Landing { .. } => "Landing",
        }
    }
}

#[derive(
    Clone, Debug, Deserialize, Serialize, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive, TS,
)]
#[ts(export)]
pub struct PlaneEvent {
    #[ts(as = "String")]
    pub from: AirportStateId,
    pub payload: PlaneEventPayload,
}

#[derive(
    Clone, Debug, Deserialize, Serialize, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive, TS,
)]
#[ts(export)]
#[non_exhaustive]
pub enum PlaneEventPayload {
    ClearForLanding(Arc<Runway>),
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::{
        state::{airport::Airport, State},
        util::{Pos2, WaypointId},
        world_data::{AirportData, Flight, ModelMotion, PlaneData, Runway, Waypoint},
    };

    #[test]
    fn go_around() {
        let mut state = State::new(&[]);
        let runway = Arc::new(Runway {
            start: Pos2::ZERO,
            end: Pos2::new(50.0, 0.0),
            ..Runway::default()
        });
        state.airports.push(Airport::new(Arc::new(AirportData {
            code: "ABC".into(),
            runways: Arc::new([Arc::clone(&runway)]),
            ..AirportData::default()
        })));
        state.planes.push(Plane::new(
            &Arc::new(PlaneData {
                motion: ModelMotion {
                    max_a: Vec2::new(5.0, 2.5),
                    max_v: Vec2::new(50.0, 10.0),
                    turning_radius: 50.0,
                },
                ..PlaneData::default()
            }),
            &Arc::new(Flight {
                to: "ABC".into(),
                ..Flight::default()
            }),
            &runway,
            &WorldData::default(),
        ));
        let config = Config {
            tick_duration: 1.0,
            plane_spawn_chance: 0.0,
            ..Default::default()
        };
        for _ in 0..100 {
            state.tick(&config, &WorldData::default());
            if state.planes.is_empty() {
                break;
            }
            // eprintln!(
            //     "{:?}\n{:?}\n{:?}\n",
            //     state.planes[0].pos.pos_ang, state.planes[0].phase, state.planes[0].pos.kinematics
            // );
            // eprintln!("{} {} {}", state.planes[0].pos.pos_ang.0.x, state.planes[0].pos.pos_ang.0.y, state.planes[0].pos.pos_ang.0.z);
        }
    }

    #[test]
    fn two_waypoints() {
        let mut state = State::new(&[]);
        let runway = Arc::new(Runway {
            start: Pos2::ZERO,
            end: Pos2::new(50.0, 0.0),
            ..Runway::default()
        });
        state.airports.push(Airport::new(Arc::new(AirportData {
            code: "ABC".into(),
            runways: Arc::new([Arc::clone(&runway)]),
            ..AirportData::default()
        })));
        state.planes.push(Plane::new(
            &Arc::new(PlaneData {
                motion: ModelMotion {
                    max_a: Vec2::new(5.0, 2.5),
                    max_v: Vec2::new(50.0, 10.0),
                    turning_radius: 50.0,
                },
                ..PlaneData::default()
            }),
            &Arc::new(Flight {
                to: "ABC".into(),
                ..Flight::default()
            }),
            &runway,
            &WorldData::default(),
        ));
        state.planes[0]
            .pos
            .planner
            .route
            .push_back(Arc::new(Waypoint {
                name: WaypointId::default(),
                pos: Pos2::new(50.0, -100.0),
                connections: Arc::new([]),
            }));
        state.planes[0]
            .pos
            .planner
            .route
            .push_back(Arc::new(Waypoint {
                name: WaypointId::default(),
                pos: Pos2::new(150.0, 100.0),
                connections: Arc::new([]),
            }));
        let config = Config {
            tick_duration: 0.25,
            plane_spawn_chance: 0.0,
            ..Default::default()
        };
        for _ in 0..250 {
            state.tick(&config, &WorldData::default());
            if state.planes.is_empty() {
                break;
            }
            // eprintln!(
            //     "{:?}\n{:?}\n{:?}\n",
            //     state.planes[0].pos.pos_ang, state.planes[0].phase, state.planes[0].pos.kinematics
            // );
            // eprintln!(
            //     "{} {} {}",
            //     state.planes[0].pos.pos_ang.0.x,
            //     state.planes[0].pos.pos_ang.0.y,
            //     state.planes[0].pos.pos_ang.0.z
            // );
        }
    }

    #[test]
    fn waypoint_auto_plan() {
        let runway = Arc::new(Runway {
            start: Pos2::ZERO,
            end: Pos2::new(50.0, 0.0),
            ..Runway::default()
        });
        let airport_data = Arc::new(AirportData {
            code: "ABC".into(),
            runways: Arc::new([Arc::clone(&runway)]),
            ..AirportData::default()
        });
        let wd = WorldData {
            classes: Arc::new([]),
            airports: Arc::new([Arc::clone(&airport_data)]),
            flights: None,
            planes: Arc::new([]),
            waypoints: Arc::new([Arc::new(Waypoint {
                name: WaypointId::default(),
                pos: Pos2::new(50.0, -100.0),
                connections: Arc::new([]),
            })]),
        };
        let mut state = State::new(&[]);
        state.airports.push(Airport::new(airport_data));
        state.planes.push(Plane::new(
            &Arc::new(PlaneData {
                motion: ModelMotion {
                    max_a: Vec2::new(5.0, 2.5),
                    max_v: Vec2::new(50.0, 10.0),
                    turning_radius: 50.0,
                },
                ..PlaneData::default()
            }),
            &Arc::new(Flight {
                to: "ABC".into(),
                ..Flight::default()
            }),
            &runway,
            &wd,
        ));
        let config = Config {
            tick_duration: 0.25,
            plane_spawn_chance: 0.0,
            ..Default::default()
        };
        for _ in 0..250 {
            state.tick(&config, &wd);
            if state.planes.is_empty() {
                break;
            }
            // eprintln!(
            //     "{:?}\n{:?}\n{:?}\n{:?}\n",
            //     state.planes[0].pos.pos_ang, state.planes[0].phase, state.planes[0].pos.kinematics, state.planes[0].pos.planner.route
            // );
            // eprintln!(
            //     "{} {} {}",
            //     state.planes[0].pos.pos_ang.0.x,
            //     state.planes[0].pos.pos_ang.0.y,
            //     state.planes[0].pos.pos_ang.0.z
            // );
        }
    }
}
