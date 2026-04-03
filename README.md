# Architecture

[ Producers ] → [ Ingest Layer ] → [ Core Queue ] → [ Delivery Layer ] → [ Consumers ]

## Ingest Layer

(HTTP/TCP)

Responsibility:

Accept incoming messages
Minimal parsing
Push into internal queue ASAP

Design:

Async server
Avoid blocking at all costs
Batch requests if possible

## Core Queue

(lock-free)

Store messages in memory
Handle high-throughput push/pop

use bytes::Bytes

zero-copy cloning
cheap to pass around
used in high-perf systems

## Delivery Layer

(batching)

Responsibility:

Pull messages from queue
Send to consumers efficiently

Design:

Batch messages before sending
Avoid per-message overhead

# Project Structure

src/
├── main.rs
├── api/
│ ├── push.rs
│ ├── pull.rs
│
├── queue/
│ ├── mod.rs
│ ├── in_memory.rs
│
├── worker/
│ ├── dispatcher.rs
│
├── types/
│ ├── message.rs
│
└── metrics/

## Crates

### Core

tokio → async runtime
axum → HTTP API
bytes → zero-copy buffers

### Concurrency / Queue

crossbeam → lock-free structures (later)
tokio::sync → for MVP

#### Serialization

serde + serde_json → start simple
👉 later: replace with binary

### Performance / Utilities

parking_lot → faster locks (if needed)
smallvec → avoid heap allocations in small batches

### Benchmarking

criterion → micro-benchmarks
OR custom load tester (better for your case)
