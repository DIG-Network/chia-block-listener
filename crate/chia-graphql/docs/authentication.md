# Authentication & Security

## Overview

The Chia GraphQL API is designed as a **read-only** interface with strong security considerations. This document covers authentication, authorization, and security best practices.

## Authentication Methods

### 1. No Authentication (Local/Development)

For local development or trusted environments:

```rust
// Simple setup without authentication
let app = Router::new()
    .route("/graphql", post(GraphQL::new(schema)));
```

⚠️ **Warning**: Only use this in completely trusted environments.

### 2. API Key Authentication

Implement API key authentication using middleware:

```rust
use axum::{
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
};

async fn auth_middleware<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("x-api-key")
        .and_then(|value| value.to_str().ok());

    match auth_header {
        Some(api_key) if is_valid_api_key(api_key) => {
            Ok(next.run(req).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

// Apply to router
let app = Router::new()
    .route("/graphql", post(GraphQL::new(schema)))
    .layer(middleware::from_fn(auth_middleware));
```

### 3. JWT Authentication

For more sophisticated authentication:

```rust
use jsonwebtoken::{decode, DecodingKey, Validation};

async fn jwt_middleware<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|value| value.to_str().ok());

    if let Some(auth_value) = auth_header {
        if auth_value.starts_with("Bearer ") {
            let token = &auth_value[7..];
            
            match decode::<Claims>(
                token,
                &DecodingKey::from_secret("secret".as_ref()),
                &Validation::default(),
            ) {
                Ok(_) => return Ok(next.run(req).await),
                Err(_) => return Err(StatusCode::UNAUTHORIZED),
            }
        }
    }
    
    Err(StatusCode::UNAUTHORIZED)
}
```

## Authorization Strategies

### 1. Query Depth Limiting

Prevent deeply nested queries that could cause performance issues:

```rust
use async_graphql::*;

let schema = Schema::build(query_root, EmptyMutation, EmptySubscription)
    .limit_depth(5)  // Maximum query depth
    .finish();
```

### 2. Query Complexity Analysis

Limit query complexity based on field weights:

```rust
let schema = Schema::build(query_root, EmptyMutation, EmptySubscription)
    .limit_complexity(1000)  // Maximum complexity score
    .finish();
```

### 3. Rate Limiting

Implement rate limiting to prevent abuse:

```rust
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

let governor_conf = Box::new(
    GovernorConfigBuilder::default()
        .per_second(10)  // 10 requests per second
        .burst_size(100)
        .finish()
        .unwrap(),
);

let app = Router::new()
    .route("/graphql", post(GraphQL::new(schema)))
    .layer(GovernorLayer {
        config: Box::leak(governor_conf),
    });
```

### 4. Field-Level Authorization

Implement custom guards for sensitive fields:

```rust
struct AuthGuard {
    required_role: String,
}

#[async_trait::async_trait]
impl Guard for AuthGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let user = ctx.data::<User>()?;
        if user.has_role(&self.required_role) {
            Ok(())
        } else {
            Err("Unauthorized".into())
        }
    }
}

// Usage in resolver
#[Object]
impl SensitiveQueries {
    #[graphql(guard = "AuthGuard { required_role: \"admin\".to_string() }")]
    async fn sensitive_data(&self) -> Result<String> {
        Ok("Sensitive information".to_string())
    }
}
```

## Security Best Practices

### 1. Input Validation

All inputs are validated:

- **Hex strings**: Validated for proper format
- **Pagination**: Limited to reasonable page sizes
- **Time ranges**: Capped to prevent excessive queries

Example validation:

```rust
if page_size > 100 {
    return Err(GraphQLError::InvalidInput(
        "Page size cannot exceed 100".to_string()
    ));
}
```

### 2. SQL Injection Prevention

All queries use parameterized statements:

```rust
// Safe: Uses parameter binding
let query = "SELECT * FROM coins WHERE puzzle_hash = $1";
sqlx::query(query)
    .bind(&puzzle_hash)
    .fetch_all(&pool)
    .await?;

// Never do this:
// let query = format!("SELECT * FROM coins WHERE puzzle_hash = '{}'", puzzle_hash);
```

### 3. CORS Configuration

Configure CORS appropriately:

```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin("https://your-frontend.com".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([CONTENT_TYPE]);

let app = Router::new()
    .route("/graphql", post(GraphQL::new(schema)))
    .layer(cors);
```

### 4. HTTPS Only

Always use HTTPS in production:

```rust
// Redirect HTTP to HTTPS
async fn redirect_http_to_https(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let uri = req.uri();
    let https_uri = format!("https://{}{}", 
        req.headers().get(HOST).and_then(|h| h.to_str().ok()).unwrap_or("localhost"),
        uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/")
    );
    
    Ok(Response::builder()
        .status(StatusCode::MOVED_PERMANENTLY)
        .header(LOCATION, https_uri)
        .body(Body::empty())
        .unwrap())
}
```

## Data Privacy

### 1. No Personally Identifiable Information (PII)

The API only exposes:
- Public blockchain data
- Aggregated statistics
- Pseudonymous addresses

### 2. Address Privacy

- Addresses are pseudonymous (puzzle hashes)
- No linking to real-world identities
- No transaction memo fields exposed

### 3. Logging Best Practices

```rust
// Log queries but not sensitive data
tracing::info!(
    "GraphQL query executed: depth={}, complexity={}", 
    query_depth, 
    query_complexity
);

// Never log:
// - Full addresses
// - API keys
// - User tokens
```

## Deployment Security

### 1. Database Connection Security

Use encrypted connections:

```toml
# PostgreSQL with SSL
DATABASE_URL="postgresql://user:pass@host/db?sslmode=require"

# Use connection pooling
DATABASE_MAX_CONNECTIONS=10
DATABASE_MIN_CONNECTIONS=2
```

### 2. Environment Variables

Never hardcode secrets:

```rust
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");

let api_key_secret = std::env::var("API_KEY_SECRET")
    .expect("API_KEY_SECRET must be set");
```

### 3. Container Security

For Docker deployments:

```dockerfile
# Run as non-root user
RUN adduser -D appuser
USER appuser

# Minimal base image
FROM alpine:latest

# Security scanning
RUN apk add --no-cache ca-certificates
```

## Monitoring & Auditing

### 1. Query Logging

Log all queries for audit:

```rust
#[derive(Default)]
struct QueryLogger;

#[async_trait::async_trait]
impl Extension for QueryLogger {
    async fn request(&self, ctx: &ExtensionContext<'_>, next: NextRequest<'_>) -> Response {
        let start = Instant::now();
        let res = next.run(ctx).await;
        let duration = start.elapsed();
        
        tracing::info!(
            "Query executed in {:?}: {}", 
            duration,
            ctx.query_string
        );
        
        res
    }
}
```

### 2. Metrics Collection

Track important metrics:

```rust
// Using prometheus
let query_counter = prometheus::IntCounter::new(
    "graphql_queries_total", 
    "Total number of GraphQL queries"
)?;

let query_duration = prometheus::Histogram::new(
    "graphql_query_duration_seconds",
    "GraphQL query duration"
)?;
```

## Security Headers

Add security headers to all responses:

```rust
use tower_http::set_header::SetResponseHeaderLayer;

let app = Router::new()
    .route("/graphql", post(GraphQL::new(schema)))
    .layer(SetResponseHeaderLayer::if_not_present(
        X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    ))
    .layer(SetResponseHeaderLayer::if_not_present(
        X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    ))
    .layer(SetResponseHeaderLayer::if_not_present(
        X_XSS_PROTECTION,
        HeaderValue::from_static("1; mode=block"),
    ));
```

## Incident Response

### 1. Rate Limit Exceeded

```json
{
  "errors": [{
    "message": "Rate limit exceeded",
    "extensions": {
      "code": "RATE_LIMITED",
      "retryAfter": 60
    }
  }]
}
```

### 2. Unauthorized Access

```json
{
  "errors": [{
    "message": "Unauthorized",
    "extensions": {
      "code": "UNAUTHORIZED"
    }
  }]
}
```

## Security Checklist

- [ ] Use HTTPS in production
- [ ] Implement authentication (API key or JWT)
- [ ] Set query depth limits
- [ ] Configure rate limiting
- [ ] Validate all inputs
- [ ] Use parameterized queries
- [ ] Configure CORS properly
- [ ] Add security headers
- [ ] Log queries for audit
- [ ] Monitor for anomalies
- [ ] Regular security updates
- [ ] Database connection encryption
- [ ] Environment variable management
- [ ] Container security scanning

## Next Steps

- Review [Getting Started](getting-started.md) for setup
- Check [Performance Guide](performance.md) for optimization
- See [Query Documentation](queries/core.md) for API usage 