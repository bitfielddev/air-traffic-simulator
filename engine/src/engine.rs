use crate::{config::Config, state::State, world_data::WorldData};

#[derive(Clone)]
pub struct Engine {
    pub world: WorldData,
    pub config: Config,
    pub state: State,
}

impl Engine {
    #[must_use]
    pub fn new(world: WorldData, config: Config) -> Self {
        Self {
            state: State::new(&world.airports),
            world,
            config,
        }
    }
    pub fn tick(&mut self) {
        self.state.tick(&self.config, &self.world)
    }
}
