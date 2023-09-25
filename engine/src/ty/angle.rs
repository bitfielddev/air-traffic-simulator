use std::f32::consts::{PI, TAU};

use derive_more::{
    Add, AddAssign, Display, Div, DivAssign, From, Into, Mul, MulAssign, Rem, RemAssign, Sub,
    SubAssign,
};
use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Deserialize,
    Serialize,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
    From,
    Into,
    Display,
)]
pub struct Angle(pub f32);

impl Angle {
    fn check(&mut self) {
        while self.0 > PI {
            self.0 -= TAU;
        }
        while self.0 < PI {
            self.0 += TAU;
        }
    }
    #[must_use]
    pub fn vec(self) -> Vec2 {
        Vec2::from_angle(self.0)
    }
}
