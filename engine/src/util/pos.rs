use dubins_paths::PosRot;
use glam::Vec3Swizzles;
use serde::{Deserialize, Serialize};

use crate::util::{angle::Angle, Pos2, Pos3};

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

impl From<Pos2Angle> for PosRot {
    fn from(value: Pos2Angle) -> Self {
        Self::new(value.0, value.1 .0)
    }
}

impl From<PosRot> for Pos2Angle {
    fn from(value: PosRot) -> Self {
        Pos2Angle(value.pos(), Angle(value.rot()))
    }
}
