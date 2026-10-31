//! Hippocampus: topology, projection and activation math for the thought graph.
//! The browser worker mirrors `physics`; Rust stays the source of truth.

pub mod physics;
pub mod projection;
pub mod telemetry;

pub use physics::{Link, Node, Params, Simulation};
pub use projection::{project, Input, Region};
pub use telemetry::{Activation, Pulse};
