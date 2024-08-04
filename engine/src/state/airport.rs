use std::{collections::VecDeque, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    state::plane::{PlaneEvent, PlaneEventPayload},
    util::{AirportStateId, PlaneStateId},
    world_data::AirportData,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Airport {
    pub id: AirportStateId,
    pub airport: Arc<AirportData>,
    pub events: VecDeque<AirportEvent>,
}

impl Airport {
    #[must_use]
    pub fn new(airport: Arc<AirportData>) -> Self {
        Self {
            id: airport.code.clone(),
            airport,
            events: VecDeque::new(),
        }
    }
    pub fn tick(&mut self, _config: &Config) -> Vec<(PlaneStateId, PlaneEvent)> {
        let mut send = vec![];
        for event in self.events.drain(..) {
            match event.payload {
                AirportEventPayload::RequestRunway => {
                    send.push((
                        event.from,
                        PlaneEvent {
                            from: self.id.clone(),
                            payload: PlaneEventPayload::ClearForLanding(Arc::clone(
                                &self.airport.runways[0],
                            )),
                        },
                    ));
                }
            }
        }
        send
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AirportEvent {
    pub from: PlaneStateId,
    pub payload: AirportEventPayload,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub enum AirportEventPayload {
    RequestRunway,
}
