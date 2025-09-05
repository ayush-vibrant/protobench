# Rust Protocol Buffer Learning Notes
*Personal learning notes for transitioning from Spring Boot CRUD development to Rust*

## How to Write REST APIs in Rust

### Framework Overview: Axum vs Spring Boot
Axum is Rust's equivalent to Spring Boot for building REST APIs, but with key philosophical differences:

- **Spring Boot**: Convention over configuration, runtime dependency injection, annotation-based
- **Rust/Axum**: Explicit configuration, compile-time state management, function-based handlers

### Basic Setup and Entry Point

**Spring Boot:**
```java
@SpringBootApplication
public class Application {
    public static void main(String[] args) {
        SpringApplication.run(Application.class, args);
    }
}
```

**Rust/Axum:**
```rust
#[tokio::main]  // Sets up async runtime
async fn main() -> anyhow::Result<()> {
    let storage = Arc::new(InMemoryStorage::new());
    let app_state = Arc::new(AppState { storage });

    let app = Router::new()
        .route("/metrics", post(submit_metric).get(query_metrics))
        .route("/statistics", get(get_statistics))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

### State Management (Dependency Injection Alternative)

**Spring Boot:**
```java
@Service
public class MetricsService {
    @Autowired
    private InMemoryStorage storage;
}
```

**Rust/Axum:**
```rust
struct AppState {
    storage: Arc<InMemoryStorage>,  // Arc = Atomic Reference Counting for shared ownership
}

// Inject state into handlers
async fn submit_metric(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Result<StatusCode, StatusCode> { ... }
```

**Key Insight**: Rust doesn't have runtime DI. Use `Arc<T>` for shared state across async handlers - it's like manually managed application context.

### Route Mapping and HTTP Methods

**Spring Boot:**
```java
@RestController
@RequestMapping("/metrics")
public class MetricsController {
    @PostMapping
    public ResponseEntity<Void> submit(@RequestBody MetricPoint metric) { ... }
    
    @GetMapping
    public ResponseEntity<List<MetricPoint>> query(@RequestParam long startTime) { ... }
}
```

**Rust/Axum:**
```rust
let app = Router::new()
    .route("/metrics", post(submit_metric).get(query_metrics))  // Multiple methods on same path
    .route("/statistics", get(get_statistics));

async fn submit_metric(
    Json(metric): Json<MetricPoint>,  // Equivalent to @RequestBody
) -> Result<StatusCode, StatusCode> { ... }
```

### Request Parameter Binding

**Spring Boot:**
```java
public ResponseEntity<?> query(
    @RequestParam long startTime,
    @RequestParam long endTime,
    @RequestParam(required = false) String hostnameFilter
) { ... }
```

**Rust/Axum:**
```rust
#[derive(Debug, Deserialize)]
struct QueryParams {
    start_time: i64,
    end_time: i64,
    hostname_filter: Option<String>,  // Option<T> = optional parameter
}

async fn query_metrics(
    Query(params): Query<QueryParams>,  // Automatic deserialization from URL params
) -> Result<Json<Vec<MetricPoint>>, StatusCode> { ... }
```

### Extractors: Axum's Key Feature
Extractors pull data from HTTP requests in a type-safe way:

- `Json<T>` - Request body as JSON (like `@RequestBody`)
- `Query<T>` - URL query parameters (like `@RequestParam`)
- `State<T>` - Shared application state (like `@Autowired`)
- `Path<T>` - URL path parameters (like `@PathVariable`)

### Error Handling and Response Types

**Spring Boot:**
```java
try {
    service.store(metric);
    return ResponseEntity.status(HttpStatus.CREATED).build();
} catch (Exception e) {
    return ResponseEntity.status(HttpStatus.INTERNAL_SERVER_ERROR).build();
}
```

**Rust/Axum:**
```rust
match state.storage.store_metric(metric) {
    Ok(_) => Ok(StatusCode::CREATED),           // HTTP 201
    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),  // HTTP 500
}
```

**Key Difference**: Rust uses `Result<T, E>` pattern instead of exceptions. More explicit but compile-time safe.

### JSON Serialization

**Spring Boot:**
```java
@RestController  // Automatic JSON via Jackson
public List<MetricPoint> getMetrics() {
    return metrics;  // Auto-serialized to JSON
}
```

**Rust/Axum:**
```rust
-> Result<Json<Vec<MetricPoint>>, StatusCode>
//         ↑ Json<T> wrapper handles serialization
```

### Complete Handler Example from Codebase

```rust
async fn query_metrics(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,  // Get shared state
    Query(params): Query<QueryParams>,                                 // Extract query params
) -> Result<Json<Vec<MetricPoint>>, StatusCode> {                     // Return JSON or error
    let query = MetricQuery {
        start_time: params.start_time,
        end_time: params.end_time,
        hostname_filter: params.hostname_filter,
    };

    match state.storage.query_metrics(&query) {
        Ok(metrics) => Ok(Json(metrics)),           // Serialize to JSON
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
```

## Cap'n Proto: Handling Optional Fields (vs Protobuf)

### The Problem: "Explicitly Set to 0" vs "Never Set"

In protobuf, you can distinguish between these cases:
```proto
message Example {
    optional int32 count = 1;  // Has has_count() method
}
```

```java
if (example.hasCount()) {
    // Was explicitly set (could be 0)
} else {
    // Was never set
}
```

### Cap'n Proto's Zero-Cost Defaults Challenge

Cap'n Proto uses **zero-cost defaults** - unset fields don't consume space, but you lose the distinction:

```capnp
struct Example {
    count @0 :Int32;  # Defaults to 0, but can't tell if explicitly set to 0
}
```

### Solution 1: Union Types (Most Precise)
```capnp
struct MyMessage {
  value :union {
    unset @0 :Void;        # Explicitly unset
    setValue @1 :Int32;    # Explicitly set to some value (including 0)
  }
}
```

**Rust Usage:**
```rust
match message.get_value().which()? {
    ValueUnset(()) => println!("Value is unset"),
    ValueSetValue(val) => println!("Value explicitly set to: {}", val?),
}
```

### Solution 2: Wrapper Struct
```capnp
struct OptionalInt32 {
  hasValue @0 :Bool;
  value @1 :Int32;
}

struct MyMessage {
  count @0 :OptionalInt32;
}
```

**Rust Usage:**
```rust
if message.get_count()?.get_has_value() {
    let value = message.get_count()?.get_value();
    println!("Count is explicitly: {}", value);
} else {
    println!("Count was never set");
}
```

### Solution 3: Sentinel Values (Domain-Dependent)
```capnp
struct MyMessage {
  count @0 :Int32 = -1;  # Use -1 to mean "unset", 0+ for real values
}
```

**Good for**: Counts, IDs where negative values don't make sense
**Bad for**: Temperature, coordinates, or other domains where all integers are valid

### Solution 4: Boolean Flag Pattern
```capnp
struct MyMessage {
  hasCount @0 :Bool;
  count @1 :Int32;    # Only meaningful if hasCount is true
}
```

### Cap'n Proto Default Values by Type

| Type | Default Value | Notes |
|------|---------------|-------|
| `Bool` | `false` | Can't distinguish false vs unset |
| `Int32`, `Int64`, etc. | `0` | Can't distinguish 0 vs unset |
| `Float32`, `Float64` | `0.0` | Can't distinguish 0.0 vs unset |
| `Text` | `""` | Empty string means unset |
| `Data` | empty bytes | |
| `List(T)` | empty list | |

### When to Use Each Solution

1. **Union Types**: When you need precise semantics and don't mind slight overhead
2. **Wrapper Struct**: When you want reusable optional types
3. **Sentinel Values**: When your domain has natural "invalid" values
4. **Boolean Flag**: Simple cases where you control both ends of the protocol

### Example from Our Codebase

In `schemas/metrics.proto`:
```proto
optional string hostname_filter = 3;  # Protobuf: explicit optionality
```

In `schemas/metrics.capnp`:
```capnp
hostnameFilter @2 :Text;  # Defaults to "", empty string means "no filter"
```

**Rust handling:**
```rust
// Check if hostname filter was provided
let hostname_filter = if query.get_hostname_filter()?.is_empty() {
    None  // Empty string = not provided
} else {
    Some(query.get_hostname_filter()?.to_string())
};
```

**Trade-off**: Cap'n Proto saves space (empty strings are zero-cost) but requires application-level conventions for optionality.

## Contract-First vs Traditional Development Approaches

### Traditional Spring Boot Development (What I Used to Do)
In traditional Spring Boot development, you write POJOs manually and rely on runtime behavior:

```java
// Manually written POJOs
public class MetricPoint {
    private long timestamp;
    private String hostname;
    private float cpuPercent;
    
    // Manual getters and setters
    public long getTimestamp() { return timestamp; }
    public void setTimestamp(long timestamp) { this.timestamp = timestamp; }
    // ... more getters/setters
}

@RestController
public class MetricsController {
    @PostMapping("/metrics")
    public ResponseEntity<Void> submitMetric(@RequestBody MetricPoint metric) {
        // Your business logic
        return ResponseEntity.ok();
    }
}
```

**Traditional Approach Flow:**
```
Code → Annotations → Runtime behavior
```

### Contract-First Development (Modern Approach)

#### REST with OpenAPI/Swagger
Define your API contract first, then generate code:

**1. API Contract Definition:**
```yaml
# api.yaml - OpenAPI specification
openapi: 3.0.0
info:
  title: Metrics API
paths:
  /metrics:
    post:
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/MetricPoint'
      responses:
        '201':
          description: Created
components:
  schemas:
    MetricPoint:
      type: object
      properties:
        timestamp:
          type: integer
          format: int64
        hostname:
          type: string
        cpuPercent:
          type: number
          format: float
```

**2. Generated POJOs:**
```java
// Generated from api.yaml (using Maven/Gradle plugin)
public class MetricPoint {
    @JsonProperty("timestamp")
    private Long timestamp;
    
    @JsonProperty("hostname") 
    private String hostname;
    
    @JsonProperty("cpuPercent")
    private Float cpuPercent;
    
    // Generated getters/setters, equals, hashCode, toString
}

// Generated controller interface
public interface MetricsApi {
    ResponseEntity<Void> submitMetric(MetricPoint metric);
}
```

**3. Your Implementation:**
```java
@RestController
public class MetricsController implements MetricsApi {  // Implement generated interface
    
    @Override
    public ResponseEntity<Void> submitMetric(MetricPoint metric) {
        // Your business logic here
        service.store(metric);
        return ResponseEntity.status(201).build();
    }
}
```

**Contract-First Approach Flow:**
```
Schema → Generated code → Runtime behavior
```

### Why Contract-First?

**Benefits you might recognize:**
1. **API Documentation**: OpenAPI spec automatically generates Swagger UI
2. **Client Generation**: Generate client SDKs for multiple languages
3. **Validation**: Ensure request/response match the contract
4. **Team Coordination**: Frontend/backend teams work from same contract

### The Parallel: REST vs gRPC

| Traditional REST      | Contract-First REST           | gRPC                         |
|-----------------------|-------------------------------|------------------------------|
| Write POJOs manually  | Generate POJOs from OpenAPI   | Generate structs from .proto |
| Write @RestController | Implement generated interface | Implement generated trait    |
| Jackson handles JSON  | Jackson handles JSON          | Protobuf handles binary      |
| Spring handles HTTP   | Spring handles HTTP           | Tonic handles HTTP/2         |

### Side-by-Side Comparison: REST vs gRPC

| Aspect                | REST + JSON            | gRPC + Protobuf      |
|-----------------------|------------------------|----------------------|
| Contract              | OpenAPI/Swagger YAML   | .proto file          |
| Generated Data Models | Jackson POJOs          | Protobuf structs     |
| Generated Interface   | Spring MVC annotations | Service trait        |
| Serialization         | JSON (Jackson)         | Binary protobuf      |
| Network Protocol      | HTTP/1.1               | HTTP/2               |
| Your Implementation   | @RestController class  | Trait implementation |
| Server Framework      | Spring Boot            | Tonic                |

### In Your ProtoBench Project

**REST Service** (traditional approach in `rest-service/src/main.rs`):
- Manually defined structs using Serde
- Hand-written Axum handlers

**gRPC Service** (contract-first in `grpc-service/`):
- Schema in `schemas/metrics.proto`
- Generated structs and traits
- You implement the generated trait

**Key Insight**: OpenAPI/Swagger is optional for REST APIs - it's the contract-first alternative to manually writing POJOs. But for gRPC, the `.proto` file approach is the only way - there's no "traditional" hand-written option. The mental model is identical: both approaches generate the "plumbing" (data structures, serialization, network handling) from a schema definition, leaving you to implement the actual business logic.