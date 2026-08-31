use std::collections::HashMap;
use std::io;
use std::path::Path;
use thiserror::Error;

use crate::id::{VertexId, EdgeId, SENTINEL_EDGE_ID};
use crate::record::{VertexRecord, EdgeRecord};
use crate::file::FileStore;

#[derive(Error, Debug)]
pub enum RivetError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Invalid vertex ID: {0}")]
    InvalidVertex(u64),
    
    #[error("Invalid edge ID: {0}")]
    InvalidEdge(u64),
    
    #[error("Invalid file format")]
    InvalidFormat,
}

pub type Result<T> = std::result::Result<T, RivetError>;

/// Main graph storage engine.
pub struct Rivet {
    file: FileStore,
    vertex_cache: HashMap<u64, VertexRecord>,
    edge_cache: HashMap<u64, EdgeRecord>,
}

impl Rivet {
    /// Open or create a database file.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = FileStore::open(path)?;
        
        // Validate header if file exists with content
        if file.header.vertex_count > 0 || file.header.edge_count > 0 {
            if !file.header.is_valid() {
                return Err(RivetError::InvalidFormat);
            }
        }

        Ok(Rivet {
            file,
            vertex_cache: HashMap::new(),
            edge_cache: HashMap::new(),
        })
    }

    /// Add a new vertex and return its ID.
    pub fn add_vertex(&mut self) -> Result<VertexId> {
        let vertex_id = self.file.header.vertex_count;
        let record = VertexRecord::new();
        
        self.vertex_cache.insert(vertex_id, record.clone());
        self.file.header.vertex_count += 1;
        self.file.write_vertex(vertex_id, &record)?;
        self.file.write_header()?;
        
        Ok(VertexId(vertex_id))
    }

    /// Add an edge from source to target.
    pub fn add_edge(&mut self, source: VertexId, target: VertexId) -> Result<EdgeId> {
        // Validate vertices exist
        if source.0 >= self.file.header.vertex_count {
            return Err(RivetError::InvalidVertex(source.0));
        }
        if target.0 >= self.file.header.vertex_count {
            return Err(RivetError::InvalidVertex(target.0));
        }

        // Get current first edge for source
        let source_record = self.vertex_cache
            .get(&source.0)
            .cloned()
            .unwrap_or_else(|| {
                self.file.read_vertex(source.0)
                    .unwrap_or_else(|_| VertexRecord::new())
            });

        let edge_id = self.file.header.edge_count;
        let new_edge = EdgeRecord::new(source.0, target.0, source_record.first_edge_id);

        // Write new edge
        self.edge_cache.insert(edge_id, new_edge.clone());
        self.file.write_edge(edge_id, &new_edge)?;

        // Update source vertex to point to new edge
        let mut updated_source = source_record;
        updated_source.first_edge_id = edge_id;
        self.vertex_cache.insert(source.0, updated_source.clone());
        self.file.write_vertex(source.0, &updated_source)?;

        self.file.header.edge_count += 1;
        self.file.write_header()?;

        Ok(EdgeId(edge_id))
    }

    /// Get all outgoing neighbors for a vertex.
    pub fn out_neighbors(&mut self, vertex: VertexId) -> Result<Vec<VertexId>> {
        if vertex.0 >= self.file.header.vertex_count {
            return Err(RivetError::InvalidVertex(vertex.0));
        }

        let vertex_record = self.vertex_cache
            .get(&vertex.0)
            .cloned()
            .unwrap_or_else(|| {
                self.file.read_vertex(vertex.0)
                    .unwrap_or_else(|_| VertexRecord::new())
            });

        let mut neighbors = Vec::new();
        let mut current_edge_id = vertex_record.first_edge_id;

        while current_edge_id != SENTINEL_EDGE_ID {
            let edge = self.edge_cache
                .get(&current_edge_id)
                .cloned()
                .ok_or(RivetError::InvalidEdge(current_edge_id))
                .or_else(|_| {
                    self.file.read_edge(current_edge_id)
                        .map_err(|_| RivetError::InvalidEdge(current_edge_id))
                })?;

            neighbors.push(VertexId(edge.target));
            current_edge_id = edge.next_edge_id;
        }

        Ok(neighbors)
    }

    /// Persist all changes to disk and close.
    pub fn close(mut self) -> Result<()> {
        self.file.flush()?;
        Ok(())
    }
}

impl Drop for Rivet {
    fn drop(&mut self) {
        let _ = self.file.flush();
    }
}
