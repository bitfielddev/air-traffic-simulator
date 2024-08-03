use std::{collections::VecDeque, sync::Arc};

use dubins_paths::DubinsPath;
use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::{
    util::pos::Pos3Angle,
    world_data::{ModelMotion, Waypoint},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlanePos {
    pub pos_ang: Pos3Angle,
    pub kinematics: Kinematics,
    pub waypoint_route: VecDeque<Arc<Waypoint>>,
    pub model_motion: ModelMotion,
    pub plan: VecDeque<FlightPlan>,
    pub plan_s: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FlightPlan {
    Dubins(#[serde(skip)] DubinsPath), // TODO
    Straight(f32),
    Turn { angle: f32, radius: f32 },
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Kinematics {
    target_sy: Option<f32>,
    target_vxz: Option<f32>,
    target_sxz: Option<f32>,
    a: Vec2,
    v: Vec2,
}

impl PlanePos {
    pub fn tick(&mut self, dt: f32) {
        todo!()
    }
}

impl Kinematics {
    pub fn tick(&mut self, dt: f32, pos_ang: Pos3Angle, model_motion: ModelMotion) -> Vec2 {
        // https://gamedev.stackexchange.com/questions/73627/move-a-2d-point-to-a-target-first-accelerate-towards-the-target-then-decelerat
        if let Some(target_sy) = self.target_sy {
            // if (target_sy - pos_ang.0.y).abs() <= (self.v.y * dt).abs() {
            //     ds_override.y = target_sy - pos_ang.0.y;
            //     self.v.y = 0.0;
            //     self.target_sy = None;
            //     self.a.y = 0.0;
            // } else
            if self.v.y.powi(2) / (2.0 * model_motion.ver_accel) <= target_sy - pos_ang.0.y {
                self.a.y = model_motion.ver_accel.copysign(target_sy - pos_ang.0.y);
            } else {
                self.a.y = model_motion.ver_accel.copysign(pos_ang.0.y - target_sy);
            }
        }

        self.v += self.a * dt;
        self.v.y = self
            .v
            .y
            .clamp(-model_motion.max_ver_vel, model_motion.max_ver_vel);
        let mut ds = self.v * dt;

        if let Some(target_sy) = self.target_sy {
            if (target_sy - pos_ang.0.y).abs() <= (self.v.y * dt).abs() {
                ds.y = target_sy - pos_ang.0.y;
                self.target_sy = None;
                self.a.y = 0.0;
            }
        }

        ds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::{angle::Angle, Pos3};

    #[test]
    fn ascend() {
        let mut k = Kinematics::default();
        k.target_sy = Some(123.0);
        let mut pos_ang = Pos3Angle(Pos3::ZERO, Angle(0.0));
        let model_motion = ModelMotion {
            max_hor_vel: 0.0,
            hor_accel: 0.0,
            max_ver_vel: 30.0,
            ver_accel: 5.0,
            turning_radius: 0.0,
        };
        for _ in 0..100 {
            pos_ang.0.y += k.tick(1.0, pos_ang, model_motion).y;
            eprintln!("{:?}", pos_ang.0.y);
            if k.target_sy.is_none() {
                break;
            }
        }
        assert_eq!(k.target_sy, None);
    }
}
