use std::{
    f32::consts::{PI, TAU},
    ops::{Add, AddAssign, Sub, SubAssign},
};

use duplicate::duplicate_item;
use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize)]
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

#[duplicate_item(
    Trait method method2      Rhs    expr   ;
    [Add] [add]  [add_assign] [Self] [rhs.0];
    [Add] [add]  [add_assign] [f32]  [rhs  ];
    [Sub] [sub]  [sub_assign] [Self] [rhs.0];
    [Sub] [sub]  [sub_assign] [f32]  [rhs  ];
)]
impl Trait<Rhs> for Angle {
    type Output = Self;
    fn method(mut self, rhs: Rhs) -> Self::Output {
        self.0.method2(expr);
        self.check();
        self
    }
}

#[duplicate_item(
    Trait       method       method2 Rhs   ;
    [AddAssign] [add_assign] [add]   [Self];
    [AddAssign] [add_assign] [add]   [f32 ];
    [SubAssign] [sub_assign] [sub]   [Self];
    [SubAssign] [sub_assign] [sub]   [f32 ];
)]
impl Trait<Rhs> for Angle {
    fn method(&mut self, rhs: Rhs) {
        *self = self.method2(rhs);
    }
}
