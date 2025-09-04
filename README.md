# ProtoBench

A performance and feature comparison of modern RPC protocols and serialization formats, implemented as identical services in Rust.

## Overview

This project implements a metrics collection service using three different approaches:
- **REST API** with JSON serialization
- **gRPC** with Protocol Buffers
- **Cap'n Proto RPC** with Cap'n Proto serialization

All implementations provide identical functionality to enable fair comparison of protocol characteristics, performance, and developer experience.

## Architecture

The project uses a Cargo workspace with shared business logic across protocol implementations:

```
protobench/
├── shared/           # Common data models and business logic
├── rest-service/     # HTTP/JSON implementation  
├── grpc-service/     # gRPC/Protobuf implementation
├── capnp-service/    # Cap'n Proto RPC implementation
├── benchmarks/       # Performance testing and analysis
└── schemas/          # Protocol definitions
```

## Service API

Each service implements the same metrics collection interface:

- `submit_metric` - Accept system metrics data points
- `query_metrics` - Retrieve metrics by time range
- `get_statistics` - Calculate aggregated metric statistics

## Data Model

```rust
MetricPoint {
    timestamp: i64,
    hostname: String,
    cpu_percent: f32,
    memory_bytes: u64, 
    disk_io_ops: u32,
    tags: HashMap<String, String>
}
```

## Comparison Goals

This benchmark evaluates:

1. **Serialization Efficiency** - Payload size and encoding/decoding performance
2. **Type Safety** - Compile-time guarantees vs runtime validation
3. **Schema Evolution** - Adding fields and maintaining backward compatibility  
4. **Network Performance** - Throughput, latency, and connection overhead
5. **Developer Experience** - Code generation, debugging, tooling support
6. **Protocol Features** - Streaming, multiplexing, error handling

## Getting Started

```bash
# Build all services
cargo build --release

# Run individual services
cargo run --bin rest-service
cargo run --bin grpc-service  
cargo run --bin capnp-service

# Execute benchmarks
cargo run --bin benchmarks
```

## Results

Benchmark results and analysis are generated in `benchmarks/results/` with detailed performance characteristics and trade-off analysis for each protocol approach.
