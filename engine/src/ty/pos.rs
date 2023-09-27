use std::{collections::VecDeque, f32::consts::TAU, sync::Arc};

use derive_more::{Index, IndexMut};
use eyre::Result;
use glam::{Vec2, Vec3Swizzles};
use serde::{Deserialize, Serialize};

use crate::ty::{
    angle::Angle,
    config::Config,
    direction::{Direction, PerpRot, FMB, LMR},
    world_data::{ModelMotion, PlaneModel},
    Pos2, Pos3,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlanePos {
    pub pos_angle: Pos3Angle,
    pub route: Vec<(u32, Pos3Angle)>,
    pub hor_plan: VecDeque<Pos2Angle>,
    pub ver_plan: VecDeque<f32>,
    pub model: Arc<PlaneModel>,
}

impl PlanePos {
    #[must_use]
    pub fn new(pos_angle: Pos3Angle, model: &Arc<PlaneModel>) -> Self {
        Self {
            pos_angle,
            route: Vec::new(),
            hor_plan: VecDeque::default(),
            ver_plan: VecDeque::default(),
            model: Arc::clone(model),
        }
    }
    pub fn update(&mut self, _cfg: &Config) {}
    pub fn plan_to_pos2(&mut self, _pos2: Pos2) -> Result<()> {
        todo!()
    }
    pub fn plan_to_pos2angle(&mut self, _pos2angle: Pos2Angle) -> Result<()> {
        todo!()
    }
    pub fn plan_to_ver(&mut self, _z: f32) -> Result<()> {
        todo!()
    }
}

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

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub enum HorPlanItem {
    Straight(f32),
    Turn(Angle),
}

#[derive(Clone, Debug, Deserialize, Serialize, Index, IndexMut)]
pub struct HorPlanner(VecDeque<HorPlanItem>);
impl HorPlanner {
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    #[must_use]
    pub fn plan_to_pos2(from: Pos2Angle, to: Pos2, mm: ModelMotion) -> Self {
        let c = from.1.vec();
        let mut direction = c.lmr(to);
        if direction == LMR::Middle {
            match c.fmb(to) {
                FMB::Front => {
                    return Self(VecDeque::from([HorPlanItem::Straight(from.0.distance(to))]))
                }
                FMB::Middle => return Self(VecDeque::new()),
                FMB::Back => direction = LMR::Left,
            }
        }
        let mut a1 = -c.perp_lmr(direction) * mm.turning_radius;
        let mut o = from.0 - a1;
        if o.distance(to) < mm.turning_radius {
            direction = direction.rev();
            a1 = -a1;
            o = from.0 - a1;
        }

        let d = to - o;
        let b_mag = mm
            .turning_radius
            .mul_add(-mm.turning_radius, d.length_squared())
            .sqrt();
        let b = d
            .rotate(Vec2::from_angle(
                (mm.turning_radius / d.length()).asin()
                    * if direction == LMR::Right { -1.0 } else { 1.0 },
            ))
            .normalize()
            * b_mag;
        let a2 = d - b;
        let mut theta = a1.angle_between(a2);
        if direction == LMR::Left && theta < 0.0 {
            theta += TAU;
        } else if direction == LMR::Right && theta > 0.0 {
            theta -= TAU;
        }
        Self(VecDeque::from([
            HorPlanItem::Turn(theta.into()),
            HorPlanItem::Straight(b_mag),
        ]))
    }
    #[must_use]
    pub fn plan_to_pos2angle(_from: Pos2Angle, _to: Pos2Angle) -> Self {
        todo!()
    }
    #[must_use]
    pub fn calc_positions(&self, _cfg: &Config) -> VecDeque<Pos2Angle> {
        todo!()
    }
    #[must_use]
    pub fn end(&self, from: Pos2Angle, mm: ModelMotion) -> Pos2Angle {
        let mut pos = from;
        for i in &self.0 {
            match i {
                HorPlanItem::Turn(angle) => {
                    let Some(turning_rot) = angle.turning_rot() else {
                        continue;
                    };
                    let a1 = pos.1.vec().perp_rot(turning_rot) * -mm.turning_radius;
                    let a2 = a1.rotate(angle.vec());
                    pos.0 = pos.0 - a1 + a2;
                    pos.1 += *angle;
                }
                HorPlanItem::Straight(d) => {
                    pos.0 += pos.1.vec() * *d;
                }
            }
        }
        pos.1 = pos.1.clamp();
        pos
    }
}

#[must_use]
pub fn plan_to_ver(_from: f32, _to: f32) -> VecDeque<f32> {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::f32::consts::{FRAC_PI_2, PI};

    use itertools::assert_equal;
    use proptest::prelude::*;

    use crate::ty::{
        angle::Angle,
        pos::{HorPlanItem, HorPlanner, Pos2Angle},
        world_data::ModelMotion,
        Pos2,
    };

    prop_compose! {
        fn arb_pos()(
            x in -1e5..1e5f32,
            y in -1e5..1e5f32,
        ) -> Pos2 {
            Pos2::new(x, y)
        }
    }

    fn mm(turning_radius: f32) -> ModelMotion {
        ModelMotion {
            turning_radius,
            ..Default::default()
        }
    }

    proptest! {
        #[test]
        #[allow(clippy::float_cmp)]
        fn plan_to_pos2_doesnt_crash(a in arb_pos(), b in arb_pos(), c: f32, d in 0.5f32..1e5f32) {
            let hp = HorPlanner::plan_to_pos2(
                Pos2Angle(a, Angle(c)),
                b,
                mm(d),
            );
            dbg!(b - hp.end(Pos2Angle(a, Angle(c)), mm(d)).0, b - a);
            for i in hp.0 {
                match i {
                    HorPlanItem::Straight(i) => assert!(!i.is_nan()),
                    HorPlanItem::Turn(i) => assert!(!i.0.is_nan())
                }
            }
        }
    }

    #[test]
    fn plan_to_pos2_90deg() {
        let res = HorPlanner::plan_to_pos2(
            Pos2Angle(Pos2::new(0.0, 0.0), Angle(FRAC_PI_2)),
            Pos2::new(10.0, 1.0),
            mm(1.0),
        );
        assert_eq!(res.len(), 2);
        let HorPlanItem::Turn(Angle(theta)) = res[0] else {
            panic!("Not HorPlanItem::Turn")
        };
        assert!((theta + FRAC_PI_2) < f32::EPSILON * 10.0);
        assert_eq!(res[1], HorPlanItem::Straight(9.0));
    }

    #[test]
    fn plan_to_pos2_180deg() {
        let res = HorPlanner::plan_to_pos2(
            Pos2Angle(Pos2::new(0.0, 0.0), Angle(FRAC_PI_2)),
            Pos2::new(2.0, -10.0),
            mm(1.0),
        );
        assert_eq!(res.len(), 2);
        let HorPlanItem::Turn(Angle(theta)) = res[0] else {
            panic!("Not HorPlanItem::Turn")
        };
        assert!((theta + PI) < f32::EPSILON * 10.0);
        assert_eq!(res[1], HorPlanItem::Straight(10.0));
    }

    #[test]
    fn plan_to_pos2_straight() {
        let res = HorPlanner::plan_to_pos2(
            Pos2Angle(Pos2::new(0.0, 0.0), Angle(0.0)),
            Pos2::new(10.0, 0.0),
            mm(1.0),
        );
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], HorPlanItem::Straight(10.0));
    }
    #[test]
    fn plan_to_pos2_opp() {
        let res = HorPlanner::plan_to_pos2(
            Pos2Angle(Pos2::new(0.0, 0.0), Angle(FRAC_PI_2)),
            Pos2::new(1.0, 0.0),
            mm(1.0),
        );
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], HorPlanItem::Turn(Angle(5.235_988)));
        assert_eq!(res[1], HorPlanItem::Straight(1.732_050_8));
        dbg!(
            res.end(Pos2Angle(Pos2::new(0.0, 0.0), Angle(FRAC_PI_2)), mm(1.0))
                .0
        );
    }
}
