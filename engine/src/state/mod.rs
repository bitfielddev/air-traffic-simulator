use std::sync::Arc;

use airport::Airport;
use plane::Plane;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{config::Config, util::PlaneStateId, world_data::AirportData};

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
    pub fn tick(&mut self, config: &Config) {
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
    }
}
