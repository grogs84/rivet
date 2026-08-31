use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;
use bincode;
use crate::record::{Header, VertexRecord, EdgeRecord};
use crate::id::SENTINEL_EDGE_ID;

/// Low-level file I/O for the graph storage.
pub struct FileStore {
    file: File,
    pub header: Header,
}

impl FileStore {
    /// Open or create a database file.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        let file_exists = path.exists();

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let header = if file_exists && file.metadata()?.len() > 0 {
            // Read existing header
            file.seek(SeekFrom::Start(0))?;
            let mut buf = vec![0u8; 256]; // Assume header fits in 256 bytes
            let n = file.read(&mut buf)?;
            buf.truncate(n);
            bincode::deserialize(&buf).map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("Failed to deserialize header: {}", e))
            })?
        } else {
            // Create new header
            let mut header = Header::new();
            // Write initial header
            file.seek(SeekFrom::Start(0))?;
            let encoded = bincode::serialize(&header).map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("Failed to serialize header: {}", e))
            })?;
            file.write_all(&encoded)?;
            header.vertex_region_offset = encoded.len() as u64;
            header.edge_region_offset = header.vertex_region_offset;
            header
        };

        Ok(FileStore { file, header })
    }

    /// Write the header to disk.
    pub fn write_header(&mut self) -> io::Result<()> {
        self.file.seek(SeekFrom::Start(0))?;
        let encoded = bincode::serialize(&self.header).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Failed to serialize header: {}", e))
        })?;
        self.file.write_all(&encoded)?;
        Ok(())
    }

    /// Write a vertex record at the given vertex ID.
    pub fn write_vertex(&mut self, vertex_id: u64, record: &VertexRecord) -> io::Result<()> {
        let offset = self.header.vertex_region_offset + vertex_id * 16; // Rough estimate
        self.file.seek(SeekFrom::Start(offset))?;
        let encoded = bincode::serialize(record).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Failed to serialize vertex: {}", e))
        })?;
        self.file.write_all(&encoded)?;
        Ok(())
    }

    /// Read a vertex record at the given vertex ID.
    pub fn read_vertex(&mut self, vertex_id: u64) -> io::Result<VertexRecord> {
        let offset = self.header.vertex_region_offset + vertex_id * 16;
        self.file.seek(SeekFrom::Start(offset))?;
        let mut buf = vec![0u8; 16];
        let n = self.file.read(&mut buf)?;
        if n == 0 {
            return Ok(VertexRecord::new());
        }
        buf.truncate(n);
        bincode::deserialize(&buf).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Failed to deserialize vertex: {}", e))
        })
    }

    /// Write an edge record at the given edge ID.
    pub fn write_edge(&mut self, edge_id: u64, record: &EdgeRecord) -> io::Result<()> {
        let offset = self.header.edge_region_offset + edge_id * 32; // Rough estimate
        self.file.seek(SeekFrom::Start(offset))?;
        let encoded = bincode::serialize(record).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Failed to serialize edge: {}", e))
        })?;
        self.file.write_all(&encoded)?;
        Ok(())
    }

    /// Read an edge record at the given edge ID.
    pub fn read_edge(&mut self, edge_id: u64) -> io::Result<EdgeRecord> {
        if edge_id == SENTINEL_EDGE_ID {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Cannot read sentinel edge"));
        }
        let offset = self.header.edge_region_offset + edge_id * 32;
        self.file.seek(SeekFrom::Start(offset))?;
        let mut buf = vec![0u8; 32];
        let n = self.file.read(&mut buf)?;
        buf.truncate(n);
        bincode::deserialize(&buf).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("Failed to deserialize edge: {}", e))
        })
    }

    /// Sync to disk.
    pub fn flush(&mut self) -> io::Result<()> {
        self.file.sync_all()
    }
}
