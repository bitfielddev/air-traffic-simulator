use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::{util::pos::Pos3Angle, world_data::ModelMotion};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Kinematics {
    pub x_target: Vec<Target>,
    pub y_target: Vec<Target>,
    pub v: Vec2,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
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
            (Some(v), Some(s), Some(t)) => {
                todo!()
            }
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
                    // https://www.wolframalpha.com/input?i=s%3D0.5%28u%2Bw%29%28w-u%29%2Fa1+%2B+0.5%28w%2Bv%29%28v-w%29%2Fa2+solve+for+w
                    let w = ((accelerate_a * (v.powi(2) - 2.0 * decelerate_a * s)
                        - decelerate_a * u.powi(2))
                        / (accelerate_a - decelerate_a))
                        .sqrt();
                    vec![
                        Self {
                            a: accelerate_a,
                            t: (w - u) / accelerate_a,
                        },
                        Self {
                            a: decelerate_a,
                            t: (v - w) / decelerate_a,
                        },
                    ]
                }
            }
            (Some(v), None, Some(t)) => {
                todo!()
            }
            (None, Some(s), Some(t)) => {
                todo!()
            }
            (Some(v), None, None) => {
                let a = max_a.copysign(v - u);
                let t = (v - u) / a;
                vec![Self { a, t }]
            }
            (None, Some(s), None) => {
                todo!()
            }
            (None, None, _) => {
                // TODO warn
                vec![]
            }
        }
    }
    pub fn sum_t(targets: Vec<Self>) -> f32 {
        targets.iter().map(|a| a.t).sum()
    }
}

impl Kinematics {
    pub fn tick(&mut self, dt: f32, model_motion: ModelMotion) -> Vec2 {
        let dsx = if self.x_target.is_empty() {
            self.v.x * dt
        } else {
            let mut dt_left = dt;
            let mut dsx = 0.0;
            while let Some(x_target) = self.x_target.first_mut() {
                if dt_left <= 0.0 {
                    break;
                }
                let dt_used = x_target.t.min(dt_left);
                self.v.x += (dt_used * x_target.a);
                dsx += self.v.x * dt_used;

                x_target.t -= dt_used;
                dt_left -= dt_used;
                if x_target.t <= 0.0 {
                    self.x_target.remove(0);
                }
            }
            if self.x_target.is_empty() {
                dsx + self.v.x * dt_left.max(0.0)
            } else {
                dsx
            }
        };
        let dsy = if self.y_target.is_empty() {
            self.v.y * dt
        } else {
            let mut dt_left = dt;
            let mut dsy = 0.0;
            while let Some(y_target) = self.y_target.first_mut() {
                if dt_left <= 0.0 {
                    break;
                }
                let dt_used = y_target.t.min(dt_left);
                self.v.y += (dt_used * y_target.a);
                dsy += self.v.y * dt_used;

                y_target.t -= dt_used;
                dt_left -= dt_used;
                if y_target.t <= 0.0 {
                    self.y_target.remove(0);
                }
            }
            if self.y_target.is_empty() {
                dsy + self.v.y * dt_left.max(0.0)
            } else {
                dsy
            }
        };

        Vec2::new(dsx, dsy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::{angle::Angle, Pos3};

    #[test]
    fn change_altitude() {
        let model_motion = ModelMotion {
            max_a: Vec2::new(f32::INFINITY, 5.0),
            max_v: Vec2::new(f32::INFINITY, 30.0),
            turning_radius: 0.0,
        };
        let mut k = Kinematics {
            y_target: Target::new(
                Some(0.0),
                Some(123.0),
                None,
                model_motion.max_v.y,
                model_motion.max_a.y,
                0.0,
            ),
            ..Kinematics::default()
        };
        let mut pos_ang = Pos3Angle(Pos3::ZERO, Angle(0.0));
        for _ in 0..100 {
            pos_ang.0.z += k.tick(1.0, model_motion).y;
            eprintln!("{:?} {:?} {:?}", pos_ang.0.z, k.v.y, k.y_target.first());
            if k.y_target.is_empty() {
                break;
            }
        }
        assert!(k.y_target.is_empty());
        assert!((pos_ang.0.z - 123.0).abs() < 0.01);
    }

    #[test]
    fn change_velocity() {
        let model_motion = ModelMotion {
            max_a: Vec2::new(5.0, f32::INFINITY),
            max_v: Vec2::new(30.0, f32::INFINITY),
            turning_radius: 0.0,
        };
        let mut k = Kinematics {
            x_target: Target::new(
                Some(30.0),
                None,
                None,
                model_motion.max_v.x,
                model_motion.max_a.x,
                0.0,
            ),
            ..Kinematics::default()
        };
        let mut pos_ang = Pos3Angle(Pos3::ZERO, Angle(0.0));
        for _ in 0..100 {
            pos_ang.0.x += k.tick(1.0, model_motion).x;
            // eprintln!("{:?} {:?}", pos_ang.0.x, k.v.x);
            if k.x_target.is_empty() {
                break;
            }
        }
        assert!(k.x_target.is_empty());
        assert!((k.v.x - 30.0).abs() < 0.01);
    }

    #[test]
    fn change_velocity_with_target_displacement() {
        let model_motion = ModelMotion {
            max_a: Vec2::new(5.0, f32::INFINITY),
            max_v: Vec2::new(30.0, f32::INFINITY),
            turning_radius: 0.0,
        };
        let mut k = Kinematics {
            x_target: Target::new(
                Some(30.0),
                Some(100.0),
                None,
                model_motion.max_v.x,
                model_motion.max_a.x,
                0.0,
            ),
            ..Kinematics::default()
        };
        let mut pos_ang = Pos3Angle(Pos3::ZERO, Angle(0.0));
        for _ in 0..100 {
            pos_ang.0.x += k.tick(1.0, model_motion).x;
            // eprintln!("{:?} {:?} {:?}", pos_ang.0.x, k.v.x, k.x_target.first());
            if k.x_target.is_empty() {
                break;
            }
        }
        assert!(k.x_target.is_empty());
        assert!((k.v.x - 30.0).abs() < 0.01);
        assert!((pos_ang.0.x - 100.0).abs() < 0.01);
    }
}
