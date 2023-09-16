use glam::Vec3Swizzles;
use serde::{Deserialize, Serialize};

use crate::ty::{angle::Angle, config::Config, Pos3};

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct PlanePos {
    pub motion: Motion,
    pub targets: PosTarget,
    pub max_hor_vel: f32,
    pub max_hor_accel: f32,
}

impl PlanePos {
    pub fn update(&mut self, cfg: &Config) {
        self.targets.update_plan(&self.motion, cfg);
        self.motion.tick(cfg);
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct Motion {
    pub coords: Pos3,
    pub ang: Angle,
    pub hor_vel: f32,
    pub ver_vel: f32,
    pub ang_vel: f32,
    pub accel: f32,
}

impl Motion {
    pub fn tick(&mut self, cfg: &Config) {
        self.hor_vel += self.accel * cfg.tick_duration;
        self.ang += self.ang_vel * cfg.tick_duration;
        self.coords += (self.ang.vec() * self.hor_vel).extend(self.ver_vel) * cfg.tick_duration;
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct PosTarget {
    pub pos: Pos3,
    pub ang: f32,
    pub hor_vel: f32,
    pub match_vel_when_reach_pos: bool,
    pub plan: (),
}

impl PosTarget {
    pub fn update_plan(&mut self, motion: &Motion, cfg: &Config) {
        todo!()
    }
}
