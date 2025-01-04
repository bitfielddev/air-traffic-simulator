use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Config {
    #[serde(default)]
    pub tick_duration: f32,
    #[serde(default)]
    pub plane_spawn_chance: f32,
    #[serde(default)]
    pub max_planes: Option<usize>,
    #[serde(default)]
    pub cruising_altitude: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tick_duration: 1.0,
            plane_spawn_chance: 0.05,
            max_planes: None,
            cruising_altitude: 512.0,
        }
    }
}
