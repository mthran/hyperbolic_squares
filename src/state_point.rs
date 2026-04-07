use serde::{Deserialize, Serialize};

/// Model parameters that describe a single simulation.
#[derive(Serialize, Deserialize)]
pub struct StatePoint {
    pub n: usize,
    pub packing_fraction: f64,
    pub replicate: u32,
}
