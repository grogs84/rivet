use serde::{Deserialize, Serialize};
use crate::id::{VertexId, EdgeId, SENTINEL_EDGE_ID};

/// On-disk vertex record: stores the ID of the first outgoing edge.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VertexRecord {
    pub first_edge_id: u64,
}

impl VertexRecord {
    pub fn new() -> Self {
        VertexRecord {
            first_edge_id: SENTINEL_EDGE_ID,
        }
    }

    pub fn has_edges(&self) -> bool {
        self.first_edge_id != SENTINEL_EDGE_ID
    }
}

/// On-disk edge record: stores source, target, and pointer to next edge.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeRecord {
    pub source: u64,
    pub target: u64,
    pub next_edge_id: u64,
}

impl EdgeRecord {
    pub fn new(source: u64, target: u64, next_edge_id: u64) -> Self {
        EdgeRecord {
            source,
            target,
            next_edge_id,
        }
    }
}

/// Database header: metadata and offsets.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Header {
    pub magic: [u8; 4],
    pub version: u32,
    pub vertex_count: u64,
    pub edge_count: u64,
    pub vertex_region_offset: u64,
    pub edge_region_offset: u64,
}

impl Header {
    pub fn new() -> Self {
        Header {
            magic: *b"RIVT",
            version: 1,
            vertex_count: 0,
            edge_count: 0,
            vertex_region_offset: 0,
            edge_region_offset: 0,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == *b"RIVT" && self.version == 1
    }
}
