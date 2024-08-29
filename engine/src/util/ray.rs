use std::ops::{Add, Neg, Sub};

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub trait Vector:
    Copy + Add<Self, Output = Self> + Sub<Self, Output = Self> + Neg<Output = Self>
{
}

impl Vector for Vec2 {}

impl Vector for Vec3 {}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Ray<T: Vector> {
    pub tail: T,
    pub vec: T,
}

impl<T: Vector> Ray<T> {
    pub fn new(tail: T, head: T) -> Self {
        Self {
            tail,
            vec: head - tail,
        }
    }
    #[must_use]
    pub fn head(&self) -> T {
        self.tail + self.vec
    }
    #[must_use]
    pub fn rev(self) -> Self {
        Self {
            tail: self.head(),
            vec: -self.vec,
        }
    }
}
