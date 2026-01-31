// Middleware for authentication, rate limiting, etc.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

/// Authentication middleware (placeholder for Phase 4)
pub async fn auth_middleware(request: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    // TODO: Implement API key authentication in Phase 4
    // For now, allow all requests
    Ok(next.run(request).await)
}
