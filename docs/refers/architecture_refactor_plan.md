# Architecture Refactor Plan

## Executive Summary

This document outlines a comprehensive refactoring strategy to evolve PostgREST from its current monolithic architecture to a more modular, extensible, and production-hardened system while preserving its core "database-centric" philosophy.

**Goals:**
- Improve extensibility through plugin architecture
- Enhance security with additional layers of protection
- Increase observability for production operations
- Maintain backward compatibility
- Preserve performance characteristics

**Timeline:** 12-18 months (phased approach)

**Risk Level:** Medium (requires careful coordination with community)

---

## Target Architecture

### High-Level Vision

Transform PostgREST from a monolithic application to a **modular core with plugin ecosystem**, while maintaining the database-centric philosophy.

```mermaid
flowchart TD
    Client[HTTP Client] -->|Request| Gateway[API Gateway Layer]
    Gateway -->|Rate Limit, Auth| Core[PostgREST Core]
    Core -->|Plugin API| Plugins[Plugin System]
    Plugins -->|Custom Logic| Auth[Auth Plugins]
    Plugins -->|Custom Logic| Cache[Cache Plugins]
    Plugins -->|Custom Logic| Observability[Observability Plugins]
    Core -->|Query| Pool[Connection Pool]
    Pool -->|SQL| Primary[(Primary DB)]
    Pool -->|Read| Replica1[(Replica 1)]
    Pool -->|Read| Replica2[(Replica 2)]
    
    Cache -->|Store| Redis[(Redis Cache)]
    Observability -->|Metrics| Prometheus[Prometheus]
    Observability -->|Traces| Jaeger[Jaeger]
    Observability -->|Logs| Loki[Loki]
```

### Core Principles

1. **Backward Compatibility:** Existing APIs and configurations continue to work
2. **Opt-In Features:** New features optional; default behavior unchanged
3. **Performance:** No performance regression for existing use cases
4. **Simplicity:** Core remains simple; complexity in optional plugins
5. **Database-Centric:** Business logic still primarily in database

---

## Phase 1: Foundation (Months 1-3)

### 1.1 Modularize Core Components

**Objective:** Break monolithic codebase into well-defined modules with clear interfaces.

**Tasks:**



**1. Extract Schema Cache Interface**
```haskell
-- Define abstract interface
class SchemaCache m where
  loadSchema :: AppConfig -> m (Either Error Schema)
  invalidateCache :: m ()
  getTable :: QualifiedIdentifier -> m (Maybe Table)
  getFunction :: QualifiedIdentifier -> m (Maybe Routine)
  getRelationships :: QualifiedIdentifier -> m [Relationship]

-- Implementations
data InMemorySchemaCache = ...
data RedisSchemaCache = ...
data HybridSchemaCache = ...  -- In-memory with Redis backup
```

**2. Extract Query Builder as Library**
```haskell
-- New package: postgrest-query-builder
module PostgREST.QueryBuilder where
  buildSelect :: ReadPlan -> SQL.Snippet
  buildInsert :: MutatePlan -> SQL.Snippet
  buildUpdate :: MutatePlan -> SQL.Snippet
  buildDelete :: MutatePlan -> SQL.Snippet
```

**3. Define Plugin Interface**
```haskell
-- Plugin system
data Plugin = Plugin
  { pluginName :: Text
  , pluginVersion :: Version
  , pluginInit :: AppConfig -> IO PluginState
  , pluginMiddleware :: Maybe Middleware
  , pluginAuthProvider :: Maybe AuthProvider
  , pluginCacheProvider :: Maybe CacheProvider
  , pluginObserver :: Maybe ObservationHandler
  }

-- Plugin loading
loadPlugins :: FilePath -> IO [Plugin]
initializePlugins :: [Plugin] -> AppConfig -> IO PluginRegistry
```

**Deliverables:**
- [ ] Schema cache interface defined and implemented
- [ ] Query builder extracted to separate package
- [ ] Plugin system interface defined
- [ ] Documentation for plugin development
- [ ] Migration guide for existing deployments

**Success Metrics:**
- All existing tests pass
- No performance regression (< 5% overhead)
- Plugin interface documented with examples

**Risks:**
- Breaking changes to internal APIs (mitigated by semantic versioning)
- Performance overhead from abstraction layers (mitigated by benchmarking)

---

## Phase 2: Security Enhancements (Months 4-6)

### 2.1 Token Revocation System

**Objective:** Add ability to revoke JWT tokens before expiration.

**Implementation:**

```haskell
-- Token revocation interface
class TokenRevocation m where
  revokeToken :: ByteString -> UTCTime -> m ()  -- token, expiry
  isRevoked :: ByteString -> m Bool
  cleanupExpired :: m ()

-- Redis implementation
data RedisTokenRevocation = RedisTokenRevocation
  { redisConn :: Redis.Connection
  , keyPrefix :: Text
  }

instance TokenRevocation RedisTokenRevocation where
  revokeToken token expiry = do
    let key = keyPrefix <> "revoked:" <> decodeUtf8 token
    let ttl = diffUTCTime expiry (getCurrentTime)
    Redis.setex key ttl "1"
  
  isRevoked token = do
    let key = keyPrefix <> "revoked:" <> decodeUtf8 token
    result <- Redis.get key
    return $ isJust result
```

**Configuration:**
```yaml
jwt-revocation:
  enabled: true
  backend: redis
  redis-url: redis://localhost:6379
  key-prefix: "pgrst:revoked:"
```

**API Endpoints:**
```
POST /admin/revoke-token
Body: {"token": "eyJ..."}
Response: 204 No Content

GET /admin/revoked-tokens
Response: ["token1", "token2", ...]
```

**Deliverables:**
- [ ] Token revocation interface
- [ ] Redis implementation
- [ ] In-memory implementation (for testing)
- [ ] Admin endpoints for token management
- [ ] Documentation and examples

**Success Metrics:**
- Token revocation latency < 10ms
- No impact on non-revoked token validation
- Automatic cleanup of expired entries



### 2.2 Rate Limiting

**Objective:** Built-in rate limiting to prevent abuse and DoS attacks.

**Implementation:**

```haskell
-- Rate limiting interface
data RateLimitConfig = RateLimitConfig
  { rlWindowSize :: Int        -- seconds
  , rlMaxRequests :: Int        -- requests per window
  , rlScope :: RateLimitScope   -- IP, User, Endpoint, Global
  , rlBackend :: RateLimitBackend
  }

data RateLimitScope
  = ByIP
  | ByUser
  | ByEndpoint
  | ByIPAndEndpoint
  | Global

-- Middleware
rateLimitMiddleware :: RateLimitConfig -> Middleware
rateLimitMiddleware config app req respond = do
  key <- getRateLimitKey config req
  allowed <- checkRateLimit config key
  if allowed
    then app req respond
    else respond $ rateLimitExceeded config
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
```

**Response Headers:**
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1735689600
Retry-After: 45
```

**Deliverables:**
- [ ] Rate limiting middleware
- [ ] Redis-backed implementation
- [ ] In-memory implementation
- [ ] Configuration schema
- [ ] Response headers
- [ ] Documentation

**Success Metrics:**
- Rate limit check latency < 5ms
- Accurate counting across distributed instances
- Graceful degradation if Redis unavailable

### 2.3 Audit Logging

**Objective:** Comprehensive audit trail for compliance and security forensics.

**Implementation:**

```haskell
-- Audit log entry
data AuditLogEntry = AuditLogEntry
  { aleTimestamp :: UTCTime
  , aleUserId :: Maybe Text
  , aleUserRole :: Text
  , aleClientIP :: Text
  , aleMethod :: Text
  , alePath :: Text
  , aleAction :: AuditAction
  , aleResource :: QualifiedIdentifier
  , aleRowsAffected :: Maybe Int
  , aleSuccess :: Bool
  , aleErrorCode :: Maybe Text
  }

data AuditAction
  = AuditRead
  | AuditCreate
  | AuditUpdate
  | AuditDelete
  | AuditRPC

-- Audit logger interface
class AuditLogger m where
  logAudit :: AuditLogEntry -> m ()
  queryAuditLog :: AuditQuery -> m [AuditLogEntry]
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
  response_status INT
);

CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp DESC);
CREATE INDEX idx_audit_log_user_id ON audit_log(user_id);
CREATE INDEX idx_audit_log_resource ON audit_log(resource);
```

**Configuration:**
```yaml
audit-logging:
  enabled: true
  backend: database  # or: file, syslog, elasticsearch
  table: audit_log
  include-request-body: false  # PII concerns
  include-response-body: false
  sample-rate: 1.0  # 100% of requests
  exclude-paths:
    - /health
    - /metrics
```

**Deliverables:**
- [ ] Audit logging middleware
- [ ] Database backend implementation
- [ ] File backend implementation
- [ ] Query API for audit logs
- [ ] Retention policy configuration
- [ ] Documentation and compliance guides

**Success Metrics:**
- Audit log latency < 10ms (async)
- No data loss
- Queryable audit trail
- GDPR/HIPAA compliance support

---

## Phase 3: Performance & Scalability (Months 7-9)

### 3.1 Response Caching

**Objective:** Reduce database load and improve response times for cacheable data.

**Implementation:**

```haskell
-- Cache interface
class ResponseCache m where
  cacheGet :: CacheKey -> m (Maybe CachedResponse)
  cachePut :: CacheKey -> CachedResponse -> TTL -> m ()
  cacheInvalidate :: CacheKey -> m ()
  cacheInvalidatePattern :: Text -> m ()

data CacheKey = CacheKey
  { ckPath :: Text
  , ckQuery :: Text
  , ckRole :: Text
  , ckAccept :: Text
  }

data CachedResponse = CachedResponse
  { crStatus :: HTTP.Status
  , crHeaders :: [HTTP.Header]
  , crBody :: LBS.ByteString
  , crTimestamp :: UTCTime
  }

-- Cache middleware
cacheMiddleware :: ResponseCache m => CacheConfig -> Middleware
cacheMiddleware config app req respond = do
  let key = makeCacheKey req
  cached <- cacheGet key
  case cached of
    Just response | not (isStale response) ->
      respond $ cachedToWaiResponse response
    _ -> do
      app req $ \response -> do
        when (isCacheable response) $
          cachePut key (waiToCachedResponse response) (getTTL config req)
        respond response
```

**Configuration:**
```yaml
caching:
  enabled: true
  backend: redis
  redis-url: redis://localhost:6379
  default-ttl: 300  # seconds
  rules:
    - path: /users
      methods: [GET]
      ttl: 60
      vary: [Accept, Accept-Language]
    - path: /posts
      methods: [GET]
      ttl: 300
      invalidate-on:
        - POST /posts
        - PATCH /posts/*
        - DELETE /posts/*
```

**Cache Invalidation:**
```haskell
-- Automatic invalidation on mutations
invalidateOnMutation :: Middleware
invalidateOnMutation app req respond = do
  app req $ \response -> do
    when (isMutation req && isSuccess response) $ do
      let resource = extractResource req
      cacheInvalidatePattern ("*" <> resource <> "*")
    respond response
```

**Deliverables:**
- [ ] Cache middleware
- [ ] Redis implementation
- [ ] In-memory implementation (for testing)
- [ ] Cache invalidation logic
- [ ] HTTP cache headers (ETag, Last-Modified)
- [ ] Configuration schema
- [ ] Documentation

**Success Metrics:**
- Cache hit rate > 60% for cacheable endpoints
- Cache lookup latency < 5ms
- Reduced database load by 40%+



### 3.2 Read Replica Support

**Objective:** Distribute read traffic across multiple PostgreSQL replicas.

**Implementation:**

```haskell
-- Connection pool with replica support
data PoolConfig = PoolConfig
  { pcPrimary :: ConnectionString
  , pcReplicas :: [ConnectionString]
  , pcReadStrategy :: ReadStrategy
  , pcPoolSize :: Int
  }

data ReadStrategy
  = RoundRobin
  | Random
  | LeastConnections
  | PreferLocal  -- Prefer replica in same datacenter

-- Pool manager
data PoolManager = PoolManager
  { pmPrimaryPool :: SQL.Pool
  , pmReplicaPools :: [SQL.Pool]
  , pmStrategy :: ReadStrategy
  , pmHealthCheck :: IORef [Bool]  -- Health status of each replica
  }

-- Query routing
executeQuery :: PoolManager -> QueryType -> SQL.Session a -> IO (Either Error a)
executeQuery pm queryType session = do
  pool <- case queryType of
    ReadOnly -> selectReplica pm
    ReadWrite -> return (pmPrimaryPool pm)
  SQL.use pool session

selectReplica :: PoolManager -> IO SQL.Pool
selectReplica pm = do
  healthStatus <- readIORef (pmHealthCheck pm)
  let healthyReplicas = filter (fst) $ zip healthStatus (pmReplicaPools pm)
  case healthyReplicas of
    [] -> return (pmPrimaryPool pm)  -- Fallback to primary
    replicas -> applyStrategy (pmStrategy pm) (map snd replicas)
```

**Configuration:**
```yaml
database:
  primary: postgresql://user:pass@primary:5432/db
  replicas:
    - postgresql://user:pass@replica1:5432/db
    - postgresql://user:pass@replica2:5432/db
  read-strategy: round-robin
  health-check-interval: 10  # seconds
  replica-lag-threshold: 5000  # milliseconds
```

**Health Checking:**
```haskell
-- Periodic health check
healthCheckLoop :: PoolManager -> IO ()
healthCheckLoop pm = forever $ do
  statuses <- forM (pmReplicaPools pm) $ \pool -> do
    result <- timeout 5000000 $ SQL.use pool checkHealth
    return $ isJust result && fromJust result
  atomicWriteIORef (pmHealthCheck pm) statuses
  threadDelay 10000000  -- 10 seconds

checkHealth :: SQL.Session Bool
checkHealth = do
  -- Check if replica is up and not too far behind
  lag <- SQL.statement () replicationLagQuery
  return $ lag < 5000  -- milliseconds
```

**Deliverables:**
- [ ] Multi-pool connection manager
- [ ] Read replica routing logic
- [ ] Health checking system
- [ ] Replication lag monitoring
- [ ] Automatic failover to primary
- [ ] Configuration schema
- [ ] Documentation

**Success Metrics:**
- Read traffic distributed across replicas
- Automatic failover on replica failure
- Replication lag monitoring
- No impact on write performance

### 3.3 Query Cost Estimation

**Objective:** Prevent expensive queries from overloading the database.

**Implementation:**

```haskell
-- Query cost estimator
data QueryCostConfig = QueryCostConfig
  { qccMaxCost :: Double
  , qccEnabled :: Bool
  , qccExemptRoles :: [Text]
  }

estimateQueryCost :: SQL.Snippet -> SQL.Session Double
estimateQueryCost query = do
  let explainQuery = "EXPLAIN (FORMAT JSON) " <> query
  result <- SQL.statement () explainQuery
  return $ extractCost result

-- Middleware
queryCostMiddleware :: QueryCostConfig -> Middleware
queryCostMiddleware config next plan = do
  unless (userRole plan `elem` qccExemptRoles config) $ do
    cost <- estimateQueryCost (planToQuery plan)
    when (cost > qccMaxCost config) $
      throwError $ QueryTooExpensive cost (qccMaxCost config)
  next plan
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
    - path: /expensive_view
      max-cost: 50000
```

**Response:**
```json
{
  "code": "PGRST123",
  "message": "Query cost too high",
  "details": "Estimated cost: 15000, maximum allowed: 10000",
  "hint": "Try adding filters or reducing the result set"
}
```

**Deliverables:**
- [ ] Query cost estimation
- [ ] Cost-based query rejection
- [ ] Per-endpoint cost limits
- [ ] Role-based exemptions
- [ ] Documentation

**Success Metrics:**
- Expensive queries blocked before execution
- No false positives (legitimate queries blocked)
- Cost estimation overhead < 10ms

---

## Phase 4: Observability (Months 10-12)

### 4.1 Distributed Tracing

**Objective:** End-to-end request tracing for debugging and performance analysis.

**Implementation:**

```haskell
-- OpenTelemetry integration
import qualified OpenTelemetry.Trace as OTel

-- Tracing middleware
tracingMiddleware :: OTel.Tracer -> Middleware
tracingMiddleware tracer app req respond = do
  OTel.inSpan tracer "http.request" $ \span -> do
    OTel.addAttribute span "http.method" (requestMethod req)
    OTel.addAttribute span "http.path" (rawPathInfo req)
    OTel.addAttribute span "http.user_agent" (lookup "User-Agent" $ requestHeaders req)
    
    app req $ \response -> do
      OTel.addAttribute span "http.status_code" (statusCode $ responseStatus response)
      respond response

-- Database query tracing
traceQuery :: OTel.Span -> SQL.Session a -> SQL.Session a
traceQuery parentSpan session = do
  OTel.inSpan' parentSpan "db.query" $ \span -> do
    OTel.addAttribute span "db.system" "postgresql"
    OTel.addAttribute span "db.operation" "SELECT"  -- or INSERT, UPDATE, DELETE
    startTime <- getCurrentTime
    result <- session
    endTime <- getCurrentTime
    OTel.addAttribute span "db.duration_ms" (diffUTCTime endTime startTime * 1000)
    return result
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

**Trace Context Propagation:**
```haskell
-- Extract trace context from headers
extractTraceContext :: Request -> Maybe OTel.SpanContext
extractTraceContext req = do
  traceParent <- lookup "traceparent" (requestHeaders req)
  parseTraceParent traceParent

-- Inject trace context into database
injectTraceContext :: OTel.SpanContext -> SQL.Session ()
injectTraceContext ctx = do
  let traceId = OTel.traceIdToHex (OTel.spanContextTraceId ctx)
  let spanId = OTel.spanIdToHex (OTel.spanContextSpanId ctx)
  SQL.statement () $ "SET LOCAL application_name = 'postgrest:" <> traceId <> ":" <> spanId <> "'"
```

**Deliverables:**
- [ ] OpenTelemetry integration
- [ ] Jaeger exporter
- [ ] Zipkin exporter
- [ ] Trace context propagation
- [ ] Database query tracing
- [ ] Configuration schema
- [ ] Documentation

**Success Metrics:**
- End-to-end request visibility
- Database query attribution
- Performance bottleneck identification
- Trace overhead < 5ms per request



### 4.2 Enhanced Metrics

**Objective:** Comprehensive metrics for monitoring and alerting.

**Implementation:**

```haskell
-- Metrics registry
data MetricsRegistry = MetricsRegistry
  { mrRequestDuration :: Histogram
  , mrRequestCount :: Counter
  , mrActiveRequests :: Gauge
  , mrDatabaseQueryDuration :: Histogram
  , mrDatabaseConnectionsActive :: Gauge
  , mrDatabaseConnectionsIdle :: Gauge
  , mrCacheHitRate :: Counter
  , mrCacheMissRate :: Counter
  , mrRateLimitExceeded :: Counter
  , mrAuthFailures :: Counter
  , mrQueryCostRejected :: Counter
  }

-- Metric labels
data RequestLabels = RequestLabels
  { rlMethod :: Text
  , rlPath :: Text
  , rlStatus :: Int
  , rlRole :: Text
  }

-- Metrics middleware
metricsMiddleware :: MetricsRegistry -> Middleware
metricsMiddleware registry app req respond = do
  startTime <- getCurrentTime
  incrementGauge (mrActiveRequests registry)
  
  app req $ \response -> do
    endTime <- getCurrentTime
    let duration = diffUTCTime endTime startTime
    let labels = RequestLabels
          { rlMethod = decodeUtf8 $ requestMethod req
          , rlPath = decodeUtf8 $ rawPathInfo req
          , rlStatus = statusCode $ responseStatus response
          , rlRole = extractRole req
          }
    
    observeHistogram (mrRequestDuration registry) labels duration
    incrementCounter (mrRequestCount registry) labels
    decrementGauge (mrActiveRequests registry)
    
    respond response
```

**Prometheus Metrics:**
```
# Request metrics
postgrest_http_requests_total{method="GET",path="/users",status="200",role="authenticated"} 1234
postgrest_http_request_duration_seconds{method="GET",path="/users",status="200",role="authenticated"} 0.045
postgrest_http_requests_active 42

# Database metrics
postgrest_db_query_duration_seconds{operation="SELECT"} 0.023
postgrest_db_connections_active 8
postgrest_db_connections_idle 2
postgrest_db_pool_wait_duration_seconds 0.001

# Cache metrics
postgrest_cache_hits_total{path="/users"} 567
postgrest_cache_misses_total{path="/users"} 123
postgrest_cache_hit_rate 0.82

# Security metrics
postgrest_rate_limit_exceeded_total{scope="ip"} 45
postgrest_auth_failures_total{reason="invalid_token"} 12
postgrest_query_cost_rejected_total 3

# Schema cache metrics
postgrest_schema_cache_reload_total 5
postgrest_schema_cache_reload_duration_seconds 1.234
postgrest_schema_cache_size_bytes 1048576
```

**Grafana Dashboard:**
- Request rate and latency
- Error rate by status code
- Database query performance
- Connection pool utilization
- Cache hit rate
- Rate limit violations
- Authentication failures

**Deliverables:**
- [ ] Enhanced metrics collection
- [ ] Prometheus exporter improvements
- [ ] Grafana dashboard templates
- [ ] Alerting rules
- [ ] Documentation

**Success Metrics:**
- Comprehensive observability
- Low-overhead metrics collection
- Actionable alerts

### 4.3 Structured Logging

**Objective:** Machine-readable logs for analysis and debugging.

**Implementation:**

```haskell
-- Structured log entry
data LogEntry = LogEntry
  { leTimestamp :: UTCTime
  , leLevel :: LogLevel
  , leMessage :: Text
  , leContext :: LogContext
  , leError :: Maybe Error
  , leTrace :: Maybe TraceContext
  }

data LogContext = LogContext
  { lcRequestId :: Maybe Text
  , lcUserId :: Maybe Text
  , lcUserRole :: Maybe Text
  , lcMethod :: Maybe Text
  , lcPath :: Maybe Text
  , lcDuration :: Maybe Double
  , lcCustomFields :: HashMap Text Value
  }

-- JSON output
instance ToJSON LogEntry where
  toJSON le = object
    [ "timestamp" .= leTimestamp le
    , "level" .= leLevel le
    , "message" .= leMessage le
    , "context" .= leContext le
    , "error" .= leError le
    , "trace" .= leTrace le
    ]
```

**Log Levels:**
- DEBUG: Detailed debugging information
- INFO: General informational messages
- WARN: Warning messages (non-critical issues)
- ERROR: Error messages (request failures)
- FATAL: Fatal errors (application crashes)

**Example Log Output:**
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
    "path": "/users",
    "status": 200,
    "duration_ms": 45.2,
    "db_query_count": 1,
    "cache_hit": true
  },
  "trace": {
    "trace_id": "0af7651916cd43dd8448eb211c80319c",
    "span_id": "b7ad6b7169203331"
  }
}
```

**Deliverables:**
- [ ] Structured logging implementation
- [ ] JSON log formatter
- [ ] Log correlation with traces
- [ ] Log sampling for high-volume endpoints
- [ ] Documentation

**Success Metrics:**
- Machine-readable logs
- Log correlation with traces
- Efficient log parsing and analysis

---

## Phase 5: Advanced Features (Months 13-18)

### 5.1 GraphQL Support

**Objective:** Optional GraphQL endpoint alongside REST API.

**Implementation:**

```haskell
-- GraphQL schema generation
generateGraphQLSchema :: SchemaCache -> GraphQL.Schema
generateGraphQLSchema cache = GraphQL.Schema
  { schemaQuery = generateQueryType cache
  , schemaMutation = generateMutationType cache
  , schemaSubscription = Nothing  -- Future work
  }

-- Query type generation
generateQueryType :: SchemaCache -> GraphQL.ObjectType
generateQueryType cache = GraphQL.ObjectType "Query" $ mconcat
  [ tableToGraphQLField table | table <- schemaTables cache ]
  ++ [ functionToGraphQLField func | func <- schemaFunctions cache ]

-- Example GraphQL query
{
  users(where: {age: {_gt: 18}}, limit: 10) {
    id
    email
    posts {
      title
      created_at
    }
  }
}
```

**Configuration:**
```yaml
graphql:
  enabled: true
  endpoint: /graphql
  playground: true  # GraphQL Playground UI
  introspection: true
  max-depth: 10  # Prevent deeply nested queries
```

**Deliverables:**
- [ ] GraphQL schema generation
- [ ] Query resolver implementation
- [ ] Mutation resolver implementation
- [ ] GraphQL Playground integration
- [ ] Documentation

**Success Metrics:**
- Feature parity with REST API
- Performance comparable to REST
- GraphQL best practices followed



### 5.2 WebSocket Support

**Objective:** Real-time updates via WebSocket subscriptions.

**Implementation:**

```haskell
-- WebSocket server
data WebSocketServer = WebSocketServer
  { wssConnections :: TVar (HashMap ConnectionId Connection)
  , wssSubscriptions :: TVar (HashMap SubscriptionId Subscription)
  , wssNotifier :: NotificationHandler
  }

data Subscription = Subscription
  { subId :: SubscriptionId
  , subConnection :: ConnectionId
  , subResource :: QualifiedIdentifier
  , subFilters :: [Filter]
  , subRole :: Text
  }

-- Subscription protocol
data ClientMessage
  = Subscribe SubscriptionId QualifiedIdentifier [Filter]
  | Unsubscribe SubscriptionId
  | Ping

data ServerMessage
  = Subscribed SubscriptionId
  | Unsubscribed SubscriptionId
  | DataChange SubscriptionId ChangeType Value
  | Error SubscriptionId Text
  | Pong

-- Change notification
handleDatabaseNotification :: WebSocketServer -> Notification -> IO ()
handleDatabaseNotification wss notif = do
  let resource = notifResource notif
  let changeType = notifChangeType notif
  let rowData = notifData notif
  
  subs <- readTVarIO (wssSubscriptions wss)
  let matchingSubs = filter (matchesSubscription resource) (HashMap.elems subs)
  
  forM_ matchingSubs $ \sub -> do
    when (matchesFilters (subFilters sub) rowData) $ do
      sendToConnection (subConnection sub) $
        DataChange (subId sub) changeType rowData
```

**Database Trigger:**
```sql
-- Trigger for change notifications
CREATE OR REPLACE FUNCTION notify_change()
RETURNS TRIGGER AS $$
BEGIN
  PERFORM pg_notify(
    'postgrest_changes',
    json_build_object(
      'table', TG_TABLE_NAME,
      'schema', TG_TABLE_SCHEMA,
      'operation', TG_OP,
      'data', CASE
        WHEN TG_OP = 'DELETE' THEN row_to_json(OLD)
        ELSE row_to_json(NEW)
      END
    )::text
  );
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply to tables
CREATE TRIGGER users_notify_change
  AFTER INSERT OR UPDATE OR DELETE ON users
  FOR EACH ROW EXECUTE FUNCTION notify_change();
```

**Client Usage:**
```javascript
const ws = new WebSocket('ws://localhost:3000/ws');

// Subscribe to changes
ws.send(JSON.stringify({
  type: 'subscribe',
  id: 'sub1',
  resource: 'users',
  filters: [{column: 'age', operator: 'gt', value: 18}]
}));

// Receive changes
ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  if (msg.type === 'data_change') {
    console.log('Change:', msg.changeType, msg.data);
  }
};
```

**Configuration:**
```yaml
websocket:
  enabled: true
  endpoint: /ws
  max-connections: 10000
  ping-interval: 30  # seconds
  max-subscriptions-per-connection: 100
```

**Deliverables:**
- [ ] WebSocket server implementation
- [ ] Subscription management
- [ ] Database change notifications
- [ ] Client library (JavaScript)
- [ ] Documentation

**Success Metrics:**
- Support 10,000+ concurrent connections
- Sub-second change notification latency
- Automatic reconnection on disconnect

### 5.3 Multi-Tenancy Support

**Objective:** Built-in support for multi-tenant applications.

**Implementation:**

```haskell
-- Tenant identification
data TenantConfig = TenantConfig
  { tcStrategy :: TenantStrategy
  , tcHeader :: Maybe Text
  , tcSubdomain :: Bool
  , tcPathPrefix :: Bool
  }

data TenantStrategy
  = TenantByHeader Text        -- X-Tenant-ID header
  | TenantBySubdomain          -- tenant.example.com
  | TenantByPathPrefix         -- /tenant/users
  | TenantByJWTClaim Text      -- JWT claim
  | TenantBySchema             -- PostgreSQL schema per tenant

-- Tenant middleware
tenantMiddleware :: TenantConfig -> Middleware
tenantMiddleware config app req respond = do
  tenantId <- extractTenantId config req
  case tenantId of
    Nothing -> respond $ missingTenantError
    Just tid -> do
      -- Set tenant context
      let req' = setTenantContext tid req
      app req' respond

-- Schema-based multi-tenancy
setTenantSchema :: Text -> SQL.Session ()
setTenantSchema tenantId = do
  let schema = "tenant_" <> tenantId
  SQL.statement () $ "SET search_path TO " <> schema <> ", public"
```

**Database Schema:**
```sql
-- Schema per tenant
CREATE SCHEMA tenant_abc123;
CREATE TABLE tenant_abc123.users (...);
CREATE TABLE tenant_abc123.posts (...);

CREATE SCHEMA tenant_def456;
CREATE TABLE tenant_def456.users (...);
CREATE TABLE tenant_def456.posts (...);

-- Or: Shared schema with tenant_id column
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  email TEXT NOT NULL,
  UNIQUE(tenant_id, email)
);

CREATE INDEX idx_users_tenant_id ON users(tenant_id);

-- RLS for tenant isolation
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON users
  USING (tenant_id = current_setting('app.tenant_id'));
```

**Configuration:**
```yaml
multi-tenancy:
  enabled: true
  strategy: schema  # or: column, header, subdomain
  tenant-header: X-Tenant-ID
  default-tenant: public
  tenant-claim: tenant_id  # JWT claim
```

**Deliverables:**
- [ ] Tenant identification strategies
- [ ] Schema-based isolation
- [ ] Column-based isolation with RLS
- [ ] Tenant context propagation
- [ ] Documentation

**Success Metrics:**
- Complete tenant isolation
- No cross-tenant data leakage
- Minimal performance overhead

---

## Risk Mitigation Strategy

### Technical Risks

**Risk 1: Performance Regression**
- **Mitigation:** Comprehensive benchmarking before/after each phase
- **Mitigation:** Performance budgets (< 5% overhead per feature)
- **Mitigation:** Feature flags to disable problematic features
- **Rollback Plan:** Revert to previous version if performance degrades

**Risk 2: Breaking Changes**
- **Mitigation:** Semantic versioning (major version for breaking changes)
- **Mitigation:** Deprecation warnings before removal
- **Mitigation:** Compatibility layer for deprecated features
- **Rollback Plan:** Maintain LTS version with security fixes only

**Risk 3: Increased Complexity**
- **Mitigation:** Modular architecture (complexity in optional plugins)
- **Mitigation:** Comprehensive documentation
- **Mitigation:** Default configuration works out-of-box
- **Rollback Plan:** Simplify or remove underutilized features



### Operational Risks

**Risk 4: Migration Complexity**
- **Mitigation:** Automated migration tools
- **Mitigation:** Detailed migration guides
- **Mitigation:** Backward compatibility for at least one major version
- **Rollback Plan:** Support running old and new versions side-by-side

**Risk 5: Community Resistance**
- **Mitigation:** Early RFC process for major changes
- **Mitigation:** Beta releases for community testing
- **Mitigation:** Clear communication of benefits
- **Rollback Plan:** Maintain conservative "stable" branch

**Risk 6: Resource Constraints**
- **Mitigation:** Phased approach (can pause between phases)
- **Mitigation:** Community contributions encouraged
- **Mitigation:** Prioritize high-impact features
- **Rollback Plan:** Extend timeline or reduce scope

### Security Risks

**Risk 7: New Attack Vectors**
- **Mitigation:** Security review for each new feature
- **Mitigation:** Penetration testing before release
- **Mitigation:** Bug bounty program
- **Rollback Plan:** Disable vulnerable features via configuration

**Risk 8: Dependency Vulnerabilities**
- **Mitigation:** Automated dependency scanning
- **Mitigation:** Regular dependency updates
- **Mitigation:** Minimal dependencies
- **Rollback Plan:** Pin to known-good versions

---

## Success Criteria

### Phase 1 Success Criteria
- [ ] All existing tests pass
- [ ] No performance regression (< 5% overhead)
- [ ] Plugin system documented with examples
- [ ] At least 2 community plugins created

### Phase 2 Success Criteria
- [ ] Token revocation working with < 10ms latency
- [ ] Rate limiting prevents DoS attacks
- [ ] Audit logging meets compliance requirements
- [ ] Security audit completed

### Phase 3 Success Criteria
- [ ] Cache hit rate > 60% for cacheable endpoints
- [ ] Read replicas reduce primary load by 40%+
- [ ] Query cost estimation prevents expensive queries
- [ ] Performance improved by 30%+ for cached endpoints

### Phase 4 Success Criteria
- [ ] End-to-end request tracing working
- [ ] Comprehensive metrics dashboard
- [ ] Structured logging enables efficient analysis
- [ ] Mean time to resolution (MTTR) reduced by 50%

### Phase 5 Success Criteria
- [ ] GraphQL feature parity with REST
- [ ] WebSocket supports 10,000+ concurrent connections
- [ ] Multi-tenancy prevents cross-tenant data leakage
- [ ] Advanced features adopted by 20%+ of users

---

## Rollout Strategy

### Beta Testing
1. **Alpha Release:** Internal testing (1 month)
2. **Beta Release:** Community testing (2 months)
3. **Release Candidate:** Production testing (1 month)
4. **General Availability:** Stable release

### Feature Flags
```yaml
features:
  plugin-system: true
  token-revocation: false  # Disabled by default
  rate-limiting: false
  response-caching: false
  read-replicas: false
  query-cost-estimation: false
  distributed-tracing: false
  graphql: false
  websocket: false
  multi-tenancy: false
```

### Gradual Rollout
1. Enable feature for 1% of traffic
2. Monitor metrics and errors
3. Increase to 10%, 50%, 100%
4. Rollback if issues detected

### Communication Plan
- **RFC Process:** Propose major changes for community feedback
- **Blog Posts:** Announce new features and benefits
- **Migration Guides:** Step-by-step upgrade instructions
- **Webinars:** Demonstrate new features
- **Office Hours:** Answer community questions

---

## Maintenance & Support

### Long-Term Support (LTS)
- **LTS Version:** Maintained for 2 years
- **Security Fixes:** Backported to LTS
- **Bug Fixes:** Critical bugs only
- **No New Features:** LTS remains stable

### Version Support Matrix
| Version | Release Date | End of Support | Status |
|---------|--------------|----------------|--------|
| 12.x    | 2024-01     | 2026-01       | LTS    |
| 13.x    | 2024-07     | 2025-07       | Stable |
| 14.x    | 2025-01     | 2026-01       | Current|
| 15.x    | 2025-07     | TBD           | Beta   |

### Deprecation Policy
1. **Announce:** Deprecation warning in release notes
2. **Warn:** Runtime warnings for deprecated features
3. **Remove:** Removal in next major version (minimum 6 months)

---

## Conclusion

This refactor plan transforms PostgREST from a monolithic application to a modular, extensible, production-hardened system while preserving its core strengths:

**Preserved:**
- Database-centric philosophy
- Zero-code API generation
- Stateless architecture
- High performance
- Simplicity for basic use cases

**Enhanced:**
- Extensibility via plugins
- Security with additional layers
- Observability for production operations
- Scalability with caching and replicas
- Advanced features (GraphQL, WebSocket, multi-tenancy)

**Timeline:** 12-18 months, phased approach

**Risk Level:** Medium, mitigated by careful planning and testing

**Expected Outcome:** PostgREST becomes the de facto standard for PostgreSQL REST APIs, suitable for both simple prototypes and large-scale production deployments.

