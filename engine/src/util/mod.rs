use glam::{Vec2, Vec3};
use smol_str::SmolStr;
use uuid::Uuid;

pub mod angle;
pub mod direction;
pub mod kinematics;
pub mod pos;
pub mod ray;

pub type AirportCode = SmolStr;
pub type FlightCode = SmolStr;
pub type Pos2 = Vec2;
pub type Pos3 = Vec3;
pub type Class = SmolStr;
pub type PlaneModelId = SmolStr;
pub type WaypointId = SmolStr;
pub type PlaneStateId = Uuid;
pub type AirportStateId = SmolStr;
