use duplicate::duplicate_item;
use glam::{Vec2, Vec3, Vec3Swizzles};

use crate::util::pos::{Pos2Angle, Pos3Angle};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum LMR {
    Left,
    Middle,
    Right,
}
impl LMR {
    #[must_use]
    pub const fn rev(&self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Middle => Self::Middle,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FMB {
    Front,
    Middle,
    Back,
}
impl FMB {
    #[must_use]
    pub const fn rev(&self) -> Self {
        match self {
            Self::Front => Self::Back,
            Self::Back => Self::Front,
            Self::Middle => Self::Middle,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Rotation {
    Clockwise,
    Anticlockwise,
}

impl Rotation {
    #[must_use]
    pub const fn rev(&self) -> Self {
        match self {
            Self::Clockwise => Self::Anticlockwise,
            Self::Anticlockwise => Self::Clockwise,
        }
    }
}

pub trait Direction {
    fn lmr(&self, other: Self) -> LMR;
    fn fmb(&self, other: Self) -> FMB;
    fn turning_rot(&self, other: Self) -> Option<Rotation>;
}

impl Direction for Vec2 {
    fn lmr(&self, other: Self) -> LMR {
        match self.perp_dot(other) {
            a if a > 0.0 => LMR::Left,
            a if a == 0.0 || a == -0.0 => LMR::Middle,
            a if a < 0.0 => LMR::Right,
            _ => {
                // error!(?self.vec, ?other, "NaN detected");
                LMR::Middle
            }
        }
    }
    fn fmb(&self, other: Self) -> FMB {
        match self.dot(other) {
            a if a > 0.0 => FMB::Front,
            a if a == 0.0 || a == -0.0 => FMB::Middle,
            a if a < 0.0 => FMB::Back,
            _ => {
                // error!(?self.vec, ?other, "NaN detected");
                FMB::Middle
            }
        }
    }
    fn turning_rot(&self, other: Self) -> Option<Rotation> {
        match self.lmr(other) {
            LMR::Left => Some(Rotation::Anticlockwise),
            LMR::Right => Some(Rotation::Clockwise),
            LMR::Middle => None,
        }
    }
}
impl Direction for Vec3 {
    #[inline]
    fn lmr(&self, other: Self) -> LMR {
        self.xy().lmr(other.xy())
    }
    #[inline]
    fn fmb(&self, other: Self) -> FMB {
        self.xy().fmb(other.xy())
    }
    #[inline]
    fn turning_rot(&self, other: Self) -> Option<Rotation> {
        self.xy().turning_rot(other.xy())
    }
}
#[duplicate_item(
    Type; [Pos2Angle]; [Pos3Angle];
)]
impl Direction for Type {
    #[inline]
    fn lmr(&self, other: Self) -> LMR {
        self.0.lmr(other.0)
    }
    #[inline]
    fn fmb(&self, other: Self) -> FMB {
        self.0.fmb(other.0)
    }
    #[inline]
    fn turning_rot(&self, other: Self) -> Option<Rotation> {
        self.0.turning_rot(other.0)
    }
}

pub trait PerpRot: Copy {
    #[must_use]
    fn perp_rot(self, rot: Rotation) -> Self;
    #[must_use]
    fn perp_lmr(self, lmr: LMR) -> Self {
        match lmr {
            LMR::Left => self.perp_rot(Rotation::Anticlockwise),
            LMR::Right => self.perp_rot(Rotation::Clockwise),
            LMR::Middle => panic!("Middle"),
        }
    }
}

impl PerpRot for Vec2 {
    fn perp_rot(self, rot: Rotation) -> Self {
        match rot {
            Rotation::Anticlockwise => self.perp(),
            Rotation::Clockwise => -self.perp(),
        }
    }
}
