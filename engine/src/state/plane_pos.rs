use std::{collections::VecDeque, sync::Arc};

use dubins_paths::DubinsPath;
use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::{
    util::{
        angle::Angle,
        direction::{PerpRot, Rotation},
        kinematics::Kinematics,
        pos::{Pos2Angle, Pos3Angle},
        ray::Ray,
    },
    world_data::{ModelMotion, Waypoint},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlanePos {
    pub pos_ang: Pos3Angle,
    pub kinematics: Kinematics,
    pub planner: FlightPlanner,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FlightPlanner {
    pub instructions: VecDeque<FlightInstruction>,
    pub route: VecDeque<Arc<Waypoint>>,
    pub instruction_s: f32,
    pub past_instructions: Vec<FlightInstruction>,
    pub past_route: Vec<Arc<Waypoint>>,
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
    pub fn tick(&mut self, dt: f32, model_motion: ModelMotion) {
        let ds = self.kinematics.tick(dt, self.pos_ang, model_motion);

        let xz = self.planner.tick(ds.x, self.pos_ang.to_2(), model_motion);
        self.pos_ang = Pos3Angle(xz.0.extend(self.pos_ang.0.z + ds.y), xz.1);
    }
}

impl FlightPlanner {
    pub fn tick(&mut self, dsx: f32, pos_ang: Pos2Angle, model_motion: ModelMotion) -> Pos2Angle {
        if self.instructions.is_empty() {
            if let Some(waypoint) = self.route.pop_front() {
                let waypoint_pos_ang =
                    Pos2Angle(waypoint.pos, Angle((waypoint.pos - pos_ang.0).to_angle()));
                let path = DubinsPath::shortest_from(
                    pos_ang.into(),
                    waypoint_pos_ang.into(),
                    model_motion.turning_radius,
                )
                .unwrap();
                self.instructions.push_back(FlightInstruction::Dubins(path));
                self.past_route.push(waypoint);
            } else {
                return Pos2Angle(pos_ang.0 + pos_ang.1.vec() * dsx, pos_ang.1);
            }
        }
        let instruction = self.instructions.front().unwrap();

        self.instruction_s += dsx;
        if let Some(sample) = instruction.sample(self.instruction_s) {
            sample
        } else {
            let dsx2 = self.instruction_s - instruction.len();
            let pos_ang2 = instruction.end();
            self.instruction_s = 0.0;
            self.past_instructions
                .push(self.instructions.pop_front().unwrap());
            self.tick(dsx2, pos_ang2, model_motion)
        }
    }
}

impl FlightInstruction {
    #[must_use]
    pub fn len(&self) -> f32 {
        match self {
            Self::Dubins(path) => path.length(),
            Self::Straight(ray) => ray.vec.length(),
            Self::Turn { angle, radius, .. } => radius * angle.0.abs(),
        }
    }
    #[must_use]
    pub fn end(&self) -> Pos2Angle {
        self.sample(self.len()).unwrap()
    }
    #[must_use]
    pub fn sample(&self, s: f32) -> Option<Pos2Angle> {
        if s > self.len() || s < 0.0 {
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
                let rotate = *angle * s / self.len();
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

    use super::*;
    use crate::util::{Pos2, Pos3, WaypointId};

    #[test]
    fn waypoints() {
        let mut plane_pos = PlanePos {
            pos_ang: Pos3Angle(Pos3::ZERO, Angle(0.0)),
            kinematics: Kinematics {
                target_sz: None,
                target_vxy: None,
                target_sxy: None,
                a: Vec2::default(),
                v: Vec2::new(1.0, 0.0),
            },
            planner: FlightPlanner {
                instructions: VecDeque::new(),
                route: VecDeque::from([
                    Arc::new(Waypoint {
                        name: WaypointId::default(),
                        pos: Pos2::new(10.0, 0.0),
                    }),
                    Arc::new(Waypoint {
                        name: WaypointId::default(),
                        pos: Pos2::new(10.0, 10.0),
                    }),
                ]),
                instruction_s: 0.0,
                past_instructions: vec![],
                past_route: vec![],
            },
        };
        let model_motion = ModelMotion {
            max_a: Vec2::INFINITY,
            max_v: Vec2::INFINITY,
            turning_radius: 2.0,
        };

        for _ in 0..25 {
            plane_pos.tick(1.0, model_motion);
            // eprintln!("{:?}", plane_pos.pos_ang);
        }
    }

    #[test]
    fn straight_turn() {
        let mut plane_pos = PlanePos {
            pos_ang: Pos3Angle(Pos3::ZERO, Angle(0.0)),
            kinematics: Kinematics {
                target_sz: None,
                target_vxy: None,
                target_sxy: None,
                a: Vec2::default(),
                v: Vec2::new(1.0, 0.0),
            },
            planner: FlightPlanner {
                instructions: VecDeque::from([
                    FlightInstruction::Straight(Ray::new(Pos2::ZERO, Pos2::new(10.0, 0.0))),
                    FlightInstruction::Turn {
                        origin: Pos2Angle(Pos2::new(10.0, 0.0), Angle(0.0)),
                        radius: 2.0,
                        angle: Angle(PI),
                    },
                ]),
                route: VecDeque::new(),
                instruction_s: 0.0,
                past_instructions: vec![],
                past_route: vec![],
            },
        };
        let model_motion = ModelMotion {
            max_a: Vec2::INFINITY,
            max_v: Vec2::INFINITY,
            turning_radius: 2.0,
        };

        for _ in 0..25 {
            plane_pos.tick(1.0, model_motion);
            // eprintln!("{:?}", plane_pos.pos_ang);
        }
    }
}
