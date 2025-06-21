use std::{cmp::Ordering, path::PathBuf};

use glam::Vec2;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(
    Clone, Debug, Serialize, Deserialize, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive, TS,
)]
#[ts(export)]
pub struct Config {
    pub tick_duration: f32,
    pub plane_spawn_chance: f32,
    pub max_planes: Option<usize>,
    pub cruising_altitude_plus: f32,
    pub cruising_altitude_minus: f32,
    pub ns_before_ew: bool,
    #[rkyv(with = rkyv::with::Map<rkyv::with::AsString>)]
    pub save_dir: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tick_duration: 1.0,
            plane_spawn_chance: 0.05,
            max_planes: None,
            cruising_altitude_plus: 1024.0,
            cruising_altitude_minus: 512.0,
            ns_before_ew: false,
            save_dir: None,
        }
    }
}

impl Config {
    #[must_use]
    pub fn cruising_altitude(&self, from: Vec2, to: Vec2) -> f32 {
        if from == to {
            return self.min_cruising_altitude();
        }
        let ew = from.x.total_cmp(&to.x);
        let ns = from.y.total_cmp(&to.y);
        if self.ns_before_ew {
            if ns == Ordering::Less || (ns == Ordering::Equal && ew == Ordering::Less) {
                self.cruising_altitude_plus
            } else {
                self.cruising_altitude_minus
            }
        } else if ew == Ordering::Less || (ew == Ordering::Equal && ns == Ordering::Less) {
            self.cruising_altitude_plus
        } else {
            self.cruising_altitude_minus
        }
    }
    #[must_use]
    pub const fn min_cruising_altitude(&self) -> f32 {
        self.cruising_altitude_plus
            .min(self.cruising_altitude_minus)
    }
}
