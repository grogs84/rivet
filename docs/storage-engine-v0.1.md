# Rivet v0.1 — Durable Graph Storage Engine

## Goal

Build the smallest useful graph storage engine that proves Rivet can persist graph topology to disk and recover it correctly after restart.

The first milestone deliberately ignores query languages, properties, labels, schemas, indexes, transactions, and distributed concerns. The focus is the storage layer itself.

A successful v0.1 should make this kind of workflow possible:

```rust
let mut db = Rivet::open("test.rivet")?;

let a = db.add_vertex()?;
let b = db.add_vertex()?;
let c = db.add_vertex()?;

db.add_edge(a, b)?;
db.add_edge(a, c)?;

drop(db);

let db = Rivet::open("test.rivet")?;
assert_eq!(db.out_neighbors(a)?, vec![b, c]);
```

## Why this is the right first milestone

This establishes the lowest layer that is both database-specific and graph-specific:

- stable vertex and edge identifiers
- an on-disk record layout
- deterministic record lookup
- persistent adjacency representation
- file creation and reopening
- graph traversal without loading the entire graph into memory

Once this works, later features such as paging, caching, crash recovery, properties, indexing, and query execution can be built on a concrete storage foundation.

## First implementation slice

Start with the simplest correct format possible.

### Core types

Introduce strongly typed identifiers:

```rust
pub struct VertexId(pub u64);
pub struct EdgeId(pub u64);
```

Define minimal fixed-size records:

```rust
struct VertexRecord {
    first_out_edge: Option<EdgeId>,
}

struct EdgeRecord {
    source: VertexId,
    target: VertexId,
    next_out_edge: Option<EdgeId>,
}
```

The initial adjacency structure can be a linked list of edge records on disk. A vertex points to the first outgoing edge; each edge points to the next outgoing edge for the same source vertex.

This is intentionally simple rather than optimized.

## Proposed implementation plan

### 1. Establish the public storage API

Create a small `Rivet` or `GraphStore` type with:

- `open(path)`
- `add_vertex()`
- `add_edge(source, target)`
- `out_neighbors(vertex)`

Keep the API free of query-language concepts.

### 2. Define a minimal file format

Use one database file initially.

Suggested layout:

```text
+------------------+
| file header      |
+------------------+
| vertex records   |
+------------------+
| edge records     |
+------------------+
```

The header should contain enough metadata to reopen the file, such as:

- magic bytes identifying a Rivet file
- file-format version
- vertex count
- edge count
- offsets for the vertex and edge regions

Prefer explicit serialization/deserialization over writing Rust structs directly with unsafe memory casts.

### 3. Implement fixed-size vertex records

Each vertex gets a monotonically increasing `VertexId`.

For v0.1, make record lookup deterministic:

```text
vertex_offset = vertex_region_start + vertex_id * VERTEX_RECORD_SIZE
```

A vertex record only needs to store the first outgoing edge pointer.

### 4. Implement fixed-size edge records

Each edge gets a monotonically increasing `EdgeId`.

An edge record stores:

- source vertex
- target vertex
- next outgoing edge for the source

Use an explicit sentinel value for "no edge" in the serialized form.

### 5. Implement edge insertion

For `add_edge(a, b)`:

1. validate that both vertex IDs exist
2. allocate the next edge ID
3. read vertex `a`
4. create the edge record with its `next_out_edge` pointing to the vertex's previous first edge
5. append/write the edge record
6. update vertex `a` so `first_out_edge` points to the new edge
7. persist updated metadata

Prepending edges keeps insertion simple and avoids walking an adjacency list just to append.

### 6. Implement adjacency traversal

For `out_neighbors(a)`:

1. read vertex `a`
2. follow `first_out_edge`
3. read the edge record
4. collect its target vertex
5. follow `next_out_edge`
6. continue until the sentinel is reached

This should perform disk reads for only the records required by that adjacency list rather than loading the whole graph.

### 7. Support clean reopen

`Rivet::open` should:

- create a new database file if none exists
- validate magic bytes and file-format version for an existing file
- load header metadata
- allow existing vertices and adjacency lists to be read immediately

### 8. Add tests around persistence

Minimum tests:

- create and reopen an empty database
- add one vertex and recover it
- add multiple vertices and preserve IDs
- add one edge and recover it
- add multiple outgoing edges and recover all neighbors
- preserve graph topology across close/reopen
- reject edges referencing nonexistent vertices
- reject invalid/corrupt file headers

Use temporary files/directories so tests do not leave artifacts behind.

### 9. Add a tiny executable example

Provide an example or binary that:

1. creates three vertices
2. creates two edges from the first vertex
3. closes the database
4. reopens it
5. prints the recovered neighbors

This becomes a simple end-to-end smoke test and demonstration.

## Suggested module structure

```text
src/
├── lib.rs
├── id.rs
├── record.rs
├── file.rs
└── store.rs
```

Possible responsibilities:

- `id.rs` — `VertexId`, `EdgeId`
- `record.rs` — fixed-size on-disk record definitions and encoding
- `file.rs` — low-level seek/read/write helpers and file header
- `store.rs` — graph storage API and adjacency operations
- `lib.rs` — public exports

Keep the module boundaries lightweight; v0.1 does not need a large abstraction hierarchy.

## Acceptance criteria

The milestone is complete when:

- Rivet creates a database file
- vertices receive stable IDs
- directed edges receive stable IDs
- topology survives process restart
- outgoing neighbors can be retrieved after reopen
- adjacency traversal does not require loading the full graph
- malformed vertex references return errors rather than panicking
- core persistence behavior is covered by automated tests

## Explicit non-goals for v0.1

Do not add yet:

- Cypher or another query language
- labels
- properties
- secondary indexes
- incoming-edge indexes
- deletions/free-space reuse
- page management
- buffer pools
- transactions
- WAL/crash recovery
- concurrency
- compression
- distributed storage

These become later milestones once the basic durable graph representation is working.

## Follow-on milestones

After v0.1, the natural progression is:

1. page-based storage
2. buffer pool/cache
3. free-space management
4. crash safety and recovery
5. richer graph records/properties
6. indexes
7. transactions/concurrency
8. query execution layers

The guiding principle is to keep the first format intentionally understandable and inspectable. A format that can be reasoned about with a hex dump is preferable to an optimized design that hides the fundamentals too early.
