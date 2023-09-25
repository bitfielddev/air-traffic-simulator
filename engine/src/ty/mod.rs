use glam::{Vec2, Vec3};
use smol_str::SmolStr;

mod angle;
pub mod config;
pub mod direction;
pub mod engine;
pub mod pos;
pub mod ray;
pub mod state;
pub mod world_data;

pub type AirportCode = SmolStr;
pub type FlightCode = SmolStr;
pub type Pos2 = Vec2;
pub type Pos3 = Vec3;
pub type Class = SmolStr;
pub type PlaneModelId = SmolStr;
pub type WaypointId = SmolStr;
pub type Timestamp = u64;
