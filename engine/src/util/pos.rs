use std::{collections::VecDeque, f32::consts::TAU, sync::Arc};

use derive_more::{Index, IndexMut};
use eyre::Result;
use glam::{Vec2, Vec3Swizzles};
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    util::{
        angle::Angle,
        direction::{Direction, PerpRot, FMB, LMR},
        Pos2, Pos3,
    },
};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Pos3Angle(pub Pos3, pub Angle);

impl Pos3Angle {
    #[must_use]
    pub fn to_2(self) -> Pos2Angle {
        Pos2Angle(self.0.xy(), self.1)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Pos2Angle(pub Pos2, pub Angle);

impl Pos2Angle {
    #[must_use]
    pub const fn to_3(self, z: f32) -> Pos3Angle {
        Pos3Angle(self.0.extend(z), self.1)
    }
}
