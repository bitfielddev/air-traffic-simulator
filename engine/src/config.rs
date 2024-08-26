use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub tick_duration: f32,
    #[serde(default)]
    pub plane_spawn_chance: f32,
    #[serde(default)]
    pub max_planes: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tick_duration: 1.0,
            plane_spawn_chance: 0.05,
            max_planes: Some(1),
        }
    }
}
