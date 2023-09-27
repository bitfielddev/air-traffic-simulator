use std::f32::consts::{PI, TAU};

use derive_more::{
    Add, AddAssign, Display, Div, DivAssign, From, Into, Mul, MulAssign, Rem, RemAssign, Sub,
    SubAssign,
};
use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::ty::direction::Rotation;

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
    pub fn clamp(mut self) -> Self {
        if self.0 > 0.0 {
            self.0 -= (self.0 / TAU).floor() * TAU;
        } else {
            self.0 -= (self.0 / TAU).ceil() * TAU;
        }
        self
    }
    #[must_use]
    pub fn vec(self) -> Vec2 {
        Vec2::from_angle(self.0)
    }
    #[must_use]
    pub fn turning_rot(self) -> Option<Rotation> {
        match self.0 {
            a if a > 0.0 => Some(Rotation::Anticlockwise),
            a if a == 0.0 || a == -0.0 => None,
            a if a < 0.0 => Some(Rotation::Clockwise),
            _ => {
                // error!("NaN detected")
                None
            }
        }
    }
}
