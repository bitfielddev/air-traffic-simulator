use std::{collections::VecDeque, sync::Arc};

use eyre::Result;
use glam::Vec3Swizzles;
use serde::{Deserialize, Serialize};

use crate::ty::{angle::Angle, config::Config, state::Plane, world_data::PlaneModel, Pos2, Pos3};

#[derive(Clone, Deserialize, Serialize)]
pub struct PlanePos {
    pub pos_angle: Pos3Angle,
    pub route: Vec<(u32, Pos3Angle)>,
    pub hor_plan: VecDeque<Pos2Angle>,
    pub ver_plan: VecDeque<f32>,
    pub model: Arc<PlaneModel>,
}

impl PlanePos {
    pub fn new(pos_angle: Pos3Angle, model: &Arc<PlaneModel>) -> Self {
        Self {
            pos_angle,
            route: Vec::new(),
            hor_plan: VecDeque::default(),
            ver_plan: VecDeque::default(),
            model: Arc::clone(model),
        }
    }
    pub fn update(&mut self, cfg: &Config) {}
    pub fn plan_to_pos2(&mut self, pos2: Pos2) -> Result<()> {
        todo!()
    }
    pub fn plan_to_pos2angle(&mut self, pos2angle: Pos2Angle) -> Result<()> {
        todo!()
    }
    pub fn plan_to_ver(&mut self, z: f32) -> Result<()> {
        todo!()
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct Pos3Angle(pub Pos3, pub Angle);

impl Pos3Angle {
    pub fn to_2(self) -> Pos2Angle {
        Pos2Angle(self.0.xy(), self.1)
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct Pos2Angle(pub Pos2, pub Angle);

impl Pos2Angle {
    pub fn to_3(self, z: f32) -> Pos3Angle {
        Pos3Angle(self.0.extend(z), self.1)
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum HorPlanItem {
    Straight(f32),
    Turn(Angle),
}
