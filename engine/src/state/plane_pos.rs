use std::{collections::VecDeque, sync::Arc};

use dubins_paths::DubinsPath;
use glam::Vec2;
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};
use ts_rs::TS;

use crate::{
    config::Config,
    util::{
        angle::Angle,
        direction::{PerpRot, Rotation},
        kinematics::Kinematics,
        pos::{Pos2Angle, Pos3Angle},
        ray::Ray,
        Pos3,
    },
    world_data::{ModelMotion, Waypoint},
};

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct PlanePos {
    pub pos_ang: Pos3Angle,
    pub kinematics: Kinematics,
    pub planner: FlightPlanner,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct FlightPlanner {
    #[ts(as = "()")]
    pub instructions: VecDeque<FlightInstruction>,
    #[ts(as = "Vec<Arc<Waypoint>>")]
    pub route: VecDeque<Arc<Waypoint>>,
    pub instruction_s: f32,
    #[ts(as = "()")]
    pub past_instructions: Vec<FlightInstruction>,
    pub past_route: Vec<Arc<Waypoint>>,
    #[ts(as = "Vec<(f32, f32, f32)>")]
    pub past_pos: Vec<Pos3>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FlightInstruction {
    Dubins(#[serde(skip)] DubinsPath), // TODO
    Straight(Ray<Vec2>),
    Turn {
        origin: Pos2Angle,
        angle: Angle,
        radius: f32,
    },
}

impl PlanePos {
    pub fn tick(&mut self, dt: f32, model_motion: ModelMotion, config: &Config) {
        let ds = self.kinematics.tick(dt, model_motion);

        let xz = self.planner.tick(
            ds.x,
            self.pos_ang.to_2(),
            model_motion,
            Some((&mut self.kinematics, config, self.pos_ang.0.z)),
        );
        self.planner.past_pos.push(self.pos_ang.0);
        self.pos_ang = Pos3Angle(xz.0.extend(self.pos_ang.0.z + ds.y), xz.1);
    }
}

impl FlightPlanner {
    #[must_use]
    pub fn new(instructions: VecDeque<FlightInstruction>, route: VecDeque<Arc<Waypoint>>) -> Self {
        Self {
            instructions,
            route,
            ..Default::default()
        }
    }
    #[tracing::instrument(skip(model_motion))]
    pub fn tick(
        &mut self,
        dsx: f32,
        pos_ang: Pos2Angle,
        model_motion: ModelMotion,
        mut altitude_changing: Option<(&mut Kinematics, &Config, f32)>,
    ) -> Pos2Angle {
        if self.instructions.is_empty() {
            if let Some(waypoint) = self.route.pop_front() {
                debug!(?waypoint.name, "Planning new instructions");
                let randomness = Vec2::new(
                    rng().random_range(
                        -2.0 * model_motion.turning_radius..2.0 * model_motion.turning_radius,
                    ),
                    rng().random_range(
                        -2.0 * model_motion.turning_radius..2.0 * model_motion.turning_radius,
                    ),
                );
                let waypoint_pos_ang = Pos2Angle(
                    waypoint.pos + randomness,
                    Angle((waypoint.pos - pos_ang.0).to_angle()),
                );
                let mut path = DubinsPath::shortest_from(
                    pos_ang.into(),
                    waypoint_pos_ang.into(),
                    model_motion.turning_radius,
                )
                .unwrap();
                path.param[2] = 0.0;

                if let Some((kinematics, config, z)) = &mut altitude_changing {
                    if !self.past_route.is_empty() {
                        kinematics.target_y(
                            Some(0.0),
                            Some(config.cruising_altitude(pos_ang.0, waypoint.pos) - *z),
                            None,
                            None,
                            model_motion,
                        );
                    }
                }

                self.instructions.push_back(FlightInstruction::Dubins(path));
                self.past_route.push(waypoint);
            } else {
                trace!("Lost");
                return Pos2Angle(pos_ang.0 + pos_ang.1.vec() * dsx, pos_ang.1);
            }
        }
        let instruction = self.instructions.front().unwrap();

        self.instruction_s += dsx;
        if let Some(sample) = instruction.sample(self.instruction_s) {
            sample
        } else {
            let dsx2 = self.instruction_s - instruction.length();
            let pos_ang2 = instruction.end();
            self.instruction_s = 0.0;
            self.past_instructions
                .push(self.instructions.pop_front().unwrap());
            self.tick(dsx2, pos_ang2, model_motion, altitude_changing)
        }
    }
}

impl FlightInstruction {
    #[must_use]
    pub fn length(&self) -> f32 {
        match self {
            Self::Dubins(path) => path.length(),
            Self::Straight(ray) => ray.vec.length(),
            Self::Turn { angle, radius, .. } => radius * angle.0.abs(),
        }
    }
    #[must_use]
    pub fn end(&self) -> Pos2Angle {
        self.sample(self.length()).unwrap()
    }
    #[must_use]
    pub fn sample(&self, s: f32) -> Option<Pos2Angle> {
        if s > self.length() || s < 0.0 {
            return None;
        }
        Some(match self {
            Self::Dubins(path) => path.sample(s).into(),
            Self::Straight(ray) => Pos2Angle(
                ray.tail + ray.vec.normalize() * s,
                Angle(ray.vec.to_angle()),
            ),
            Self::Turn {
                origin,
                angle,
                radius,
            } => {
                let vec = origin.1.vec().perp_rot(if angle.0 >= 0.0 {
                    Rotation::Anticlockwise
                } else {
                    Rotation::Clockwise
                }) * (*radius);
                let rotate = *angle * s / self.length();
                let pos = (origin.0 + vec) + (-vec).rotate(rotate.vec());
                let angle = Angle(origin.1 .0 + rotate.0).clamp();
                Pos2Angle(pos, angle)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use assertables::*;

    use super::*;
    use crate::util::{Pos2, Pos3, WaypointId};

    #[test]
    fn waypoints() {
        let mut plane_pos = PlanePos {
            pos_ang: Pos3Angle(Pos3::ZERO, Angle(0.0)),
            kinematics: Kinematics {
                v: Vec2::new(1.0, 0.0),
                ..Default::default()
            },
            planner: FlightPlanner::new(
                VecDeque::new(),
                VecDeque::from([
                    Arc::new(Waypoint {
                        name: WaypointId::default(),
                        pos: Pos2::new(10.0, 0.0),
                        connections: Arc::new([]),
                    }),
                    Arc::new(Waypoint {
                        name: WaypointId::default(),
                        pos: Pos2::new(10.0, 10.0),
                        connections: Arc::new([]),
                    }),
                ]),
            ),
        };
        let model_motion = ModelMotion {
            max_a: Vec2::INFINITY,
            max_v: Vec2::INFINITY,
            turning_radius: 0.5,
        };

        for _ in 0..25 {
            plane_pos.tick(1.0, model_motion, &Config::default());
            // eprintln!("{:?}", plane_pos.pos_ang);
            if plane_pos.planner.instructions.is_empty() {
                assert_lt!(
                    plane_pos.pos_ang.to_2().0.distance(Pos2::new(10.0, 10.0)),
                    2.0
                );
                break;
            }
        }
    }

    #[test]
    fn straight_turn() {
        let mut plane_pos = PlanePos {
            pos_ang: Pos3Angle(Pos3::ZERO, Angle(0.0)),
            kinematics: Kinematics {
                v: Vec2::new(1.0, 0.0),
                ..Default::default()
            },
            planner: FlightPlanner::new(
                VecDeque::from([
                    FlightInstruction::Straight(Ray::new(Pos2::ZERO, Pos2::new(10.0, 0.0))),
                    FlightInstruction::Turn {
                        origin: Pos2Angle(Pos2::new(10.0, 0.0), Angle(0.0)),
                        radius: 2.0,
                        angle: Angle(PI),
                    },
                ]),
                VecDeque::new(),
            ),
        };
        let model_motion = ModelMotion {
            max_a: Vec2::INFINITY,
            max_v: Vec2::INFINITY,
            turning_radius: 2.0,
        };

        for _ in 0..25 {
            plane_pos.tick(1.0, model_motion, &Config::default());
            // eprintln!("{:?}", plane_pos.pos_ang);
            if plane_pos.planner.instructions.is_empty() {
                assert_lt!(
                    plane_pos.pos_ang.to_2().0.distance(Pos2::new(10.0, 4.0)),
                    1.0
                );
                assert_in_delta!(plane_pos.pos_ang.1 .0, PI, 0.01);
                break;
            }
        }
    }
}
