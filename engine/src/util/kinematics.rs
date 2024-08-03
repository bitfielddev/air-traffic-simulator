use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::{util::pos::Pos3Angle, world_data::ModelMotion};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Kinematics {
    pub target_sy: Option<f32>,
    pub target_vxz: Option<f32>,
    pub target_sxz: Option<f32>,
    pub a: Vec2,
    pub v: Vec2,
}

impl Kinematics {
    pub fn tick(&mut self, dt: f32, pos_ang: Pos3Angle, model_motion: ModelMotion) -> Vec2 {
        // https://gamedev.stackexchange.com/questions/73627/move-a-2d-point-to-a-target-first-accelerate-towards-the-target-then-decelerat
        if let Some(target_sy) = self.target_sy {
            self.a.y = if self.v.y.powi(2) / (2.0 * model_motion.max_a.y) <= target_sy - pos_ang.0.y
            {
                model_motion.max_a.y.copysign(target_sy - pos_ang.0.y)
            } else {
                model_motion.max_a.y.copysign(pos_ang.0.y - target_sy)
            }
        }
        if let Some(target_vxz) = self.target_vxz {
            if let Some(target_sxz) = self.target_sxz {
                self.a.x = self.v.x.mul_add(-self.v.x, target_vxz.powi(2)) / (2.0 * target_sxz);
            } else {
                self.a.x = if target_vxz > self.v.x {
                    model_motion.max_a.x.min(target_vxz - self.v.x)
                } else {
                    (-model_motion.max_a.x).max(target_vxz - self.v.x)
                }
            }
        }

        self.v += (self.a * dt).clamp(-model_motion.max_v, model_motion.max_v);
        let mut ds = self.v * dt;

        if let Some(target_sy) = self.target_sy {
            if (target_sy - pos_ang.0.y).abs() <= (self.v.y * dt).abs() {
                ds.y = target_sy - pos_ang.0.y;
                self.target_sy = None;
                self.a.y = 0.0;
            }
        }
        if let Some(target_vxz) = self.target_vxz {
            if let Some(target_sxz) = self.target_sxz {
                self.target_sxz = Some(target_sxz - ds.x.abs());
            }
            if (target_vxz - self.v.x).abs() <= (self.a.x * dt).abs() {
                self.v.x = target_vxz;
                if let Some(target_sxz) = self.target_sxz {
                    ds.x += target_sxz.copysign(ds.x);
                }
                self.target_vxz = None;
                self.target_sxz = None;
                self.a.x = 0.0;
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
    fn change_altitude() {
        let mut k = Kinematics {
            target_sy: Some(123.0),
            ..Kinematics::default()
        };
        let mut pos_ang = Pos3Angle(Pos3::ZERO, Angle(0.0));
        let model_motion = ModelMotion {
            max_a: Vec2::new(f32::INFINITY, 5.0),
            max_v: Vec2::new(f32::INFINITY, 30.0),
            turning_radius: 0.0,
        };
        for _ in 0..100 {
            pos_ang.0.y += k.tick(1.0, pos_ang, model_motion).y;
            // eprintln!("{:?}", pos_ang.0.y);
            if k.target_sy.is_none() {
                break;
            }
        }
        assert_eq!(k.target_sy, None);
    }

    #[test]
    fn change_velocity() {
        let mut k = Kinematics {
            target_vxz: Some(123.0),
            ..Kinematics::default()
        };
        let mut pos_ang = Pos3Angle(Pos3::ZERO, Angle(0.0));
        let model_motion = ModelMotion {
            max_a: Vec2::new(5.0, f32::INFINITY),
            max_v: Vec2::new(30.0, f32::INFINITY),
            turning_radius: 0.0,
        };
        for _ in 0..100 {
            pos_ang.0.x += k.tick(1.0, pos_ang, model_motion).x;
            // eprintln!("{:?}", k.v.x);
            if k.target_vxz.is_none() {
                break;
            }
        }
        assert_eq!(k.target_vxz, None);
    }

    #[test]
    fn change_velocity_with_target_displacement() {
        let mut k = Kinematics {
            target_vxz: Some(123.0),
            target_sxz: Some(456.0),
            ..Kinematics::default()
        };
        let mut pos_ang = Pos3Angle(Pos3::ZERO, Angle(0.0));
        let model_motion = ModelMotion {
            max_a: Vec2::new(5.0, f32::INFINITY),
            max_v: Vec2::new(30.0, f32::INFINITY),
            turning_radius: 0.0,
        };
        for _ in 0..100 {
            pos_ang.0.x += k.tick(1.0, pos_ang, model_motion).x;
            // eprintln!("{:?} {:?}", k.v.x, pos_ang.0.x);
            if k.target_vxz.is_none() {
                break;
            }
        }
        assert_eq!(k.target_vxz, None);
        assert_eq!(k.target_sxz, None);
    }
}
