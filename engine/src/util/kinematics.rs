use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::world_data::ModelMotion;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Kinematics {
    pub x_target: Vec<Target>,
    pub y_target: Vec<Target>,
    pub v: Vec2,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Target {
    pub a: f32,
    pub dt: f32,
}

impl Target {
    #[must_use]
    pub fn new(
        v: Option<f32>,
        ds: Option<f32>,
        dt: Option<f32>,
        max_v: f32,
        max_a: f32,
        u: f32,
    ) -> Vec<Self> {
        match (v, ds, dt) {
            (Some(v), Some(ds), Some(dt)) => {
                todo!()
            }
            (Some(v), Some(ds), None) => {
                let max_v = max_v.copysign(ds);
                let accelerate_a = max_a.copysign(max_v - u);
                let decelerate_a = max_a.copysign(v - max_v);
                let max_accelerate_ds = u.mul_add(-u, max_v.powi(2)) / accelerate_a / 2.0;
                let max_decelerate_ds = max_v.mul_add(-max_v, v.powi(2)) / decelerate_a / 2.0;
                if ds.abs() > (max_accelerate_ds + max_decelerate_ds).abs() {
                    let accelerate_dt = (max_v - u) / accelerate_a;
                    let constant_dt = (ds - max_accelerate_ds - max_decelerate_ds) / max_v;
                    let decelerate_dt = (v - max_v) / decelerate_a;
                    vec![
                        Self {
                            a: accelerate_a,
                            dt: accelerate_dt,
                        },
                        Self {
                            a: 0.0,
                            dt: constant_dt,
                        },
                        Self {
                            a: decelerate_a,
                            dt: decelerate_dt,
                        },
                    ]
                } else {
                    // https://www.wolframalpha.com/input?i=s%3D0.5%28u%2Bw%29%28w-u%29%2Fa1+%2B+0.5%28w%2Bv%29%28v-w%29%2Fa2+solve+for+w
                    let w = (accelerate_a.mul_add(
                        v.mul_add(v, -(2.0 * decelerate_a * ds)),
                        -(decelerate_a * u.powi(2)),
                    ) / (accelerate_a - decelerate_a))
                        .sqrt()
                        .copysign(ds);
                    let accelerate_dt = (w - u) / accelerate_a;
                    let decelerate_dt = (v - w) / decelerate_a;
                    vec![
                        Self {
                            a: accelerate_a,
                            dt: accelerate_dt,
                        },
                        Self {
                            a: decelerate_a,
                            dt: decelerate_dt,
                        },
                    ]
                }
            }
            (Some(v), None, Some(dt)) => {
                todo!()
            }
            (None, Some(ds), Some(dt)) => {
                todo!()
            }
            (Some(v), None, None) => {
                let a = max_a.copysign(v - u);
                let dt = (v - u) / a;
                vec![Self { a, dt }]
            }
            (None, Some(ds), None) => {
                todo!()
            }
            (None, None, _) => {
                // TODO warn
                vec![]
            }
        }
    }
    #[must_use]
    pub fn sum_t(targets: Vec<Self>) -> f32 {
        targets.iter().map(|a| a.dt).sum()
    }
}

impl Kinematics {
    pub fn target_x(
        &mut self,
        v: Option<f32>,
        ds: Option<f32>,
        dt: Option<f32>,
        model_motion: ModelMotion,
    ) -> &Vec<Target> {
        self.x_target = Target::new(
            v,
            ds,
            dt,
            model_motion.max_v.x,
            model_motion.max_a.x,
            self.v.x,
        );
        &self.x_target
    }
    pub fn target_y(
        &mut self,
        v: Option<f32>,
        ds: Option<f32>,
        dt: Option<f32>,
        model_motion: ModelMotion,
    ) -> &Vec<Target> {
        self.y_target = Target::new(
            v,
            ds,
            dt,
            model_motion.max_v.y,
            model_motion.max_a.y,
            self.v.y,
        );
        &self.y_target
    }
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
                let dt_used = x_target.dt.min(dt_left);
                let old_v = self.v.y;
                self.v.x = dt_used
                    .mul_add(x_target.a, self.v.x)
                    .clamp(-model_motion.max_v.x, model_motion.max_v.x);
                dsx += 0.5 * (old_v + self.v.x) * dt_used;

                x_target.dt -= dt_used;
                dt_left -= dt_used;
                if x_target.dt <= 0.0 {
                    self.x_target.remove(0);
                }
            }
            if self.x_target.is_empty() {
                self.v.x.mul_add(dt_left.max(0.0), dsx)
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
                let dt_used = y_target.dt.min(dt_left);
                let old_v = self.v.y;
                self.v.y = dt_used
                    .mul_add(y_target.a, self.v.y)
                    .clamp(-model_motion.max_v.y, model_motion.max_v.y);
                dsy += 0.5 * (old_v + self.v.y) * dt_used;

                y_target.dt -= dt_used;
                dt_left -= dt_used;
                if y_target.dt <= 0.0 {
                    self.y_target.remove(0);
                }
            }
            if self.y_target.is_empty() {
                self.v.y.mul_add(dt_left.max(0.0), dsy)
            } else {
                dsy
            }
        };

        Vec2::new(dsx, dsy)
    }
}

#[cfg(test)]
mod tests {
    use assertables::*;

    use super::*;
    use crate::util::{angle::Angle, pos::Pos3Angle, Pos3};

    #[test]
    fn change_altitude() {
        let model_motion = ModelMotion {
            max_a: Vec2::new(f32::INFINITY, 5.0),
            max_v: Vec2::new(f32::INFINITY, 30.0),
            turning_radius: 0.0,
        };
        let mut k = Kinematics::default();
        k.target_y(Some(0.0), Some(123.0), None, model_motion);
        let mut pos_ang = Pos3Angle(Pos3::ZERO, Angle(0.0));

        for _ in 0..100 {
            pos_ang.0.z += k.tick(1.0, model_motion).y;
            // eprintln!("{:?} {:?} {:?}", pos_ang.0.z, k.v.y, k.y_target.first());
            if k.y_target.is_empty() {
                break;
            }
        }
        assert!(k.y_target.is_empty());
        assert_in_delta!(pos_ang.0.z, 123.0, 1.0);
    }

    #[test]
    fn change_velocity() {
        let model_motion = ModelMotion {
            max_a: Vec2::new(5.0, f32::INFINITY),
            max_v: Vec2::new(30.0, f32::INFINITY),
            turning_radius: 0.0,
        };
        let mut k = Kinematics::default();
        k.target_x(Some(30.0), None, None, model_motion);
        let mut pos_ang = Pos3Angle(Pos3::ZERO, Angle(0.0));

        for _ in 0..100 {
            pos_ang.0.x += k.tick(1.0, model_motion).x;
            // eprintln!("{:?} {:?}", pos_ang.0.x, k.v.x);
            if k.x_target.is_empty() {
                break;
            }
        }
        assert!(k.x_target.is_empty());
        assert_in_delta!(k.v.x, 30.0, 1.0);
    }

    #[test]
    fn change_velocity_with_target_displacement() {
        let model_motion = ModelMotion {
            max_a: Vec2::new(5.0, f32::INFINITY),
            max_v: Vec2::new(30.0, f32::INFINITY),
            turning_radius: 0.0,
        };
        let mut k = Kinematics::default();
        k.target_x(Some(30.0), Some(100.0), None, model_motion);
        let mut pos_ang = Pos3Angle(Pos3::ZERO, Angle(0.0));

        for _ in 0..100 {
            pos_ang.0.x += k.tick(1.0, model_motion).x;
            // eprintln!("{:?} {:?} {:?}", pos_ang.0.x, k.v.x, k.x_target.first());
            if k.x_target.is_empty() {
                break;
            }
        }
        assert!(k.x_target.is_empty());
        assert_in_delta!(k.v.x, 30.0, 1.0);
        assert_gt!(pos_ang.0.x, 100.0);
    }
}
