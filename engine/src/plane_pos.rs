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

#[derive(Clone, Debug, Deserialize, Serialize)]
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
