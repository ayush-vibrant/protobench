# ProtoBench

A performance and feature comparison of modern RPC protocols and serialization formats, implemented as identical services in Rust.

## Overview

This project implements a metrics collection service using three different approaches:
- **REST API** with JSON serialization
- **gRPC** with Protocol Buffers
- **Cap'n Proto RPC** with Cap'n Proto serialization

All implementations provide identical functionality to enable fair comparison of protocol characteristics, performance, and developer experience.

## Architecture

### Workspace Design

The project uses a **Cargo workspace** pattern that promotes code reuse while enabling fair protocol comparison:

```
protobench/
├── shared/           # Business logic foundation
├── schemas/          # Protocol contract definitions
├── rest-service/     # HTTP/JSON implementation
├── grpc-service/     # gRPC/Protobuf implementation
├── capnp-service/    # Cap'n Proto RPC implementation
├── benchmarks/       # Performance testing harness
└── analysis/         # Results processing & visualization
```

### Component Responsibilities

#### **1. `shared/` - Business Logic Foundation**
**Responsibility**: Provides protocol-agnostic business logic and data models

**Key Contents**:
- `MetricPoint`, `MetricQuery`, `MetricStatistics` structs
- `MetricsService` trait defining the service contract
- `InMemoryStorage` - shared storage backend
- Common utilities and error handling

**Design Impact**: Ensures **identical business logic** across all three implementations, eliminating implementation bias in benchmarks

#### **2. `schemas/` - Contract Definitions**  
**Responsibility**: Protocol-specific schema definitions for the same logical API

**Key Contents**:
- `metrics.proto` - gRPC Protocol Buffers definition
- `metrics.capnp` - Cap'n Proto schema definition
- `openapi.yaml` - REST API specification

**Design Impact**: Demonstrates **contract-first development** approach and enables direct comparison of schema expressiveness

#### **3. Service Implementations (REST/gRPC/Cap'n Proto)**
**Responsibility**: Protocol-specific server implementations using identical business logic

**Each service provides**:
- `submit_metric` - Accept metrics data
- `query_metrics` - Retrieve time-ranged metrics  
- `get_statistics` - Calculate aggregated stats

**Design Impact**: **Fair comparison baseline** - same functionality, different protocols

#### **4. `benchmarks/` - Performance Testing Harness**
**Responsibility**: Comprehensive performance measurement across all protocols

**Key Components**:
- **Client implementations** for each protocol (`rest_client.rs`, `grpc_client.rs`, `capnp_client.rs`)
- **Criterion-based benchmarking** for statistical rigor
- **Load testing scenarios** with varying data sizes and concurrent connections

**Design Impact**: Provides **empirical data** for protocol trade-off analysis

#### **5. `analysis/` - Results Processing & Visualization**
**Responsibility**: Transform raw benchmark data into actionable insights

**Key Components**:
- Python analysis scripts for data processing
- Chart generation for visual comparisons
- Statistical analysis of performance characteristics

**Design Impact**: Converts raw performance data into **comparative insights**

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

## Benchmarking Strategy

### 🎯 Common Task - Identical Metrics Collection Service

**The Universal API**:
All three protocols implement the same metrics collection operations:

1. **`submit_metric`** - Accept system monitoring data points
2. **`query_metrics`** - Time-ranged retrieval with optional hostname filtering  
3. **`get_statistics`** - Calculate aggregated metrics (averages, counts)

**Standardized Test Data**:
- **MetricPoint**: CPU %, memory usage, disk I/O, hostname, tags
- **Deterministic generation**: Seeded RNG (seed=42) for reproducible results
- **Realistic scenarios**: 10 hostnames, multiple environments/regions/services
- **Variable complexity**: 1x to 50x payload size factors for scalability testing

### 📊 Metrics Being Measured

**Primary Performance Metrics**:

1. **Latency Measurements**:
   - Average request latency (milliseconds)
   - Min/max latency distribution
   - P50, P90, P99 percentiles (via Criterion.rs)

2. **Throughput Metrics**:
   - Requests per second (RPS)
   - Peak concurrent request handling
   - Burst load capacity

3. **Scalability Characteristics**:
   - Payload size impact (1x to 50x complexity)
   - Concurrency scaling (1 to 20+ concurrent requests)
   - Performance degradation points

4. **Reliability Metrics**:
   - Success rate under load
   - Error rate analysis
   - Connection failure handling

**Secondary Analysis Dimensions**:
- **Serialization efficiency** (JSON vs Protobuf vs Cap'n Proto)
- **Network utilization** (HTTP/1.1 vs HTTP/2 vs custom TCP)
- **Memory consumption** patterns
- **CPU utilization** during processing

### 🔬 Comparison Methodology

**Multi-Layered Benchmarking Approach**:

1. **Criterion.rs Statistical Benchmarking**:
   - Micro-benchmarks with statistical confidence intervals
   - Automatic outlier detection and warm-up periods
   - Protocol-specific client implementations

2. **Python Load Testing Suite**:
   - Macro-benchmarks simulating real-world usage
   - Concurrent request handling analysis
   - Tipping point identification

3. **Controlled Variables**:
   - **Same business logic**: Shared Rust library ensures identical processing
   - **Same data model**: Consistent MetricPoint across all protocols
   - **Same test data**: Deterministic generation for fair comparison
   - **Same hardware**: All tests run on identical infrastructure

**Visualization Strategy**:
- **Performance curves** showing scaling characteristics
- **Tipping point charts** identifying where protocols excel/struggle
- **Comparative dashboards** with side-by-side metrics
- **Heat maps** for concurrency vs payload size analysis

### 🎯 Expected Results & Insights

**Anticipated Performance Patterns**:

1. **REST/JSON Expectations**:
   - **Strengths**: Simple payloads, debugging ease, universal compatibility
   - **Weaknesses**: JSON parsing overhead, HTTP/1.1 limitations, large payload inefficiency
   - **Tipping Point**: Performance degrades with payload complexity and high concurrency

2. **gRPC/Protobuf Expectations**:
   - **Strengths**: Binary efficiency, HTTP/2 multiplexing, schema validation, streaming
   - **Weaknesses**: Setup complexity, debugging difficulty
   - **Tipping Point**: Excels at high concurrency and complex data structures

3. **Cap'n Proto Expectations**:
   - **Strengths**: Zero-copy deserialization, minimal CPU overhead
   - **Weaknesses**: Implementation complexity, limited ecosystem, threading constraints
   - **Tipping Point**: Best for memory-constrained or ultra-low-latency scenarios

**Key Research Questions**:
- Where does JSON parsing become a bottleneck?
- At what concurrency level does HTTP/2 multiplexing show clear advantages?
- What payload size makes binary serialization worthwhile?
- How significant is the zero-copy advantage of Cap'n Proto?

**Practical Decision Framework**:
The benchmarking aims to provide decision criteria like:
- "Use REST for < X concurrent requests with < Y payload size"
- "Switch to gRPC when concurrency > Z or payload complexity > W"  
- "Consider Cap'n Proto for memory-constrained environments with < V setup complexity tolerance"

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
