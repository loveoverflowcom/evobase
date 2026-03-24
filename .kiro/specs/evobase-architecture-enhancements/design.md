# Design Document: EvoBase Architecture Enhancements

## Tổng Quan (Overview)

EvoBase hiện tại là một REST API server với core functionality (CRUD, Auth, RLS). Dựa trên phân tích kiến trúc PostgREST, chúng ta xác định được các gap quan trọng cần nâng cấp để EvoBase trở thành production-ready system với khả năng mở rộng, bảo mật, và observability tốt hơn.

**Mục tiêu chính:**
- Nâng cao extensibility thông qua plugin architecture
- Tăng cường security với token revocation, rate limiting, audit logging
- Cải thiện performance với caching, read replicas, query cost estimation
- Tăng observability với distributed tracing, enhanced metrics, structured logging
- Thêm advanced features: GraphQL, WebSocket, multi-tenancy

**Phạm vi:**
- Backend: Rust (Axum framework)
- Database: PostgreSQL
- Cache: Redis
- Monitoring: Prometheus, Jaeger, Loki
- Frontend: Flutter workbench (existing)

**Timeline:** 12-18 tháng (5 phases)

**Nguyên tắc thiết kế:**
- Giữ nguyên database-centric philosophy
- Backward compatibility với existing APIs
- Opt-in features (không breaking changes)
- Performance không được regression
- Simplicity for basic use cases


## Kiến Trúc Hệ Thống (System Architecture)

### Current Architecture (As-Is)

```mermaid
flowchart TD
    Client[HTTP Client] -->|Request| Auth[Auth Middleware]
    Auth -->|JWT Validation| Router[Axum Router]
    Router -->|CRUD| RestHandler[REST Handler]
    Router -->|RPC| RpcHandler[RPC Handler]
    Router -->|Docs| DocsHandler[Docs Handler]
    RestHandler -->|Query| Pool[Connection Pool]
    RpcHandler -->|Execute| Pool
    DocsHandler -->|Introspect| Pool
    Pool -->|SQL| DB[(PostgreSQL)]
    
    style Auth fill:#90EE90
    style DB fill:#4169E1
```

**Đặc điểm hiện tại:**
- ✅ Core REST API với JWT auth
- ✅ Basic CRUD operations
- ✅ RLS support
- ✅ Admin mode
- ✅ SSE events
- ⏳ Lab features (đang implement)
- ❌ Không có caching layer
- ❌ Không có rate limiting
- ❌ Không có token revocation
- ❌ Không có distributed tracing
- ❌ Không có read replica support


### Target Architecture (To-Be)

```mermaid
flowchart TD
    Client[HTTP Client] -->|Request| Gateway[API Gateway Layer]
    Gateway -->|Rate Limit| Auth[Auth Middleware]
    Auth -->|JWT + Revocation Check| Cache[Cache Layer]
    Cache -->|Cache Miss| Trace[Tracing Layer]
    Trace -->|Span| Plugin[Plugin System]
    Plugin -->|Custom Logic| Router[Axum Router]
    
    Router -->|CRUD| RestHandler[REST Handler]
    Router -->|RPC| RpcHandler[RPC Handler]
    Router -->|GraphQL| GraphQLHandler[GraphQL Handler]
    Router -->|WebSocket| WSHandler[WebSocket Handler]
    Router -->|Docs| DocsHandler[Docs Handler]
    
    RestHandler -->|Query| QueryBuilder[Query Builder]
    RpcHandler -->|Execute| QueryBuilder
    GraphQLHandler -->|Resolve| QueryBuilder
    
    QueryBuilder -->|Cost Check| CostEstimator[Cost Estimator]
    CostEstimator -->|OK| PoolManager[Pool Manager]
    PoolManager -->|Read| ReplicaPool[Replica Pool]
    PoolManager -->|Write| PrimaryPool[Primary Pool]
    
    ReplicaPool -->|SQL| Replica1[(Replica 1)]
    ReplicaPool -->|SQL| Replica2[(Replica 2)]
    PrimaryPool -->|SQL| Primary[(Primary DB)]
    
    Cache -->|Store| Redis[(Redis Cache)]
    Auth -->|Check Revocation| Redis
    Gateway -->|Rate Limit State| Redis
    
    Trace -->|Traces| Jaeger[Jaeger]
    Router -->|Metrics| Prometheus[Prometheus]
    Router -->|Logs| Loki[Loki]
    
    WSHandler -->|LISTEN| Primary
    Primary -->|NOTIFY| WSHandler
    
    style Gateway fill:#FFD700
    style Cache fill:#FF6347
    style Plugin fill:#9370DB
    style Redis fill:#DC143C
    style Jaeger fill:#00CED1
    style Prometheus fill:#FF8C00
    style Primary fill:#4169E1
```

**Các layer mới:**
1. **API Gateway Layer**: Rate limiting, request validation
2. **Cache Layer**: Response caching, query result caching
3. **Tracing Layer**: Distributed tracing với OpenTelemetry
4. **Plugin System**: Extensibility cho custom logic
5. **Pool Manager**: Read replica routing, health checking
6. **Cost Estimator**: Query cost estimation, expensive query prevention
7. **Observability Stack**: Prometheus, Jaeger, Loki


## Core Components Design

### 1. Plugin System Architecture

**Mục đích:** Cho phép mở rộng EvoBase với custom logic mà không cần fork code.

```mermaid
flowchart LR
    Core[EvoBase Core] -->|Load| Registry[Plugin Registry]
    Registry -->|Initialize| Auth[Auth Plugins]
    Registry -->|Initialize| Cache[Cache Plugins]
    Registry -->|Initialize| Obs[Observability Plugins]
    Registry -->|Initialize| Custom[Custom Plugins]
    
    Request[HTTP Request] -->|1| Core
    Core -->|2| Auth
    Auth -->|3| Cache
    Cache -->|4| Obs
    Obs -->|5| Handler[Request Handler]
    
    style Registry fill:#9370DB
    style Auth fill:#90EE90
    style Cache fill:#FF6347
    style Obs fill:#00CED1
```

**Plugin Interface (Rust):**

```rust
// Plugin trait
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn init(&mut self, config: &PluginConfig) -> Result<()>;
    fn middleware(&self) -> Option<Box<dyn Middleware>>;
    fn shutdown(&mut self) -> Result<()>;
}

// Plugin types
pub enum PluginType {
    Auth(Box<dyn AuthPlugin>),
    Cache(Box<dyn CachePlugin>),
    Observability(Box<dyn ObservabilityPlugin>),
    Custom(Box<dyn Plugin>),
}

// Plugin registry
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
    middleware_chain: Vec<Box<dyn Middleware>>,
}

impl PluginRegistry {
    pub fn load_from_dir(&mut self, path: &Path) -> Result<()>;
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<()>;
    pub fn build_middleware_chain(&self) -> Vec<Box<dyn Middleware>>;
}
```

**Configuration:**

```yaml
plugins:
  enabled: true
  directory: ./plugins
  plugins:
    - name: custom-auth
      enabled: true
      config:
        provider: oauth2
        client_id: xxx
    - name: redis-cache
      enabled: true
      config:
        url: redis://localhost:6379
```


### 2. Caching Layer Design

**Mục đích:** Giảm database load, tăng response time cho cacheable data.

```mermaid
flowchart TD
    Request[HTTP Request] -->|1| CacheMiddleware[Cache Middleware]
    CacheMiddleware -->|2 Check| Redis[(Redis Cache)]
    Redis -->|Hit| Response[Return Cached Response]
    Redis -->|Miss| Handler[Request Handler]
    Handler -->|3 Query| DB[(PostgreSQL)]
    DB -->|4 Result| Handler
    Handler -->|5 Store| Redis
    Handler -->|6| Response
    
    Mutation[Mutation Request] -->|Invalidate| Redis
    
    style Redis fill:#DC143C
    style DB fill:#4169E1
```

**Cache Strategy:**

```rust
// Cache key generation
pub struct CacheKey {
    path: String,
    query: String,
    role: String,
    accept: String,
}

impl CacheKey {
    pub fn to_string(&self) -> String {
        format!("evobase:{}:{}:{}:{}", 
            self.path, self.query, self.role, self.accept)
    }
}

// Cache interface
#[async_trait]
pub trait ResponseCache: Send + Sync {
    async fn get(&self, key: &CacheKey) -> Result<Option<CachedResponse>>;
    async fn put(&self, key: &CacheKey, response: CachedResponse, ttl: Duration) -> Result<()>;
    async fn invalidate(&self, pattern: &str) -> Result<()>;
}

// Redis implementation
pub struct RedisCache {
    client: redis::Client,
    default_ttl: Duration,
}

// Cache configuration
pub struct CacheConfig {
    pub enabled: bool,
    pub backend: CacheBackend,
    pub default_ttl: Duration,
    pub rules: Vec<CacheRule>,
}

pub struct CacheRule {
    pub path_pattern: String,
    pub methods: Vec<String>,
    pub ttl: Duration,
    pub vary: Vec<String>,
    pub invalidate_on: Vec<String>,
}
```

**Configuration:**

```yaml
caching:
  enabled: true
  backend: redis
  redis-url: redis://localhost:6379
  default-ttl: 300  # seconds
  rules:
    - path: /rest/users
      methods: [GET]
      ttl: 60
      vary: [Accept, Accept-Language]
      invalidate-on:
        - POST /rest/users
        - PATCH /rest/users/*
        - DELETE /rest/users/*
    - path: /rest/posts
      methods: [GET]
      ttl: 300
      invalidate-on:
        - POST /rest/posts
        - PATCH /rest/posts/*
        - DELETE /rest/posts/*
```


### 3. Security Layer Design

#### 3.1 Token Revocation System

**Mục đích:** Cho phép revoke JWT tokens trước khi expiry.

```mermaid
flowchart TD
    Request[HTTP Request] -->|1 Extract JWT| Auth[Auth Middleware]
    Auth -->|2 Check Revocation| Redis[(Redis)]
    Redis -->|Revoked| Reject[401 Unauthorized]
    Redis -->|Valid| Validate[Validate JWT]
    Validate -->|Invalid| Reject
    Validate -->|Valid| Handler[Request Handler]
    
    Admin[Admin Request] -->|Revoke Token| RevokeAPI[Revoke API]
    RevokeAPI -->|Store| Redis
    
    style Redis fill:#DC143C
```

**Implementation:**

```rust
// Token revocation interface
#[async_trait]
pub trait TokenRevocation: Send + Sync {
    async fn revoke(&self, token: &str, expiry: DateTime<Utc>) -> Result<()>;
    async fn is_revoked(&self, token: &str) -> Result<bool>;
    async fn cleanup_expired(&self) -> Result<()>;
}

// Redis implementation
pub struct RedisTokenRevocation {
    client: redis::Client,
    key_prefix: String,
}

impl RedisTokenRevocation {
    pub async fn revoke(&self, token: &str, expiry: DateTime<Utc>) -> Result<()> {
        let key = format!("{}revoked:{}", self.key_prefix, token);
        let ttl = (expiry - Utc::now()).num_seconds();
        self.client.setex(&key, ttl, "1").await?;
        Ok(())
    }
    
    pub async fn is_revoked(&self, token: &str) -> Result<bool> {
        let key = format!("{}revoked:{}", self.key_prefix, token);
        let exists = self.client.exists(&key).await?;
        Ok(exists)
    }
}
```

**Admin API:**

```rust
// POST /admin/revoke-token
pub async fn revoke_token(
    State(revocation): State<Arc<dyn TokenRevocation>>,
    Json(payload): Json<RevokeTokenRequest>,
) -> Result<StatusCode> {
    revocation.revoke(&payload.token, payload.expiry).await?;
    Ok(StatusCode::NO_CONTENT)
}

// GET /admin/revoked-tokens
pub async fn list_revoked_tokens(
    State(revocation): State<Arc<dyn TokenRevocation>>,
) -> Result<Json<Vec<String>>> {
    let tokens = revocation.list_revoked().await?;
    Ok(Json(tokens))
}
```


#### 3.2 Rate Limiting

**Mục đích:** Prevent abuse và DoS attacks.

```mermaid
flowchart TD
    Request[HTTP Request] -->|1| RateLimit[Rate Limit Middleware]
    RateLimit -->|2 Check| Redis[(Redis)]
    Redis -->|Exceeded| Reject[429 Too Many Requests]
    Redis -->|OK| Increment[Increment Counter]
    Increment -->|3| Handler[Request Handler]
    
    style Redis fill:#DC143C
```

**Implementation:**

```rust
// Rate limit configuration
pub struct RateLimitConfig {
    pub enabled: bool,
    pub global: RateLimitRule,
    pub per_ip: RateLimitRule,
    pub per_user: RateLimitRule,
    pub per_endpoint: Vec<EndpointRateLimit>,
}

pub struct RateLimitRule {
    pub window: Duration,
    pub max_requests: u32,
}

pub struct EndpointRateLimit {
    pub path: String,
    pub window: Duration,
    pub max_requests: u32,
}

// Rate limiter interface
#[async_trait]
pub trait RateLimiter: Send + Sync {
    async fn check(&self, key: &str, rule: &RateLimitRule) -> Result<RateLimitResult>;
}

pub struct RateLimitResult {
    pub allowed: bool,
    pub limit: u32,
    pub remaining: u32,
    pub reset: DateTime<Utc>,
}

// Redis implementation using sliding window
pub struct RedisRateLimiter {
    client: redis::Client,
}

impl RedisRateLimiter {
    pub async fn check(&self, key: &str, rule: &RateLimitRule) -> Result<RateLimitResult> {
        let now = Utc::now().timestamp();
        let window_start = now - rule.window.as_secs() as i64;
        
        // Remove old entries
        self.client.zremrangebyscore(key, 0, window_start).await?;
        
        // Count current requests
        let count: u32 = self.client.zcard(key).await?;
        
        if count >= rule.max_requests {
            return Ok(RateLimitResult {
                allowed: false,
                limit: rule.max_requests,
                remaining: 0,
                reset: Utc::now() + rule.window,
            });
        }
        
        // Add current request
        self.client.zadd(key, now, now).await?;
        self.client.expire(key, rule.window.as_secs()).await?;
        
        Ok(RateLimitResult {
            allowed: true,
            limit: rule.max_requests,
            remaining: rule.max_requests - count - 1,
            reset: Utc::now() + rule.window,
        })
    }
}
```

**Response Headers:**

```rust
// Add rate limit headers to response
pub fn add_rate_limit_headers(
    response: &mut Response,
    result: &RateLimitResult,
) {
    response.headers_mut().insert(
        "X-RateLimit-Limit",
        result.limit.to_string().parse().unwrap(),
    );
    response.headers_mut().insert(
        "X-RateLimit-Remaining",
        result.remaining.to_string().parse().unwrap(),
    );
    response.headers_mut().insert(
        "X-RateLimit-Reset",
        result.reset.timestamp().to_string().parse().unwrap(),
    );
    
    if !result.allowed {
        response.headers_mut().insert(
            "Retry-After",
            (result.reset - Utc::now()).num_seconds().to_string().parse().unwrap(),
        );
    }
}
```

**Configuration:**

```yaml
rate-limiting:
  enabled: true
  global:
    window: 60  # seconds
    max-requests: 1000
  per-ip:
    window: 60
    max-requests: 100
  per-user:
    window: 60
    max-requests: 500
  per-endpoint:
    - path: /rpc/expensive_function
      window: 60
      max-requests: 10
    - path: /rest/users
      window: 60
      max-requests: 200
```


#### 3.3 Audit Logging

**Mục đích:** Comprehensive audit trail cho compliance và security forensics.

```mermaid
flowchart LR
    Request[HTTP Request] -->|1| Handler[Request Handler]
    Handler -->|2 Execute| DB[(PostgreSQL)]
    DB -->|3 Result| Handler
    Handler -->|4 Log| AuditLogger[Audit Logger]
    AuditLogger -->|5 Store| AuditTable[(audit_log table)]
    
    style AuditTable fill:#4169E1
```

**Database Schema:**

```sql
CREATE TABLE audit_log (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT now(),
    user_id TEXT,
    user_role TEXT NOT NULL,
    client_ip INET NOT NULL,
    method TEXT NOT NULL,
    path TEXT NOT NULL,
    action TEXT NOT NULL,
    resource TEXT NOT NULL,
    rows_affected INT,
    success BOOLEAN NOT NULL,
    error_code TEXT,
    request_body JSONB,
    response_status INT,
    duration_ms INT
);

CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp DESC);
CREATE INDEX idx_audit_log_user_id ON audit_log(user_id);
CREATE INDEX idx_audit_log_resource ON audit_log(resource);
CREATE INDEX idx_audit_log_action ON audit_log(action);
```

**Implementation:**

```rust
// Audit log entry
pub struct AuditLogEntry {
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub user_role: String,
    pub client_ip: IpAddr,
    pub method: String,
    pub path: String,
    pub action: AuditAction,
    pub resource: String,
    pub rows_affected: Option<i32>,
    pub success: bool,
    pub error_code: Option<String>,
    pub request_body: Option<serde_json::Value>,
    pub response_status: i32,
    pub duration_ms: i64,
}

pub enum AuditAction {
    Read,
    Create,
    Update,
    Delete,
    Rpc,
}

// Audit logger interface
#[async_trait]
pub trait AuditLogger: Send + Sync {
    async fn log(&self, entry: AuditLogEntry) -> Result<()>;
    async fn query(&self, query: AuditQuery) -> Result<Vec<AuditLogEntry>>;
}

// Database implementation
pub struct DatabaseAuditLogger {
    pool: PgPool,
}

impl DatabaseAuditLogger {
    pub async fn log(&self, entry: AuditLogEntry) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO audit_log (
                timestamp, user_id, user_role, client_ip, method, path,
                action, resource, rows_affected, success, error_code,
                request_body, response_status, duration_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            entry.timestamp,
            entry.user_id,
            entry.user_role,
            entry.client_ip.to_string(),
            entry.method,
            entry.path,
            entry.action.to_string(),
            entry.resource,
            entry.rows_affected,
            entry.success,
            entry.error_code,
            entry.request_body,
            entry.response_status,
            entry.duration_ms as i32,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
```

**Configuration:**

```yaml
audit-logging:
  enabled: true
  backend: database
  table: audit_log
  include-request-body: false  # PII concerns
  include-response-body: false
  sample-rate: 1.0  # 100% of requests
  exclude-paths:
    - /health
    - /metrics
  async: true  # Don't block request
```


### 4. Observability Layer Design

#### 4.1 Distributed Tracing

**Mục đích:** End-to-end request tracing cho debugging và performance analysis.

```mermaid
flowchart TD
    Client[HTTP Client] -->|traceparent header| Gateway[API Gateway]
    Gateway -->|Create Span| Tracer[OpenTelemetry Tracer]
    Tracer -->|http.request span| Auth[Auth Middleware]
    Auth -->|auth.validate span| Cache[Cache Layer]
    Cache -->|cache.check span| Handler[Request Handler]
    Handler -->|db.query span| DB[(PostgreSQL)]
    
    Tracer -->|Export| Jaeger[Jaeger]
    
    style Tracer fill:#00CED1
    style Jaeger fill:#00CED1
```

**Implementation:**

```rust
use opentelemetry::{global, trace::{Tracer, Span}};
use opentelemetry_jaeger::JaegerPipeline;

// Initialize tracer
pub fn init_tracer(config: &TracingConfig) -> Result<()> {
    let tracer = JaegerPipeline::new()
        .with_service_name("evobase")
        .with_agent_endpoint(&config.jaeger_endpoint)
        .with_trace_config(
            opentelemetry::sdk::trace::config()
                .with_sampler(opentelemetry::sdk::trace::Sampler::TraceIdRatioBased(
                    config.sample_rate,
                ))
        )
        .install_batch(opentelemetry::runtime::Tokio)?;
    
    global::set_tracer_provider(tracer);
    Ok(())
}

// Tracing middleware
pub async fn tracing_middleware(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response> {
    let tracer = global::tracer("evobase");
    
    // Extract trace context from headers
    let parent_context = extract_trace_context(&req);
    
    // Create span
    let mut span = tracer
        .span_builder("http.request")
        .with_parent_context(parent_context)
        .start(&tracer);
    
    // Add attributes
    span.set_attribute(KeyValue::new("http.method", req.method().to_string()));
    span.set_attribute(KeyValue::new("http.path", req.uri().path().to_string()));
    span.set_attribute(KeyValue::new("http.user_agent", 
        req.headers().get("user-agent").map(|v| v.to_str().unwrap_or("")).unwrap_or("")));
    
    let start = Instant::now();
    
    // Execute request
    let response = next.run(req).await;
    
    let duration = start.elapsed();
    
    // Add response attributes
    span.set_attribute(KeyValue::new("http.status_code", response.status().as_u16() as i64));
    span.set_attribute(KeyValue::new("http.duration_ms", duration.as_millis() as i64));
    
    span.end();
    
    Ok(response)
}

// Database query tracing
pub async fn trace_query<T>(
    tracer: &Tracer,
    parent_span: &Span,
    query: &str,
    f: impl Future<Output = Result<T>>,
) -> Result<T> {
    let mut span = tracer
        .span_builder("db.query")
        .with_parent_context(parent_span.span_context().clone())
        .start(tracer);
    
    span.set_attribute(KeyValue::new("db.system", "postgresql"));
    span.set_attribute(KeyValue::new("db.operation", extract_operation(query)));
    
    let start = Instant::now();
    let result = f.await;
    let duration = start.elapsed();
    
    span.set_attribute(KeyValue::new("db.duration_ms", duration.as_millis() as i64));
    
    if let Err(e) = &result {
        span.set_attribute(KeyValue::new("error", true));
        span.set_attribute(KeyValue::new("error.message", e.to_string()));
    }
    
    span.end();
    
    result
}
```

**Configuration:**

```yaml
tracing:
  enabled: true
  exporter: jaeger
  jaeger-endpoint: http://localhost:14268/api/traces
  sample-rate: 0.1  # 10% of requests
  include-db-queries: true
  include-query-params: false  # PII concerns
```


#### 4.2 Enhanced Metrics

**Mục đích:** Comprehensive metrics cho monitoring và alerting.

```mermaid
flowchart LR
    Request[HTTP Request] -->|Record| Metrics[Metrics Collector]
    Metrics -->|Increment| Counters[Counters]
    Metrics -->|Observe| Histograms[Histograms]
    Metrics -->|Set| Gauges[Gauges]
    
    Prometheus[Prometheus] -->|Scrape| MetricsEndpoint[/metrics]
    MetricsEndpoint -->|Expose| Metrics
    
    Grafana[Grafana] -->|Query| Prometheus
    
    style Prometheus fill:#FF8C00
    style Grafana fill:#FF8C00
```

**Implementation:**

```rust
use prometheus::{
    Counter, Histogram, Gauge, Registry, Opts, HistogramOpts,
    register_counter_vec, register_histogram_vec, register_gauge_vec,
};

// Metrics registry
pub struct MetricsRegistry {
    // Request metrics
    pub request_count: CounterVec,
    pub request_duration: HistogramVec,
    pub active_requests: GaugeVec,
    
    // Database metrics
    pub db_query_duration: HistogramVec,
    pub db_connections_active: Gauge,
    pub db_connections_idle: Gauge,
    pub db_pool_wait_duration: Histogram,
    
    // Cache metrics
    pub cache_hits: CounterVec,
    pub cache_misses: CounterVec,
    
    // Security metrics
    pub rate_limit_exceeded: CounterVec,
    pub auth_failures: CounterVec,
    pub query_cost_rejected: Counter,
    
    // Schema cache metrics
    pub schema_cache_reload: Counter,
    pub schema_cache_reload_duration: Histogram,
    pub schema_cache_size_bytes: Gauge,
}

impl MetricsRegistry {
    pub fn new() -> Result<Self> {
        Ok(Self {
            request_count: register_counter_vec!(
                "evobase_http_requests_total",
                "Total HTTP requests",
                &["method", "path", "status", "role"]
            )?,
            request_duration: register_histogram_vec!(
                "evobase_http_request_duration_seconds",
                "HTTP request duration",
                &["method", "path", "status", "role"]
            )?,
            active_requests: register_gauge_vec!(
                "evobase_http_requests_active",
                "Active HTTP requests",
                &["method", "path"]
            )?,
            db_query_duration: register_histogram_vec!(
                "evobase_db_query_duration_seconds",
                "Database query duration",
                &["operation"]
            )?,
            db_connections_active: register_gauge!(
                "evobase_db_connections_active",
                "Active database connections"
            )?,
            db_connections_idle: register_gauge!(
                "evobase_db_connections_idle",
                "Idle database connections"
            )?,
            cache_hits: register_counter_vec!(
                "evobase_cache_hits_total",
                "Cache hits",
                &["path"]
            )?,
            cache_misses: register_counter_vec!(
                "evobase_cache_misses_total",
                "Cache misses",
                &["path"]
            )?,
            rate_limit_exceeded: register_counter_vec!(
                "evobase_rate_limit_exceeded_total",
                "Rate limit exceeded",
                &["scope"]
            )?,
            auth_failures: register_counter_vec!(
                "evobase_auth_failures_total",
                "Authentication failures",
                &["reason"]
            )?,
            // ... other metrics
        })
    }
}

// Metrics middleware
pub async fn metrics_middleware(
    State(metrics): State<Arc<MetricsRegistry>>,
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response> {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    
    // Increment active requests
    metrics.active_requests
        .with_label_values(&[&method, &path])
        .inc();
    
    let start = Instant::now();
    
    // Execute request
    let response = next.run(req).await;
    
    let duration = start.elapsed();
    let status = response.status().as_u16().to_string();
    let role = extract_role(&response).unwrap_or("anonymous");
    
    // Record metrics
    metrics.request_count
        .with_label_values(&[&method, &path, &status, role])
        .inc();
    
    metrics.request_duration
        .with_label_values(&[&method, &path, &status, role])
        .observe(duration.as_secs_f64());
    
    metrics.active_requests
        .with_label_values(&[&method, &path])
        .dec();
    
    Ok(response)
}
```

**Prometheus Metrics Example:**

```
# Request metrics
evobase_http_requests_total{method="GET",path="/rest/users",status="200",role="authenticated"} 1234
evobase_http_request_duration_seconds{method="GET",path="/rest/users",status="200",role="authenticated"} 0.045
evobase_http_requests_active{method="GET",path="/rest/users"} 42

# Database metrics
evobase_db_query_duration_seconds{operation="SELECT"} 0.023
evobase_db_connections_active 8
evobase_db_connections_idle 2

# Cache metrics
evobase_cache_hits_total{path="/rest/users"} 567
evobase_cache_misses_total{path="/rest/users"} 123

# Security metrics
evobase_rate_limit_exceeded_total{scope="ip"} 45
evobase_auth_failures_total{reason="invalid_token"} 12
```


#### 4.3 Structured Logging

**Mục đích:** Machine-readable logs cho analysis và debugging.

```mermaid
flowchart LR
    App[EvoBase] -->|JSON Logs| Stdout[Stdout]
    Stdout -->|Collect| Promtail[Promtail]
    Promtail -->|Push| Loki[Loki]
    Grafana[Grafana] -->|Query| Loki
    
    style Loki fill:#00CED1
    style Grafana fill:#FF8C00
```

**Implementation:**

```rust
use tracing::{info, warn, error, debug};
use tracing_subscriber::{fmt, EnvFilter};

// Initialize structured logging
pub fn init_logging(config: &LoggingConfig) -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .json()  // JSON format
        .with_current_span(true)
        .with_span_list(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();
    
    Ok(())
}

// Structured log entry
#[derive(Serialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub context: LogContext,
    pub error: Option<String>,
    pub trace: Option<TraceContext>,
}

#[derive(Serialize)]
pub struct LogContext {
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub user_role: Option<String>,
    pub method: Option<String>,
    pub path: Option<String>,
    pub duration_ms: Option<i64>,
    pub custom_fields: HashMap<String, serde_json::Value>,
}

// Logging middleware
pub async fn logging_middleware(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response> {
    let request_id = Uuid::new_v4().to_string();
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let user_agent = req.headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        user_agent = %user_agent,
        "Request started"
    );
    
    let start = Instant::now();
    
    let response = next.run(req).await;
    
    let duration = start.elapsed();
    let status = response.status().as_u16();
    
    info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        status = status,
        duration_ms = duration.as_millis() as i64,
        "Request completed"
    );
    
    Ok(response)
}
```

**Example JSON Log Output:**

```json
{
  "timestamp": "2024-03-24T10:30:45.123Z",
  "level": "INFO",
  "message": "Request completed",
  "context": {
    "request_id": "req_abc123",
    "user_id": "user_456",
    "user_role": "authenticated",
    "method": "GET",
    "path": "/rest/users",
    "status": 200,
    "duration_ms": 45,
    "db_query_count": 1,
    "cache_hit": true
  },
  "trace": {
    "trace_id": "0af7651916cd43dd8448eb211c80319c",
    "span_id": "b7ad6b7169203331"
  }
}
```

**Configuration:**

```yaml
logging:
  level: info  # debug, info, warn, error
  format: json  # json or text
  include-trace: true
  include-query-params: false  # PII concerns
  sample-rate: 1.0  # 100% of requests
```


### 5. Performance Layer Design

#### 5.1 Read Replica Support

**Mục đích:** Distribute read traffic across multiple PostgreSQL replicas.

```mermaid
flowchart TD
    Request[HTTP Request] -->|Route| PoolManager[Pool Manager]
    PoolManager -->|Read Query| ReplicaSelector[Replica Selector]
    PoolManager -->|Write Query| PrimaryPool[Primary Pool]
    
    ReplicaSelector -->|Round Robin| Replica1Pool[Replica 1 Pool]
    ReplicaSelector -->|Round Robin| Replica2Pool[Replica 2 Pool]
    
    Replica1Pool -->|SQL| Replica1[(Replica 1)]
    Replica2Pool -->|SQL| Replica2[(Replica 2)]
    PrimaryPool -->|SQL| Primary[(Primary DB)]
    
    HealthCheck[Health Check] -->|Monitor| Replica1
    HealthCheck -->|Monitor| Replica2
    HealthCheck -->|Update Status| ReplicaSelector
    
    style Primary fill:#4169E1
    style Replica1 fill:#87CEEB
    style Replica2 fill:#87CEEB
```

**Implementation:**

```rust
// Pool configuration
pub struct PoolConfig {
    pub primary: String,
    pub replicas: Vec<String>,
    pub read_strategy: ReadStrategy,
    pub pool_size: u32,
    pub health_check_interval: Duration,
    pub replica_lag_threshold: Duration,
}

pub enum ReadStrategy {
    RoundRobin,
    Random,
    LeastConnections,
}

// Pool manager
pub struct PoolManager {
    primary_pool: PgPool,
    replica_pools: Vec<PgPool>,
    strategy: ReadStrategy,
    health_status: Arc<RwLock<Vec<bool>>>,
    current_index: Arc<AtomicUsize>,
}

impl PoolManager {
    pub async fn new(config: PoolConfig) -> Result<Self> {
        let primary_pool = PgPoolOptions::new()
            .max_connections(config.pool_size)
            .connect(&config.primary)
            .await?;
        
        let mut replica_pools = Vec::new();
        for replica_url in &config.replicas {
            let pool = PgPoolOptions::new()
                .max_connections(config.pool_size)
                .connect(replica_url)
                .await?;
            replica_pools.push(pool);
        }
        
        let health_status = Arc::new(RwLock::new(vec![true; replica_pools.len()]));
        
        let manager = Self {
            primary_pool,
            replica_pools,
            strategy: config.read_strategy,
            health_status,
            current_index: Arc::new(AtomicUsize::new(0)),
        };
        
        // Start health check loop
        manager.start_health_check(config.health_check_interval);
        
        Ok(manager)
    }
    
    pub async fn execute_query<T>(
        &self,
        query_type: QueryType,
        f: impl FnOnce(&PgPool) -> BoxFuture<'_, Result<T>>,
    ) -> Result<T> {
        match query_type {
            QueryType::Read => {
                let pool = self.select_replica().await;
                f(pool).await
            }
            QueryType::Write => {
                f(&self.primary_pool).await
            }
        }
    }
    
    async fn select_replica(&self) -> &PgPool {
        let health_status = self.health_status.read().await;
        
        // Find healthy replicas
        let healthy_indices: Vec<usize> = health_status
            .iter()
            .enumerate()
            .filter(|(_, &healthy)| healthy)
            .map(|(i, _)| i)
            .collect();
        
        if healthy_indices.is_empty() {
            // Fallback to primary
            return &self.primary_pool;
        }
        
        // Apply strategy
        let index = match self.strategy {
            ReadStrategy::RoundRobin => {
                let current = self.current_index.fetch_add(1, Ordering::Relaxed);
                healthy_indices[current % healthy_indices.len()]
            }
            ReadStrategy::Random => {
                let mut rng = rand::thread_rng();
                healthy_indices[rng.gen_range(0..healthy_indices.len())]
            }
            ReadStrategy::LeastConnections => {
                // TODO: Track connection count per pool
                healthy_indices[0]
            }
        };
        
        &self.replica_pools[index]
    }
    
    fn start_health_check(&self, interval: Duration) {
        let replica_pools = self.replica_pools.clone();
        let health_status = self.health_status.clone();
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                
                let mut statuses = Vec::new();
                for pool in &replica_pools {
                    let healthy = check_replica_health(pool).await;
                    statuses.push(healthy);
                }
                
                *health_status.write().await = statuses;
            }
        });
    }
}

async fn check_replica_health(pool: &PgPool) -> bool {
    // Check if replica is up and not too far behind
    let result = sqlx::query_scalar::<_, i64>(
        "SELECT EXTRACT(EPOCH FROM (now() - pg_last_xact_replay_timestamp())) * 1000"
    )
    .fetch_one(pool)
    .await;
    
    match result {
        Ok(lag_ms) => lag_ms < 5000,  // Less than 5 seconds lag
        Err(_) => false,
    }
}
```

**Configuration:**

```yaml
database:
  primary: postgresql://user:pass@primary:5432/db
  replicas:
    - postgresql://user:pass@replica1:5432/db
    - postgresql://user:pass@replica2:5432/db
  read-strategy: round-robin
  pool-size: 10
  health-check-interval: 10  # seconds
  replica-lag-threshold: 5000  # milliseconds
```


#### 5.2 Query Cost Estimation

**Mục đích:** Prevent expensive queries from overloading database.

```mermaid
flowchart TD
    Request[HTTP Request] -->|Parse| QueryBuilder[Query Builder]
    QueryBuilder -->|Generate SQL| CostEstimator[Cost Estimator]
    CostEstimator -->|EXPLAIN| DB[(PostgreSQL)]
    DB -->|Cost| CostEstimator
    CostEstimator -->|Check Threshold| Decision{Cost OK?}
    Decision -->|Yes| Execute[Execute Query]
    Decision -->|No| Reject[413 Query Too Expensive]
    
    style CostEstimator fill:#FFD700
```

**Implementation:**

```rust
// Query cost configuration
pub struct QueryCostConfig {
    pub enabled: bool,
    pub max_cost: f64,
    pub exempt_roles: Vec<String>,
    pub per_endpoint: HashMap<String, f64>,
}

// Cost estimator
pub struct QueryCostEstimator {
    pool: PgPool,
    config: QueryCostConfig,
}

impl QueryCostEstimator {
    pub async fn estimate_cost(&self, query: &str) -> Result<f64> {
        let explain_query = format!("EXPLAIN (FORMAT JSON) {}", query);
        
        let result: serde_json::Value = sqlx::query_scalar(&explain_query)
            .fetch_one(&self.pool)
            .await?;
        
        // Extract total cost from EXPLAIN output
        let cost = result[0]["Plan"]["Total Cost"]
            .as_f64()
            .ok_or_else(|| anyhow!("Failed to extract cost"))?;
        
        Ok(cost)
    }
    
    pub async fn check_query_cost(
        &self,
        query: &str,
        role: &str,
        endpoint: &str,
    ) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        // Check if role is exempt
        if self.config.exempt_roles.contains(&role.to_string()) {
            return Ok(());
        }
        
        // Get max cost for this endpoint
        let max_cost = self.config.per_endpoint
            .get(endpoint)
            .copied()
            .unwrap_or(self.config.max_cost);
        
        // Estimate cost
        let cost = self.estimate_cost(query).await?;
        
        if cost > max_cost {
            return Err(QueryTooExpensiveError {
                estimated_cost: cost,
                max_cost,
            }.into());
        }
        
        Ok(())
    }
}

// Error type
#[derive(Debug, thiserror::Error)]
#[error("Query cost too high: estimated {estimated_cost}, maximum allowed {max_cost}")]
pub struct QueryTooExpensiveError {
    pub estimated_cost: f64,
    pub max_cost: f64,
}

// Middleware
pub async fn query_cost_middleware(
    State(estimator): State<Arc<QueryCostEstimator>>,
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response> {
    // Extract query from request
    let query = extract_query(&req)?;
    let role = extract_role(&req).unwrap_or("anonymous");
    let endpoint = req.uri().path();
    
    // Check cost
    estimator.check_query_cost(&query, role, endpoint).await?;
    
    // Execute request
    next.run(req).await
}
```

**Error Response:**

```json
{
  "error": {
    "code": "QUERY_TOO_EXPENSIVE",
    "message": "Query cost too high",
    "details": "Estimated cost: 15000, maximum allowed: 10000",
    "hint": "Try adding filters or reducing the result set"
  }
}
```

**Configuration:**

```yaml
query-cost:
  enabled: true
  max-cost: 10000
  exempt-roles:
    - admin
    - analytics
  per-endpoint:
    /rest/expensive_view: 50000
    /rest/users: 5000
```


## Data Flow Diagrams

### Request Flow với Các Layer Mới

```mermaid
sequenceDiagram
    participant Client
    participant Gateway as API Gateway
    participant Auth as Auth Middleware
    participant Cache as Cache Layer
    participant Trace as Tracing Layer
    participant Plugin as Plugin System
    participant Handler as Request Handler
    participant Cost as Cost Estimator
    participant Pool as Pool Manager
    participant DB as PostgreSQL
    participant Redis
    participant Jaeger
    
    Client->>Gateway: HTTP Request
    Gateway->>Gateway: Rate Limit Check
    Gateway->>Redis: Check Rate Limit
    Redis-->>Gateway: OK
    
    Gateway->>Auth: Forward Request
    Auth->>Redis: Check Token Revocation
    Redis-->>Auth: Not Revoked
    Auth->>Auth: Validate JWT
    
    Auth->>Cache: Forward Request
    Cache->>Redis: Check Cache
    alt Cache Hit
        Redis-->>Cache: Cached Response
        Cache-->>Client: Return Response
    else Cache Miss
        Cache->>Trace: Forward Request
        Trace->>Jaeger: Create Span
        
        Trace->>Plugin: Forward Request
        Plugin->>Plugin: Execute Plugins
        
        Plugin->>Handler: Forward Request
        Handler->>Handler: Build Query
        
        Handler->>Cost: Estimate Cost
        Cost->>DB: EXPLAIN Query
        DB-->>Cost: Cost Estimate
        Cost-->>Handler: Cost OK
        
        Handler->>Pool: Execute Query
        Pool->>Pool: Select Pool (Read/Write)
        Pool->>DB: Execute SQL
        DB-->>Pool: Result
        Pool-->>Handler: Result
        
        Handler->>Cache: Store in Cache
        Cache->>Redis: Store Response
        
        Handler->>Trace: Complete Span
        Trace->>Jaeger: Export Span
        
        Handler-->>Client: Return Response
    end
```


### Auth Flow với Token Revocation

```mermaid
sequenceDiagram
    participant Client
    participant Auth as Auth Middleware
    participant Redis
    participant JWT as JWT Validator
    participant Handler
    participant Admin
    
    Note over Client,Handler: Normal Request Flow
    Client->>Auth: Request with JWT
    Auth->>Redis: Check if Token Revoked
    Redis-->>Auth: Not Revoked
    Auth->>JWT: Validate JWT
    JWT-->>Auth: Valid
    Auth->>Handler: Forward Request
    Handler-->>Client: Response
    
    Note over Admin,Redis: Token Revocation Flow
    Admin->>Admin: POST /admin/revoke-token
    Admin->>Redis: Store Revoked Token
    Redis-->>Admin: OK
    
    Note over Client,Handler: Subsequent Request with Revoked Token
    Client->>Auth: Request with Revoked JWT
    Auth->>Redis: Check if Token Revoked
    Redis-->>Auth: Revoked!
    Auth-->>Client: 401 Unauthorized
```

### Caching Flow

```mermaid
sequenceDiagram
    participant Client
    participant Cache as Cache Middleware
    participant Redis
    participant Handler
    participant DB
    
    Note over Client,DB: Cache Miss Flow
    Client->>Cache: GET /rest/users
    Cache->>Redis: Check Cache Key
    Redis-->>Cache: Miss
    Cache->>Handler: Forward Request
    Handler->>DB: Query
    DB-->>Handler: Result
    Handler-->>Cache: Response
    Cache->>Redis: Store (TTL=60s)
    Cache-->>Client: Response
    
    Note over Client,DB: Cache Hit Flow
    Client->>Cache: GET /rest/users
    Cache->>Redis: Check Cache Key
    Redis-->>Cache: Hit!
    Cache-->>Client: Cached Response
    
    Note over Client,DB: Cache Invalidation on Mutation
    Client->>Handler: POST /rest/users
    Handler->>DB: Insert
    DB-->>Handler: Success
    Handler->>Redis: Invalidate Pattern "/rest/users*"
    Handler-->>Client: Response
```

### Monitoring & Tracing Flow

```mermaid
sequenceDiagram
    participant Client
    participant Gateway
    participant Trace as Tracing Layer
    participant Handler
    participant DB
    participant Jaeger
    participant Prometheus
    participant Loki
    
    Client->>Gateway: HTTP Request (traceparent header)
    Gateway->>Trace: Extract Trace Context
    Trace->>Trace: Create Root Span
    Trace->>Jaeger: Start Trace
    
    Trace->>Handler: Forward Request
    Handler->>Handler: Create DB Span
    Handler->>DB: Execute Query
    DB-->>Handler: Result
    Handler->>Handler: End DB Span
    
    Handler->>Prometheus: Record Metrics
    Note over Prometheus: request_count++<br/>request_duration.observe()
    
    Handler->>Loki: Write Structured Log
    Note over Loki: JSON log with trace_id
    
    Handler-->>Trace: Response
    Trace->>Trace: End Root Span
    Trace->>Jaeger: Export Complete Trace
    
    Trace-->>Client: Response (with trace_id)
```


## Integration Points

### 1. PostgreSQL Integration

**Primary Database:**
- Write operations (INSERT, UPDATE, DELETE)
- Schema introspection
- RLS policy enforcement
- Stored procedures (RPC)

**Read Replicas:**
- Read operations (SELECT)
- Health checking
- Replication lag monitoring

**Connection:**
```rust
// Primary connection
let primary_pool = PgPoolOptions::new()
    .max_connections(10)
    .connect("postgresql://user:pass@primary:5432/db")
    .await?;

// Replica connections
let replica_pools = vec![
    PgPoolOptions::new()
        .max_connections(10)
        .connect("postgresql://user:pass@replica1:5432/db")
        .await?,
    PgPoolOptions::new()
        .max_connections(10)
        .connect("postgresql://user:pass@replica2:5432/db")
        .await?,
];
```

### 2. Redis Integration

**Use Cases:**
- Response caching
- Token revocation list
- Rate limiting state
- Session storage (future)

**Connection:**
```rust
use redis::Client;

let redis_client = Client::open("redis://localhost:6379")?;
let mut con = redis_client.get_connection()?;

// Cache operations
con.set_ex("cache:key", "value", 300)?;  // TTL 300s
let value: Option<String> = con.get("cache:key")?;

// Rate limiting
con.zadd("ratelimit:ip:1.2.3.4", timestamp, timestamp)?;
con.expire("ratelimit:ip:1.2.3.4", 60)?;

// Token revocation
con.set_ex("revoked:token123", "1", 3600)?;
let revoked: bool = con.exists("revoked:token123")?;
```

**Configuration:**
```yaml
redis:
  url: redis://localhost:6379
  pool-size: 10
  timeout: 5s
  max-retries: 3
```


### 3. Monitoring Systems Integration

#### Prometheus Integration

**Metrics Endpoint:**
```rust
use axum::{Router, routing::get};
use prometheus::{Encoder, TextEncoder};

async fn metrics_handler(
    State(registry): State<Arc<Registry>>,
) -> Result<String> {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(String::from_utf8(buffer)?)
}

let app = Router::new()
    .route("/metrics", get(metrics_handler))
    .with_state(registry);
```

**Prometheus Configuration:**
```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'evobase'
    scrape_interval: 15s
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: /metrics
```

#### Jaeger Integration

**Trace Export:**
```rust
use opentelemetry_jaeger::JaegerPipeline;

let tracer = JaegerPipeline::new()
    .with_service_name("evobase")
    .with_agent_endpoint("localhost:6831")
    .install_batch(opentelemetry::runtime::Tokio)?;
```

**Jaeger Configuration:**
```yaml
# docker-compose.yml
jaeger:
  image: jaegertracing/all-in-one:latest
  ports:
    - "6831:6831/udp"  # Agent
    - "16686:16686"    # UI
```

#### Loki Integration

**Log Shipping:**
```yaml
# promtail.yml
clients:
  - url: http://localhost:3100/loki/api/v1/push

scrape_configs:
  - job_name: evobase
    static_configs:
      - targets:
          - localhost
        labels:
          job: evobase
          __path__: /var/log/evobase/*.log
```

### 4. External Systems Integration (Plugins)

**Plugin Interface:**
```rust
// Example: OAuth2 authentication plugin
pub struct OAuth2Plugin {
    client_id: String,
    client_secret: String,
    provider_url: String,
}

#[async_trait]
impl Plugin for OAuth2Plugin {
    fn name(&self) -> &str {
        "oauth2-auth"
    }
    
    fn init(&mut self, config: &PluginConfig) -> Result<()> {
        self.client_id = config.get("client_id")?;
        self.client_secret = config.get("client_secret")?;
        self.provider_url = config.get("provider_url")?;
        Ok(())
    }
    
    fn middleware(&self) -> Option<Box<dyn Middleware>> {
        Some(Box::new(OAuth2Middleware::new(self)))
    }
}

// Example: Elasticsearch audit log plugin
pub struct ElasticsearchAuditPlugin {
    client: elasticsearch::Elasticsearch,
}

#[async_trait]
impl Plugin for ElasticsearchAuditPlugin {
    fn name(&self) -> &str {
        "elasticsearch-audit"
    }
    
    async fn on_request_complete(&self, entry: AuditLogEntry) -> Result<()> {
        self.client
            .index(IndexParts::Index("audit-logs"))
            .body(entry)
            .send()
            .await?;
        Ok(())
    }
}
```


## Technology Stack

### Backend

**Core:**
- **Language:** Rust 1.75+
- **Framework:** Axum 0.7
- **Async Runtime:** Tokio 1.35
- **Database Driver:** SQLx 0.7 (PostgreSQL)

**Security:**
- **JWT:** jsonwebtoken 9.2
- **Crypto:** ring 0.17
- **Password Hashing:** argon2 0.5

**Caching & Storage:**
- **Redis Client:** redis 0.24
- **Cache:** moka 0.12 (in-memory fallback)

**Observability:**
- **Tracing:** tracing 0.1, tracing-subscriber 0.3
- **OpenTelemetry:** opentelemetry 0.21, opentelemetry-jaeger 0.20
- **Metrics:** prometheus 0.13

**Serialization:**
- **JSON:** serde 1.0, serde_json 1.0
- **YAML:** serde_yaml 0.9

**Utilities:**
- **Error Handling:** anyhow 1.0, thiserror 1.0
- **Config:** config 0.13
- **UUID:** uuid 1.6
- **DateTime:** chrono 0.4

### Database

**Primary:**
- **PostgreSQL:** 15+ (recommended 16+)
- **Extensions:** pgcrypto, uuid-ossp

**Replication:**
- **Streaming Replication:** Built-in PostgreSQL
- **Logical Replication:** For multi-region (future)

### Cache & Message Queue

**Redis:**
- **Version:** 7.0+
- **Deployment:** Standalone or Sentinel
- **Persistence:** RDB + AOF

### Monitoring Stack

**Metrics:**
- **Prometheus:** 2.45+
- **Grafana:** 10.0+

**Tracing:**
- **Jaeger:** 1.50+

**Logging:**
- **Loki:** 2.9+
- **Promtail:** 2.9+

### Frontend (Existing)

**Workbench:**
- **Flutter:** 3.41.5
- **Dart:** 3.11.3
- **State Management:** flutter_bloc 8.1

### Development Tools

**Build & Test:**
- **Cargo:** 1.75+
- **cargo-watch:** 8.4 (development)
- **cargo-nextest:** 0.9 (testing)

**Code Quality:**
- **clippy:** Rust linter
- **rustfmt:** Code formatter
- **cargo-audit:** Security audit

**Database:**
- **sqlx-cli:** 0.7 (migrations)
- **pgAdmin:** 4.0+ (administration)


## Phased Implementation Plan

### Phase 1: Foundation & Modularity (Tháng 1-3)

**Mục tiêu:** Tạo nền tảng cho extensibility và modularity.

**Tasks:**

1. **Extract Schema Cache Interface** (2 tuần)
   - Define `SchemaCache` trait
   - Implement in-memory cache
   - Implement Redis cache (optional)
   - Add cache invalidation via LISTEN/NOTIFY

2. **Extract Query Builder as Library** (2 tuần)
   - Create `evobase-query-builder` crate
   - Move query building logic
   - Define clean API
   - Add comprehensive tests

3. **Define Plugin System** (3 tuần)
   - Design plugin interface
   - Implement plugin registry
   - Add plugin loading from directory
   - Create example plugins
   - Document plugin development

4. **Refactor Middleware Chain** (1 tuần)
   - Modularize existing middleware
   - Add middleware composition
   - Support plugin middleware

**Deliverables:**
- [ ] Schema cache interface implemented
- [ ] Query builder extracted to separate crate
- [ ] Plugin system functional
- [ ] Documentation for plugin development
- [ ] Migration guide for existing deployments

**Success Criteria:**
- All existing tests pass
- No performance regression (< 5% overhead)
- Plugin system documented with examples
- At least 2 example plugins created

**Risks:**
- Breaking changes to internal APIs → Mitigate with semantic versioning
- Performance overhead → Mitigate with benchmarking


### Phase 2: Security Enhancements (Tháng 4-6)

**Mục tiêu:** Tăng cường security với token revocation, rate limiting, audit logging.

**Tasks:**

1. **Token Revocation System** (2 tuần)
   - Define `TokenRevocation` trait
   - Implement Redis backend
   - Implement in-memory backend (testing)
   - Add admin endpoints
   - Add revocation check to auth middleware

2. **Rate Limiting** (2 tuần)
   - Define `RateLimiter` trait
   - Implement Redis sliding window
   - Add rate limit middleware
   - Add response headers
   - Configure per-endpoint limits

3. **Audit Logging** (3 tuần)
   - Design audit log schema
   - Implement `AuditLogger` trait
   - Add database backend
   - Add file backend (optional)
   - Add audit middleware
   - Create query API

4. **Field-Level Encryption** (Optional, 1 tuần)
   - Design encryption interface
   - Implement transparent encryption
   - Add key management

**Deliverables:**
- [ ] Token revocation working
- [ ] Rate limiting prevents DoS
- [ ] Audit logging meets compliance
- [ ] Security audit completed

**Success Criteria:**
- Token revocation latency < 10ms
- Rate limiting accurate across distributed instances
- Audit logging captures all mutations
- Security audit passes

**Risks:**
- Redis dependency → Mitigate with in-memory fallback
- Performance impact → Mitigate with async logging


### Phase 3: Performance & Scalability (Tháng 7-9)

**Mục tiêu:** Cải thiện performance với caching, read replicas, query cost estimation.

**Tasks:**

1. **Response Caching** (3 tuần)
   - Define `ResponseCache` trait
   - Implement Redis backend
   - Add cache middleware
   - Implement cache invalidation
   - Add HTTP cache headers (ETag, Last-Modified)
   - Configure cache rules

2. **Read Replica Support** (3 tuần)
   - Design `PoolManager`
   - Implement replica routing
   - Add health checking
   - Monitor replication lag
   - Automatic failover to primary

3. **Query Cost Estimation** (2 tuần)
   - Implement cost estimator
   - Add cost check middleware
   - Configure per-endpoint limits
   - Add role-based exemptions

4. **Connection Pooling Optimization** (1 tuần)
   - Tune pool sizes
   - Add pool metrics
   - Optimize connection lifecycle

**Deliverables:**
- [ ] Cache hit rate > 60% for cacheable endpoints
- [ ] Read replicas reduce primary load by 40%+
- [ ] Query cost estimation prevents expensive queries
- [ ] Performance improved by 30%+ for cached endpoints

**Success Criteria:**
- Cache lookup latency < 5ms
- Read traffic distributed across replicas
- Expensive queries blocked before execution
- No performance regression for uncached requests

**Risks:**
- Cache invalidation complexity → Mitigate with pattern-based invalidation
- Replica lag → Mitigate with health checking


### Phase 4: Observability (Tháng 10-12)

**Mục tiêu:** Tăng observability với distributed tracing, enhanced metrics, structured logging.

**Tasks:**

1. **Distributed Tracing** (3 tuần)
   - Integrate OpenTelemetry
   - Add Jaeger exporter
   - Implement tracing middleware
   - Add database query tracing
   - Trace context propagation

2. **Enhanced Metrics** (2 tuần)
   - Expand metrics collection
   - Add detailed labels
   - Create Grafana dashboards
   - Define alerting rules

3. **Structured Logging** (2 tuần)
   - Implement JSON logging
   - Add log correlation with traces
   - Configure log levels
   - Add log sampling

4. **Health Checks & Readiness Probes** (1 tuần)
   - Implement `/health` endpoint
   - Implement `/ready` endpoint
   - Add dependency health checks

**Deliverables:**
- [ ] End-to-end request tracing working
- [ ] Comprehensive metrics dashboard
- [ ] Structured logging enables efficient analysis
- [ ] Mean time to resolution (MTTR) reduced by 50%

**Success Criteria:**
- Trace overhead < 5ms per request
- All critical metrics exposed
- Logs machine-readable (JSON)
- Grafana dashboards operational

**Risks:**
- Tracing overhead → Mitigate with sampling
- Log volume → Mitigate with sampling and filtering


### Phase 5: Advanced Features (Tháng 13-18)

**Mục tiêu:** Thêm advanced features: GraphQL, WebSocket, multi-tenancy.

**Tasks:**

1. **GraphQL Support** (6 tuần)
   - Generate GraphQL schema from database
   - Implement query resolver
   - Implement mutation resolver
   - Add GraphQL Playground
   - Ensure feature parity with REST

2. **WebSocket Support** (4 tuần)
   - Implement WebSocket server
   - Add subscription management
   - Integrate with PostgreSQL LISTEN/NOTIFY
   - Create client library (JavaScript)

3. **Multi-Tenancy Support** (4 tuần)
   - Design tenant identification strategies
   - Implement schema-based isolation
   - Implement column-based isolation with RLS
   - Add tenant context propagation

4. **Batch Operations Optimization** (2 tuần)
   - Optimize bulk insert/update/delete
   - Add batch query support
   - Implement transaction batching

**Deliverables:**
- [ ] GraphQL feature parity with REST
- [ ] WebSocket supports 10,000+ concurrent connections
- [ ] Multi-tenancy prevents cross-tenant data leakage
- [ ] Advanced features adopted by 20%+ of users

**Success Criteria:**
- GraphQL performance comparable to REST
- WebSocket sub-second notification latency
- Complete tenant isolation
- No cross-tenant data leakage

**Risks:**
- GraphQL complexity → Mitigate with clear documentation
- WebSocket scalability → Mitigate with connection limits


## Dependencies Between Phases

```mermaid
graph TD
    P1[Phase 1: Foundation] --> P2[Phase 2: Security]
    P1 --> P3[Phase 3: Performance]
    P1 --> P4[Phase 4: Observability]
    
    P2 --> P5[Phase 5: Advanced Features]
    P3 --> P5
    P4 --> P5
    
    P1 -.->|Plugin System| P2
    P1 -.->|Plugin System| P3
    P1 -.->|Plugin System| P4
    
    P3 -.->|Caching| P5
    P4 -.->|Tracing| P5
    
    style P1 fill:#90EE90
    style P2 fill:#FFD700
    style P3 fill:#FF6347
    style P4 fill:#00CED1
    style P5 fill:#9370DB
```

**Critical Path:**
1. Phase 1 (Foundation) → Prerequisite cho tất cả phases khác
2. Phase 2, 3, 4 có thể parallel sau Phase 1
3. Phase 5 (Advanced Features) phụ thuộc vào Phase 2, 3, 4

**Parallel Execution:**
- Phase 2 (Security) và Phase 3 (Performance) có thể parallel
- Phase 4 (Observability) có thể parallel với Phase 2 và 3
- Phase 5 chỉ bắt đầu sau khi Phase 2, 3, 4 hoàn thành


## Timeline Estimates

### Gantt Chart

```mermaid
gantt
    title EvoBase Architecture Enhancements Timeline
    dateFormat YYYY-MM
    section Phase 1
    Foundation & Modularity           :p1, 2024-04, 3M
    section Phase 2
    Security Enhancements             :p2, 2024-07, 3M
    section Phase 3
    Performance & Scalability         :p3, 2024-07, 3M
    section Phase 4
    Observability                     :p4, 2024-10, 3M
    section Phase 5
    Advanced Features                 :p5, 2025-01, 6M
```

### Detailed Timeline

| Phase | Duration | Start | End | Team Size |
|-------|----------|-------|-----|-----------|
| Phase 1: Foundation | 3 tháng | Tháng 1 | Tháng 3 | 2 devs |
| Phase 2: Security | 3 tháng | Tháng 4 | Tháng 6 | 2 devs |
| Phase 3: Performance | 3 tháng | Tháng 4 | Tháng 6 | 2 devs |
| Phase 4: Observability | 3 tháng | Tháng 7 | Tháng 9 | 2 devs |
| Phase 5: Advanced Features | 6 tháng | Tháng 10 | Tháng 15 | 3 devs |

**Total Duration:** 15 tháng (có overlap)

**Team Requirements:**
- 2 backend developers (Rust)
- 1 DevOps engineer (Phase 4+)
- 1 frontend developer (Flutter workbench updates)
- 1 QA engineer (testing)


## Risk Assessment

### Technical Risks

| Risk | Severity | Probability | Impact | Mitigation |
|------|----------|-------------|--------|------------|
| Performance Regression | 🔴 High | 🟡 Medium | 🔴 High | Comprehensive benchmarking, performance budgets (< 5% overhead) |
| Breaking Changes | 🔴 High | 🟡 Medium | 🔴 High | Semantic versioning, deprecation warnings, compatibility layer |
| Increased Complexity | 🟡 Medium | 🔴 High | 🟡 Medium | Modular architecture, complexity in optional plugins |
| Redis Dependency | 🟡 Medium | 🟢 Low | 🟡 Medium | In-memory fallback, graceful degradation |
| Schema Cache Staleness | 🟡 Medium | 🟡 Medium | 🟡 Medium | Automatic NOTIFY, manual reload endpoint |

### Operational Risks

| Risk | Severity | Probability | Impact | Mitigation |
|------|----------|-------------|--------|------------|
| Migration Complexity | 🟡 Medium | 🟡 Medium | 🟡 Medium | Automated migration tools, detailed guides |
| Resource Constraints | 🟡 Medium | 🟡 Medium | 🟡 Medium | Phased approach, community contributions |
| Community Resistance | 🟢 Low | 🟡 Medium | 🟡 Medium | Early RFC process, beta releases |

### Security Risks

| Risk | Severity | Probability | Impact | Mitigation |
|------|----------|-------------|--------|------------|
| New Attack Vectors | 🔴 High | 🟡 Medium | 🔴 High | Security review, penetration testing, bug bounty |
| Dependency Vulnerabilities | 🟡 Medium | 🟡 Medium | 🟡 Medium | Automated scanning, regular updates |
| JWT Secret Exposure | 🔴 High | 🟢 Low | 🔴 High | Secure storage, rotation policy |

### Rollback Plans

**Phase 1-2:**
- Revert to previous version if critical issues
- Feature flags to disable problematic features
- Maintain LTS version with security fixes

**Phase 3-4:**
- Disable caching/tracing if performance issues
- Fallback to primary DB if replica issues
- Reduce sampling rate if overhead too high

**Phase 5:**
- GraphQL/WebSocket optional, can be disabled
- Multi-tenancy opt-in, no impact on single-tenant


## Configuration Management

### Feature Flags

```yaml
# config/features.yaml
features:
  # Phase 1
  plugin-system: true
  schema-cache-redis: false
  
  # Phase 2
  token-revocation: false
  rate-limiting: false
  audit-logging: false
  field-encryption: false
  
  # Phase 3
  response-caching: false
  read-replicas: false
  query-cost-estimation: false
  
  # Phase 4
  distributed-tracing: false
  enhanced-metrics: true
  structured-logging: true
  
  # Phase 5
  graphql: false
  websocket: false
  multi-tenancy: false
```

### Environment-Specific Configuration

```yaml
# config/development.yaml
database:
  primary: postgresql://localhost:5432/evobase_dev
  replicas: []

redis:
  url: redis://localhost:6379

logging:
  level: debug
  format: text

tracing:
  enabled: false

# config/production.yaml
database:
  primary: postgresql://primary.prod:5432/evobase
  replicas:
    - postgresql://replica1.prod:5432/evobase
    - postgresql://replica2.prod:5432/evobase

redis:
  url: redis://redis.prod:6379
  pool-size: 20

logging:
  level: info
  format: json

tracing:
  enabled: true
  sample-rate: 0.1

caching:
  enabled: true
  default-ttl: 300

rate-limiting:
  enabled: true
  per-ip:
    max-requests: 100
    window: 60
```

### Configuration Validation

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct AppConfig {
    #[validate(nested)]
    pub database: DatabaseConfig,
    
    #[validate(nested)]
    pub redis: Option<RedisConfig>,
    
    #[validate(nested)]
    pub features: FeatureFlags,
    
    #[validate(nested)]
    pub security: SecurityConfig,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::File::with_name(&format!(
                "config/{}",
                std::env::var("ENV").unwrap_or_else(|_| "development".to_string())
            )))
            .add_source(config::Environment::with_prefix("EVOBASE"))
            .build()?;
        
        let app_config: AppConfig = config.try_deserialize()?;
        app_config.validate()?;
        
        Ok(app_config)
    }
}
```


## Testing Strategy

### Unit Testing

**Coverage Target:** 80%+

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_token_revocation() {
        let revocation = InMemoryTokenRevocation::new();
        let token = "test_token";
        let expiry = Utc::now() + Duration::seconds(3600);
        
        // Revoke token
        revocation.revoke(token, expiry).await.unwrap();
        
        // Check revocation
        assert!(revocation.is_revoked(token).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_rate_limiting() {
        let limiter = InMemoryRateLimiter::new();
        let rule = RateLimitRule {
            window: Duration::seconds(60),
            max_requests: 10,
        };
        
        // Make 10 requests (should succeed)
        for _ in 0..10 {
            let result = limiter.check("test_key", &rule).await.unwrap();
            assert!(result.allowed);
        }
        
        // 11th request should fail
        let result = limiter.check("test_key", &rule).await.unwrap();
        assert!(!result.allowed);
    }
}
```

### Integration Testing

**Test Database:** Docker container với PostgreSQL

```rust
#[tokio::test]
async fn test_query_with_caching() {
    let test_db = TestDatabase::new().await;
    let redis = TestRedis::new().await;
    let app = create_test_app(test_db.pool(), redis.client()).await;
    
    // First request (cache miss)
    let response1 = app
        .oneshot(Request::builder()
            .uri("/rest/users")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    assert_eq!(response1.status(), StatusCode::OK);
    assert_eq!(response1.headers().get("X-Cache"), Some(&"MISS".parse().unwrap()));
    
    // Second request (cache hit)
    let response2 = app
        .oneshot(Request::builder()
            .uri("/rest/users")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    assert_eq!(response2.status(), StatusCode::OK);
    assert_eq!(response2.headers().get("X-Cache"), Some(&"HIT".parse().unwrap()));
}
```

### Performance Testing

**Load Testing với k6:**

```javascript
// load-test.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export let options = {
  stages: [
    { duration: '2m', target: 100 },  // Ramp up to 100 users
    { duration: '5m', target: 100 },  // Stay at 100 users
    { duration: '2m', target: 0 },    // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'],  // 95% of requests < 500ms
    http_req_failed: ['rate<0.01'],    // Error rate < 1%
  },
};

export default function () {
  let response = http.get('http://localhost:3000/rest/users');
  
  check(response, {
    'status is 200': (r) => r.status === 200,
    'response time < 500ms': (r) => r.timings.duration < 500,
  });
  
  sleep(1);
}
```

### Security Testing

**Penetration Testing Checklist:**
- [ ] SQL injection attempts
- [ ] JWT token manipulation
- [ ] Rate limit bypass attempts
- [ ] Authorization bypass attempts
- [ ] XSS/CSRF attacks
- [ ] DoS attacks

**Automated Security Scanning:**
```bash
# Dependency audit
cargo audit

# Security linting
cargo clippy -- -W clippy::all

# OWASP ZAP scan
zap-cli quick-scan http://localhost:3000
```


## Deployment Strategy

### Docker Compose (Development)

```yaml
# docker-compose.yml
version: '3.8'

services:
  evobase:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgresql://postgres:password@postgres:5432/evobase
      - REDIS_URL=redis://redis:6379
      - ENV=development
    depends_on:
      - postgres
      - redis
      - jaeger
    volumes:
      - ./config:/app/config

  postgres:
    image: postgres:16
    environment:
      - POSTGRES_PASSWORD=password
      - POSTGRES_DB=evobase
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "6831:6831/udp"
      - "16686:16686"

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana

volumes:
  postgres_data:
  redis_data:
  prometheus_data:
  grafana_data:
```

### Kubernetes (Production)

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: evobase
spec:
  replicas: 3
  selector:
    matchLabels:
      app: evobase
  template:
    metadata:
      labels:
        app: evobase
    spec:
      containers:
      - name: evobase
        image: evobase:latest
        ports:
        - containerPort: 3000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: evobase-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: evobase-secrets
              key: redis-url
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: evobase
spec:
  selector:
    app: evobase
  ports:
  - port: 80
    targetPort: 3000
  type: LoadBalancer
```

### CI/CD Pipeline

```yaml
# .github/workflows/ci.yml
name: CI/CD

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all-features
      - name: Run clippy
        run: cargo clippy -- -D warnings
      - name: Run security audit
        run: cargo audit

  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build Docker image
        run: docker build -t evobase:${{ github.sha }} .
      - name: Push to registry
        run: docker push evobase:${{ github.sha }}

  deploy:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - name: Deploy to Kubernetes
        run: kubectl set image deployment/evobase evobase=evobase:${{ github.sha }}
```


## Monitoring & Alerting

### Grafana Dashboards

**Dashboard 1: System Overview**
- Request rate (requests/sec)
- Response time (p50, p95, p99)
- Error rate (%)
- Active connections
- Cache hit rate

**Dashboard 2: Database Performance**
- Query duration (p50, p95, p99)
- Connection pool utilization
- Replica lag
- Slow queries
- Transaction rate

**Dashboard 3: Security Metrics**
- Authentication failures
- Rate limit violations
- Token revocations
- Audit log entries
- Suspicious activity

### Alerting Rules

```yaml
# prometheus/alerts.yml
groups:
  - name: evobase
    interval: 30s
    rules:
      # High error rate
      - alert: HighErrorRate
        expr: rate(evobase_http_requests_total{status=~"5.."}[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} (threshold: 0.05)"

      # High response time
      - alert: HighResponseTime
        expr: histogram_quantile(0.95, rate(evobase_http_request_duration_seconds_bucket[5m])) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High response time detected"
          description: "P95 response time is {{ $value }}s (threshold: 1s)"

      # Database connection pool exhausted
      - alert: DatabasePoolExhausted
        expr: evobase_db_connections_active / evobase_db_connections_max > 0.9
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Database connection pool nearly exhausted"
          description: "Pool utilization is {{ $value }} (threshold: 0.9)"

      # High replica lag
      - alert: HighReplicaLag
        expr: evobase_db_replica_lag_seconds > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High replica lag detected"
          description: "Replica lag is {{ $value }}s (threshold: 10s)"

      # Cache unavailable
      - alert: CacheUnavailable
        expr: up{job="redis"} == 0
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "Redis cache unavailable"
          description: "Redis has been down for 2 minutes"
```

### On-Call Runbook

**High Error Rate:**
1. Check Grafana dashboard for error patterns
2. Check Loki logs for error details
3. Check Jaeger traces for failed requests
4. Identify root cause (database, cache, external service)
5. Apply fix or rollback

**High Response Time:**
1. Check database query performance
2. Check cache hit rate
3. Check replica lag
4. Identify slow queries
5. Optimize or add indexes

**Database Pool Exhausted:**
1. Check active connections
2. Check for long-running queries
3. Increase pool size if needed
4. Kill long-running queries if necessary


## Migration Guide

### Upgrading from Current EvoBase

**Step 1: Backup**
```bash
# Backup database
pg_dump evobase > evobase_backup.sql

# Backup configuration
cp -r config config.backup
```

**Step 2: Update Dependencies**
```bash
# Update Rust dependencies
cargo update

# Update Docker images
docker-compose pull
```

**Step 3: Run Migrations**
```bash
# Run database migrations
sqlx migrate run

# Verify migrations
sqlx migrate info
```

**Step 4: Update Configuration**
```yaml
# Add new configuration sections
features:
  plugin-system: true
  token-revocation: false  # Enable gradually
  rate-limiting: false
  response-caching: false

redis:
  url: redis://localhost:6379  # Add Redis if using new features
```

**Step 5: Deploy**
```bash
# Development
docker-compose up -d

# Production
kubectl apply -f k8s/
```

**Step 6: Verify**
```bash
# Check health
curl http://localhost:3000/health

# Check metrics
curl http://localhost:3000/metrics

# Check logs
docker-compose logs -f evobase
```

### Rollback Procedure

**If issues occur:**
```bash
# Stop new version
docker-compose down

# Restore database
psql evobase < evobase_backup.sql

# Restore configuration
cp -r config.backup config

# Start old version
docker-compose -f docker-compose.old.yml up -d
```

### Breaking Changes

**Phase 1:**
- None (backward compatible)

**Phase 2:**
- Token revocation requires Redis (optional)
- Rate limiting requires Redis (optional)

**Phase 3:**
- Read replicas require configuration changes
- Caching requires Redis

**Phase 4:**
- Tracing requires Jaeger (optional)
- Structured logging changes log format

**Phase 5:**
- GraphQL endpoint is new (opt-in)
- WebSocket endpoint is new (opt-in)
- Multi-tenancy requires schema changes (opt-in)


## Success Metrics

### Phase 1: Foundation & Modularity

| Metric | Target | Measurement |
|--------|--------|-------------|
| Test Coverage | 80%+ | `cargo tarpaulin` |
| Performance Overhead | < 5% | Benchmark comparison |
| Plugin Examples | 2+ | Count in `/plugins` |
| Documentation Pages | 10+ | Count in `/docs` |

### Phase 2: Security Enhancements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Token Revocation Latency | < 10ms | Prometheus metrics |
| Rate Limit Accuracy | 99%+ | Load testing |
| Audit Log Coverage | 100% mutations | Database query |
| Security Audit Score | Pass | External audit |

### Phase 3: Performance & Scalability

| Metric | Target | Measurement |
|--------|--------|-------------|
| Cache Hit Rate | > 60% | Prometheus metrics |
| Primary Load Reduction | > 40% | Database monitoring |
| Query Cost Rejections | > 0 | Prometheus counter |
| P95 Response Time | < 200ms | Prometheus histogram |

### Phase 4: Observability

| Metric | Target | Measurement |
|--------|--------|-------------|
| Trace Coverage | 100% endpoints | Jaeger UI |
| Trace Overhead | < 5ms | Benchmark |
| Metrics Count | 50+ | Prometheus |
| Log Parsing Success | 100% | Loki query |

### Phase 5: Advanced Features

| Metric | Target | Measurement |
|--------|--------|-------------|
| GraphQL Feature Parity | 100% | Feature comparison |
| WebSocket Connections | 10,000+ | Load testing |
| Tenant Isolation | 100% | Security testing |
| Feature Adoption | 20%+ | Usage analytics |

### Overall Success Criteria

**Performance:**
- ✅ No regression in baseline performance
- ✅ 30%+ improvement for cached endpoints
- ✅ 40%+ reduction in primary database load

**Reliability:**
- ✅ 99.9% uptime
- ✅ < 1% error rate
- ✅ Automatic failover working

**Security:**
- ✅ Zero security vulnerabilities
- ✅ Audit logging compliant
- ✅ Token revocation working

**Observability:**
- ✅ End-to-end tracing
- ✅ Comprehensive metrics
- ✅ Structured logging

**Adoption:**
- ✅ 20%+ users enable advanced features
- ✅ Positive community feedback
- ✅ Active plugin ecosystem


## Conclusion

### Summary

Design document này đề xuất một kế hoạch toàn diện để nâng cấp EvoBase từ một REST API server cơ bản thành một production-ready system với khả năng mở rộng, bảo mật, và observability cao.

**Những gì được giữ nguyên:**
- ✅ Database-centric philosophy
- ✅ Zero-code API generation từ schema
- ✅ Stateless architecture
- ✅ High performance
- ✅ Simplicity for basic use cases
- ✅ Backward compatibility

**Những gì được nâng cấp:**
- 🚀 Extensibility via plugin system
- 🔒 Enhanced security (token revocation, rate limiting, audit logging)
- ⚡ Better performance (caching, read replicas, query cost estimation)
- 📊 Comprehensive observability (tracing, metrics, logging)
- 🎯 Advanced features (GraphQL, WebSocket, multi-tenancy)

### Key Takeaways

1. **Phased Approach:** 5 phases trong 15-18 tháng, có thể parallel execution
2. **Opt-In Features:** Tất cả features mới đều optional, không breaking changes
3. **Production-Ready:** Focus vào reliability, security, và observability
4. **Community-Driven:** Plugin system cho phép community mở rộng
5. **PostgREST-Inspired:** Học hỏi từ PostgREST architecture nhưng adapt cho Rust/Axum

### Next Steps

**Immediate (Tuần 1-2):**
1. Review và approve design document
2. Set up project structure
3. Create GitHub issues cho Phase 1 tasks
4. Assign team members

**Short-term (Tháng 1-3):**
1. Implement Phase 1 (Foundation & Modularity)
2. Create plugin examples
3. Write documentation
4. Set up CI/CD pipeline

**Long-term (Tháng 4-18):**
1. Execute Phase 2-5 theo timeline
2. Continuous testing và monitoring
3. Community feedback và iteration
4. Production deployment

### Questions & Feedback

**Open Questions:**
- Redis clustering strategy cho production?
- GraphQL schema generation approach?
- Multi-tenancy: schema-based hay column-based?
- Plugin distribution mechanism?

**Feedback Welcome:**
- Architecture decisions
- Technology choices
- Timeline estimates
- Risk assessment

---

**Document Version:** 1.0  
**Last Updated:** 2024-03-24  
**Status:** Draft for Review  
**Authors:** EvoBase Team  
**Reviewers:** TBD

