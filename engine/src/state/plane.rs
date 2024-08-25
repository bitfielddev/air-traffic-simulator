use std::{collections::VecDeque, sync::Arc};

use dubins_paths::DubinsPath;
use glam::Vec3Swizzles;
use serde::{Deserialize, Serialize};
use smol_str::ToSmolStr;
use uuid::Uuid;

use crate::{
    config::Config,
    state::{
        airport::{AirportEvent, AirportEventPayload},
        plane_pos::{FlightInstruction, FlightPlanner, PlanePos},
    },
    util::{
        angle::Angle,
        kinematics::Kinematics,
        pos::{Pos2Angle, Pos3Angle},
        AirportStateId, PlaneStateId,
    },
    world_data::{Flight, PlaneData, Runway},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Plane {
    pub id: PlaneStateId,
    pub pos: PlanePos,
    pub model: Arc<PlaneData>,
    pub flight: Arc<Flight>,
    pub phase: PhaseData,
    pub events: VecDeque<PlaneEvent>,
}

impl Plane {
    #[must_use]
    pub fn new(model: &Arc<PlaneData>, flight: &Arc<Flight>, runway: &Arc<Runway>) -> Self {
        let mut s = Self {
            id: Uuid::new_v4().to_smolstr(),
            pos: PlanePos {
                pos_ang: Pos3Angle(
                    runway.start3(),
                    Angle((runway.end - runway.start).to_angle()),
                ),
                kinematics: Kinematics::default(),
                planner: FlightPlanner::new(VecDeque::from([FlightInstruction::Straight(
                    runway.ray(),
                )])),
            },
            model: Arc::clone(model),
            flight: Arc::clone(flight),
            events: VecDeque::new(),
            phase: PhaseData::Takeoff {
                runway: Arc::clone(runway),
            },
        };
        s.pos
            .kinematics
            .target_x(Some(s.model.motion.max_v.x), None, None, s.model.motion);
        s
    }
    pub fn tick(&mut self, config: &Config) -> (bool, Vec<(AirportStateId, AirportEvent)>) {
        let mut remove = false;
        let mut send = vec![];

        let mut landing_runway = None;
        for event in self.events.drain(..) {
            match event.payload {
                PlaneEventPayload::ClearForLanding(runway) => {
                    landing_runway = Some(runway);
                }
            }
        }

        if let Some(new_phase) = match &mut self.phase {
            PhaseData::Takeoff { runway } => {
                let plane_pos = self.pos.pos_ang.0.xy();
                let runway_progress =
                    plane_pos.distance(runway.start) / runway.end.distance(runway.start);
                (runway_progress >= 0.75).then(|| {
                    self.pos.kinematics.target_y(
                        Some(0.0),
                        Some(512.0 - self.pos.pos_ang.0.z),
                        None,
                        self.model.motion,
                    );
                    PhaseData::Cruise
                })
            }
            PhaseData::Cruise => self.pos.planner.route.is_empty().then(|| {
                send.push((
                    self.flight.to.clone(),
                    AirportEvent {
                        from: self.id.clone(),
                        payload: AirportEventPayload::RequestRunway,
                    },
                ));
                self.pos.kinematics.target_x(
                    Some(self.model.motion.max_v.x / 2.0),
                    None,
                    None,
                    self.model.motion,
                );
                PhaseData::Descent
            }),
            PhaseData::Descent => {
                landing_runway.map(|landing_runway| {
                    self.pos.planner.instructions.extend([
                        // TODO
                        FlightInstruction::Dubins(
                            DubinsPath::shortest_from(
                                self.pos.pos_ang.to_2().into(),
                                Pos2Angle(
                                    landing_runway.start,
                                    Angle((landing_runway.end - landing_runway.start).to_angle()),
                                )
                                .into(),
                                self.model.motion.turning_radius,
                            )
                            .unwrap(),
                        ),
                        FlightInstruction::Straight(landing_runway.ray()),
                    ]);
                    self.pos.kinematics.target_y(
                        Some(0.0),
                        Some(landing_runway.altitude - self.pos.pos_ang.0.z),
                        None,
                        self.model.motion,
                    );
                    PhaseData::Landing {
                        runway: landing_runway,
                    }
                })
            }
            PhaseData::Landing { runway: _runway } => {
                if self.pos.planner.instructions.is_empty() {
                    remove = true;
                }
                None
            }
        } {
            self.phase = new_phase;
        }
        self.pos.tick(config.tick_duration, self.model.motion);
        (remove, send)
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
pub struct PlaneEvent {
    pub from: AirportStateId,
    pub payload: PlaneEventPayload,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
        util::Pos2,
        world_data::{AirportData, Flight, ModelMotion, PlaneData, Runway},
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
        ));
        let config = Config {
            tick_duration: 1.0,
            plane_spawn_chance: 0.0,
        };
        for _ in 0..100 {
            state.tick(&config);
            if state.planes.is_empty() {
                break;
            }
            eprintln!(
                "{:?}\n{:?}\n{:?}\n",
                state.planes[0].pos.pos_ang, state.planes[0].phase, state.planes[0].pos.kinematics
            );
        }
    }
}
