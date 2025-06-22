use bytes::Bytes;
use tracing::{error, info, warn};

use crate::{config::Config, state::State, util::PlaneStateId, world_data::WorldData};

#[derive(Clone, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub struct Engine {
    pub world: WorldData,
    pub config: Config,
    pub state: State,
}

impl Engine {
    #[must_use]
    pub fn new(world: WorldData, config: Config) -> Self {
        if let Some(old_engine) = config
            .save_path
            .as_ref()
            .and_then(|p| {
                std::fs::read(p)
                    .inspect_err(|e| error!(in_ = "load", "{e:#}"))
                    .ok()
            })
            .and_then(|b| {
                rkyv::from_bytes::<Self, rkyv::rancor::Error>(&b)
                    .inspect_err(|e| error!(in_ = "load", "{e:#}"))
                    .ok()
            })
            .and_then(|ng| {
                if ng.world == world {
                    Some(ng)
                } else {
                    warn!(in_ = "load", "ignoring old state due to new world data");
                    None
                }
            })
        {
            info!(path=?config.save_path.as_ref().unwrap(), "Loading old engine");
            Self {
                config,
                ..old_engine
            }
        } else {
            Self {
                state: State::new(&world.airports),
                world,
                config,
            }
        }
    }
    pub fn tick(&mut self) -> (Vec<PlaneStateId>, Bytes) {
        self.state.tick(&self.config, &self.world)
    }
}
