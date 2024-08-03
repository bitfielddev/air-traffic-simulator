use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub tick_duration: f32,
    pub plane_spawn_chance: f32,
}
