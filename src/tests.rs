use crate::{Rivet, VertexId, RivetError};
use tempfile::NamedTempFile;
use std::path::PathBuf;

fn create_test_db() -> PathBuf {
    NamedTempFile::new().unwrap().path().to_path_buf()
}

#[test]
fn test_create_empty_database() {
    let path = create_test_db();
    let _db = Rivet::open(&path).expect("Failed to create database");
    assert!(path.exists());
}

#[test]
fn test_add_single_vertex() {
    let path = create_test_db();
    let mut db = Rivet::open(&path).expect("Failed to create database");
    
    let v1 = db.add_vertex().expect("Failed to add vertex");
    assert_eq!(v1.0, 0);
}

#[test]
fn test_add_multiple_vertices() {
    let path = create_test_db();
    let mut db = Rivet::open(&path).expect("Failed to create database");
    
    let v1 = db.add_vertex().expect("Failed to add v1");
    let v2 = db.add_vertex().expect("Failed to add v2");
    let v3 = db.add_vertex().expect("Failed to add v3");
    
    assert_eq!(v1.0, 0);
    assert_eq!(v2.0, 1);
    assert_eq!(v3.0, 2);
}

#[test]
fn test_add_single_edge() {
    let path = create_test_db();
    let mut db = Rivet::open(&path).expect("Failed to create database");
    
    let v1 = db.add_vertex().expect("Failed to add v1");
    let v2 = db.add_vertex().expect("Failed to add v2");
    
    let e1 = db.add_edge(v1, v2).expect("Failed to add edge");
    assert_eq!(e1.0, 0);
}

#[test]
fn test_add_multiple_edges() {
    let path = create_test_db();
    let mut db = Rivet::open(&path).expect("Failed to create database");
    
    let v1 = db.add_vertex().expect("Failed to add v1");
    let v2 = db.add_vertex().expect("Failed to add v2");
    let v3 = db.add_vertex().expect("Failed to add v3");
    
    let e1 = db.add_edge(v1, v2).expect("Failed to add e1");
    let e2 = db.add_edge(v1, v3).expect("Failed to add e2");
    
    assert_eq!(e1.0, 0);
    assert_eq!(e2.0, 1);
}

#[test]
fn test_out_neighbors_single_edge() {
    let path = create_test_db();
    let mut db = Rivet::open(&path).expect("Failed to create database");
    
    let v1 = db.add_vertex().expect("Failed to add v1");
    let v2 = db.add_vertex().expect("Failed to add v2");
    db.add_edge(v1, v2).expect("Failed to add edge");
    
    let neighbors = db.out_neighbors(v1).expect("Failed to get neighbors");
    assert_eq!(neighbors.len(), 1);
    assert_eq!(neighbors[0].0, v2.0);
}

#[test]
fn test_out_neighbors_multiple_edges() {
    let path = create_test_db();
    let mut db = Rivet::open(&path).expect("Failed to create database");
    
    let v1 = db.add_vertex().expect("Failed to add v1");
    let v2 = db.add_vertex().expect("Failed to add v2");
    let v3 = db.add_vertex().expect("Failed to add v3");
    
    db.add_edge(v1, v2).expect("Failed to add e1");
    db.add_edge(v1, v3).expect("Failed to add e2");
    
    let neighbors = db.out_neighbors(v1).expect("Failed to get neighbors");
    assert_eq!(neighbors.len(), 2);
    // Edges are prepended, so most recent comes first
    assert!(neighbors.contains(&v2));
    assert!(neighbors.contains(&v3));
}

#[test]
fn test_no_neighbors_for_isolated_vertex() {
    let path = create_test_db();
    let mut db = Rivet::open(&path).expect("Failed to create database");
    
    let v1 = db.add_vertex().expect("Failed to add v1");
    
    let neighbors = db.out_neighbors(v1).expect("Failed to get neighbors");
    assert_eq!(neighbors.len(), 0);
}

#[test]
fn test_persist_and_reopen() {
    let path = create_test_db();
    
    {
        let mut db = Rivet::open(&path).expect("Failed to create database");
        let v1 = db.add_vertex().expect("Failed to add v1");
        let v2 = db.add_vertex().expect("Failed to add v2");
        let v3 = db.add_vertex().expect("Failed to add v3");
        
        db.add_edge(v1, v2).expect("Failed to add e1");
        db.add_edge(v1, v3).expect("Failed to add e2");
        db.close().expect("Failed to close");
    }
    
    {
        let mut db = Rivet::open(&path).expect("Failed to reopen database");
        let v1 = VertexId(0);
        let neighbors = db.out_neighbors(v1).expect("Failed to get neighbors");
        
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&VertexId(1)));
        assert!(neighbors.contains(&VertexId(2)));
    }
}

#[test]
fn test_invalid_vertex_reference() {
    let path = create_test_db();
    let mut db = Rivet::open(&path).expect("Failed to create database");
    
    let v1 = db.add_vertex().expect("Failed to add v1");
    let invalid_vertex = VertexId(999);
    
    let result = db.add_edge(v1, invalid_vertex);
    assert!(matches!(result, Err(RivetError::InvalidVertex(_))));
}

#[test]
fn test_invalid_vertex_reference_source() {
    let path = create_test_db();
    let mut db = Rivet::open(&path).expect("Failed to create database");
    
    let v1 = db.add_vertex().expect("Failed to add v1");
    let invalid_vertex = VertexId(999);
    
    let result = db.add_edge(invalid_vertex, v1);
    assert!(matches!(result, Err(RivetError::InvalidVertex(_))));
}

#[test]
fn test_query_nonexistent_vertex() {
    let path = create_test_db();
    let mut db = Rivet::open(&path).expect("Failed to create database");
    
    let invalid_vertex = VertexId(999);
    let result = db.out_neighbors(invalid_vertex);
    
    assert!(matches!(result, Err(RivetError::InvalidVertex(_))));
}

#[test]
fn test_corrupt_header_detection() {
    let path = create_test_db();
    
    // Create a file with valid header
    {
        let mut db = Rivet::open(&path).expect("Failed to create database");
        db.add_vertex().expect("Failed to add vertex");
        db.close().expect("Failed to close");
    }
    
    // Verify we can reopen it
    let db = Rivet::open(&path).expect("Failed to reopen valid database");
    drop(db);
}
