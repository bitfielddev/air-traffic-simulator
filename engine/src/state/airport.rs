use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    util::{AirportStateId, PlaneStateId},
    world_data::AirportData,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Airport {
    pub id: AirportStateId,
    pub airport: Arc<AirportData>,
    pub events: Arc<RwLock<VecDeque<AirportEvent>>>,
}

impl Airport {
    #[must_use]
    pub fn new(airport: Arc<AirportData>) -> Self {
        Self {
            id: airport.code.clone(),
            airport,
            events: Arc::new(RwLock::default()),
        }
    }
    pub fn tick(&mut self, config: &Config) {}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AirportEvent {
    pub from: PlaneStateId,
    pub payload: AirportEventPayload,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub enum AirportEventPayload {}
