use std::{cmp::Ordering, sync::Arc};

use airport::Airport;
use plane::Plane;
use rand::{prelude::*, Rng};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use crate::{
    config::Config,
    util::{FlightCode, PlaneStateId},
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
    pub fn airport(&self, id: &PlaneStateId) -> Option<&Airport> {
        self.airports.iter().find(|a| a.id == *id)
    }
    #[must_use]
    pub fn airport_mut(&mut self, id: &PlaneStateId) -> Option<&mut Airport> {
        self.airports.iter_mut().find(|a| a.id == *id)
    }
    pub fn tick(&mut self, config: &Config, wd: &WorldData) {
        let mut remove_list = vec![];
        for (id, (remove, send)) in self
            .planes
            .par_iter_mut()
            .map(|plane| (plane.id.clone(), plane.tick(config)))
            .collect::<Vec<_>>()
        {
            if remove {
                remove_list.push(id);
            }
            for (airport, event) in send {
                if let Some(airport) = self.airport_mut(&airport) {
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
                    plane.events.push_back(event);
                }
            }
        }

        if thread_rng().gen_range(0.0..=1.0) < config.plane_spawn_chance {
            let plane = wd.planes.choose(&mut thread_rng()).unwrap();
            let flight = if let Some(flights) = &wd.flights {
                flights.choose(&mut thread_rng()).unwrap()
            } else {
                // TODO check whether plane can land in runway
                &Arc::new(Flight {
                    airline: SmolStr::default(),
                    code: FlightCode::default(),
                    from: wd
                        .airports
                        .choose(&mut thread_rng())
                        .unwrap()
                        .code
                        .to_owned(),
                    to: wd
                        .airports
                        .choose(&mut thread_rng())
                        .unwrap()
                        .code
                        .to_owned(),
                    plane: Arc::new([plane.id.to_owned()]),
                })
            };
            let runway = self
                .airport(&flight.from)
                .unwrap()
                .airport
                .runways
                .choose(&mut thread_rng())
                .unwrap();
            self.planes.push(Plane::new(plane, flight, runway, wd))
        }
    }
}
