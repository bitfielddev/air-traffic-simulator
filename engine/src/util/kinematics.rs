use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::{util::pos::Pos3Angle, world_data::ModelMotion};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Kinematics {
    pub x_target: Vec<Target>,
    pub y_target: Vec<Target>,
    pub a: Vec2,
    pub v: Vec2,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Target {
    pub a: f32,
    pub t: f32,
}

impl Target {
    pub fn new(
        v: Option<f32>,
        s: Option<f32>,
        t: Option<f32>,
        max_v: f32,
        max_a: f32,
        u: f32,
    ) -> Vec<Target> {
        match (v, s, t) {
            (Some(v), Some(s), Some(t)) => {}
            (Some(v), Some(s), None) => {
                let max_v = max_v.copysign(s);
                let accelerate_a = max_a.copysign(max_v - u);
                let decelerate_a = max_a.copysign(v - max_v);
                let max_accelerate_s = (max_v.powi(2) - u.powi(2)) / accelerate_a / 2.0;
                let max_decelerate_s = (v.powi(2) - max_v.powi(2)) / decelerate_a / 2.0;
                if s.abs() > (max_accelerate_s + max_decelerate_s).abs() {
                    let accelerate_t = (max_v - u) / accelerate_a;
                    let constant_t = (s - max_accelerate_s - max_decelerate_s) / max_v;
                    let decelerate_t = (v - max_v) / decelerate_a;
                    vec![
                        Self {
                            a: accelerate_a,
                            t: accelerate_t,
                        },
                        Self {
                            a: 0.0,
                            t: constant_t,
                        },
                        Self {
                            a: decelerate_a,
                            t: decelerate_t,
                        },
                    ]
                } else {
                    vec![Self {}, Self {}]
                }
            }
            (Some(v), None, Some(t)) => {}
            (None, Some(s), Some(t)) => {}
            (Some(v), None, None) => {}
            (None, None, Some(t)) => {}
            (None, Some(s), None) => {}
            (None, None, None) => {
                // TODO warn
                vec![]
            }
        }
    }
}

impl Kinematics {
    pub fn tick(&mut self, dt: f32, pos_ang: Pos3Angle, model_motion: ModelMotion) -> Vec2 {
        let old_v = self.v;

        // https://gamedev.stackexchange.com/questions/73627/move-a-2d-point-to-a-target-first-accelerate-towards-the-target-then-decelerat
        if let Some(target_sz) = self.target_sz {
            self.a.y = if self.v.y.powi(2) / (2.0 * model_motion.max_a.y)
                <= (target_sz - pos_ang.0.z).abs()
            {
                model_motion.max_a.y.copysign(target_sz - pos_ang.0.z)
            } else {
                model_motion.max_a.y.copysign(pos_ang.0.z - target_sz)
            };
        }
        if let Some(target_vxy) = self.target_vxy {
            if let Some(target_sxy) = self.target_sxy {
                self.a.x = self.v.x.mul_add(-self.v.x, target_vxy.powi(2)) / (2.0 * target_sxy);
            } else {
                self.a.x = if target_vxy > self.v.x {
                    model_motion.max_a.x.min((target_vxy - self.v.x) / dt)
                } else {
                    (-model_motion.max_a.x).max((target_vxy - self.v.x) / dt)
                }
            }
        }

        self.v = (self.v + self.a * dt).clamp(-model_motion.max_v, model_motion.max_v);
        let mut ds = self.v * dt;

        if let Some(target_sz) = self.target_sz {
            if (target_sz - (pos_ang.0.z + ds.y)).abs() < 0.01
                || (target_sz - pos_ang.0.z).signum() != (target_sz - pos_ang.0.z - ds.y).signum()
            {
                ds.y = target_sz - pos_ang.0.z;
                self.target_sz = None;
                self.v.y = 0.0;
                self.a.y = 0.0;
            }
        }
        if let Some(target_vxy) = self.target_vxy {
            if let Some(target_sxy) = self.target_sxy {
                self.target_sxy = Some(target_sxy - ds.x);
            }
            if (target_vxy - self.v.x).abs() < 0.01
                || (target_vxy - self.v.x).signum() != (target_vxy - old_v.x).signum()
            {
                self.v.x = target_vxy;
                if let Some(target_sxy) = self.target_sxy {
                    ds.x -= target_sxy.copysign(ds.x);
                }
                self.target_vxy = None;
                self.target_sxy = None;
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
            target_sz: Some(123.0),
            ..Kinematics::default()
        };
        let mut pos_ang = Pos3Angle(Pos3::ZERO, Angle(0.0));
        let model_motion = ModelMotion {
            max_a: Vec2::new(f32::INFINITY, 5.0),
            max_v: Vec2::new(f32::INFINITY, 30.0),
            turning_radius: 0.0,
        };
        for _ in 0..100 {
            pos_ang.0.z += k.tick(1.0, pos_ang, model_motion).y;
            // eprintln!("{:?}", pos_ang.0.z);
            if k.target_sz.is_none() {
                break;
            }
        }
        assert_eq!(k.target_sz, None);
        assert!((pos_ang.0.z - 123.0).abs() < 0.01);
    }

    #[test]
    fn change_velocity() {
        let mut k = Kinematics {
            target_vxy: Some(30.0),
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
            // eprintln!("{:?} {:?}", pos_ang.0.x, k.v.x);
            if k.target_vxy.is_none() {
                break;
            }
        }
        assert_eq!(k.target_vxy, None);
        assert!((k.v.x - 30.0).abs() < 0.01);
    }

    #[test]
    fn change_velocity_with_target_displacement() {
        let mut k = Kinematics {
            target_vxy: Some(30.0),
            target_sxy: Some(100.0),
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
            if k.target_vxy.is_none() {
                break;
            }
        }
        assert_eq!(k.target_vxy, None);
        assert_eq!(k.target_sxy, None);
        assert!((k.v.x - 30.0).abs() < 0.01);
        assert!((pos_ang.0.x - 100.0).abs() < 0.01);
    }
}
