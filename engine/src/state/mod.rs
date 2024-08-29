use std::sync::Arc;

use airport::Airport;
use plane::Plane;
use rand::{prelude::*, Rng};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use tracing::{debug, info};

use crate::{
    config::Config,
    util::{AirportStateId, FlightCode, PlaneStateId},
    world_data::{AirportData, Flight, WorldData},
};

pub mod airport;
pub mod plane;
pub mod plane_pos;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct State {
    pub planes: Vec<Plane>,
    pub airports: Vec<Airport>,
}
impl State {
    #[must_use]
    pub fn new(airports: &[Arc<AirportData>]) -> Self {
        Self {
            planes: Vec::default(),
            airports: airports
                .iter()
                .map(|a| Airport::new(Arc::clone(a)))
                .collect(),
        }
    }
    #[must_use]
    pub fn plane(&self, id: &PlaneStateId) -> Option<&Plane> {
        self.planes.iter().find(|a| a.id == *id)
    }
    #[must_use]
    pub fn plane_mut(&mut self, id: &PlaneStateId) -> Option<&mut Plane> {
        self.planes.iter_mut().find(|a| a.id == *id)
    }
    #[must_use]
    pub fn airport(&self, id: &AirportStateId) -> Option<&Airport> {
        self.airports.iter().find(|a| a.id == *id)
    }
    #[must_use]
    pub fn airport_mut(&mut self, id: &AirportStateId) -> Option<&mut Airport> {
        self.airports.iter_mut().find(|a| a.id == *id)
    }
    #[tracing::instrument(skip_all)]
    pub fn tick(&mut self, config: &Config, wd: &WorldData) -> (Vec<PlaneStateId>, Vec<u8>) {
        let mut remove_list = vec![];
        for (id, (remove, send)) in self
            .planes
            .par_iter_mut()
            .map(|plane| (plane.id, plane.tick(config)))
            .collect::<Vec<_>>()
        {
            if remove {
                info!(%id, "Removing plane");
                remove_list.push(id);
            }
            for (airport, event) in send {
                if let Some(airport) = self.airport_mut(&airport) {
                    debug!(?event, to=%airport.id, "Sending airport event");
                    airport.events.push_back(event);
                }
            }
        }
        self.planes.retain(|plane| !remove_list.contains(&plane.id));

        for send in self
            .airports
            .par_iter_mut()
            .map(|airport| airport.tick(config))
            .collect::<Vec<_>>()
        {
            for (plane, event) in send {
                if let Some(plane) = self.plane_mut(&plane) {
                    debug!(?event, to=%plane.id, "Sending plane event");
                    plane.events.push_back(event);
                }
            }
        }

        if config.max_planes.map_or(true, |m| self.planes.len() < m)
            && thread_rng().gen_range(0.0..=1.0) < config.plane_spawn_chance
        {
            let plane = wd.planes.choose(&mut thread_rng()).unwrap();
            #[expect(clippy::option_if_let_else)]
            let flight = if let Some(flights) = &wd.flights {
                flights.choose(&mut thread_rng()).unwrap()
            } else {
                // TODO check whether plane can land in runway
                &Arc::new(Flight {
                    airline: SmolStr::default(),
                    code: FlightCode::default(),
                    from: wd.airports.choose(&mut thread_rng()).unwrap().code.clone(),
                    to: wd.airports.choose(&mut thread_rng()).unwrap().code.clone(),
                    plane: Arc::new([plane.id.clone()]),
                })
            };
            let runway = self
                .airport(&flight.from)
                .unwrap()
                .airport
                .runways
                .choose(&mut thread_rng())
                .unwrap();
            let plane = Plane::new(plane, flight, runway, wd);
            info!(%plane.id, %plane.model.id, %plane.flight.code, %plane.flight.from, %plane.flight.to, "Creating plane");
            self.planes.push(plane);
        }
        (remove_list, self.coord_state())
    }
    #[must_use]
    pub fn coord_state(&self) -> Vec<u8> {
        self.planes
            .iter()
            .flat_map(|p| {
                p.id.into_bytes()
                    .into_iter()
                    .chain(p.pos.pos_ang.0.x.to_le_bytes())
                    .chain(p.pos.pos_ang.0.y.to_le_bytes())
                    .chain(p.pos.pos_ang.0.z.to_le_bytes())
                    .chain(p.pos.pos_ang.1 .0.to_le_bytes())
                    .chain(p.pos.kinematics.v.x.to_le_bytes())
                    .chain(p.pos.kinematics.v.y.to_le_bytes())
            })
            .collect::<Vec<_>>()
    }
}
