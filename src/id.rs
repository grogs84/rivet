use serde::{Deserialize, Serialize};
use std::fmt;

/// Strongly-typed vertex identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct VertexId(pub u64);

impl fmt::Display for VertexId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V({})", self.0)
    }
}

/// Strongly-typed edge identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct EdgeId(pub u64);

impl fmt::Display for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "E({})", self.0)
    }
}

pub const SENTINEL_EDGE_ID: u64 = u64::MAX;
