// HTTP request handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::AgentServer;

/// Create the main application router
pub fn create_router(server: Arc<AgentServer>) -> Router {
    Router::new()
        .route("/v1/messages", post(handle_message))
        .route("/v1/session/:id", get(get_session).delete(delete_session))
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_endpoint))
        .with_state(server)
}

/// Request body for /v1/messages endpoint (Claude-compatible)
#[derive(Debug, Deserialize)]
pub struct MessageRequest {
    /// Model to use (e.g., "claude-sonnet-4-5-20250929")
    pub model: String,
    /// Messages in conversation
    pub messages: Vec<Message>,
    /// Maximum tokens to generate
    #[serde(default)]
    pub max_tokens: Option<u32>,
    /// System prompt
    #[serde(default)]
    pub system: Option<String>,
    /// Session ID for conversation continuity
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Message in Claude format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Response body for /v1/messages endpoint (Claude-compatible)
#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub stop_reason: String,
    pub session_id: String,
}

/// Content block in response
#[derive(Debug, Serialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: String,
}

/// Handle POST /v1/messages - Main chat endpoint
async fn handle_message(
    State(server): State<Arc<AgentServer>>,
    Json(request): Json<MessageRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    use crate::claude::MessageRequest as ClaudeRequest;
    use crate::metrics::{RequestMetric, ResponseComparison};
    use crate::router::RouteDecision;
    use std::time::Instant;

    let start_time = Instant::now();

    // Get or create session
    let mut session = server
        .session_manager()
        .get_or_create(request.session_id.as_deref())?;

    // Extract user message (last message should be user role)
    let user_message = request
        .messages
        .last()
        .ok_or_else(|| anyhow::anyhow!("No messages in request"))?
        .content
        .clone();

    // Add to conversation history
    session.conversation.add_user_message(user_message.clone());

    // Process query through router
    let router = server.router().read().await;
    let decision = router.route(&user_message);

    let (response_text, routing_decision) = match decision {
        RouteDecision::Forward { reason } => {
            let reason_str = format!("{:?}", reason);
            tracing::info!(
                session_id = %session.id,
                reason = %reason_str,
                "Forwarding to Claude API"
            );

            // Build Claude API request with full conversation context
            let claude_request = ClaudeRequest::with_context(session.conversation.get_messages());

            // Forward to Claude
            let response = server
                .claude_client()
                .send_message(&claude_request)
                .await?;

            // Extract text from response
            let text = response.text();

            (text, "forward".to_string())
        }
        RouteDecision::Local { .. } => {
            tracing::info!(session_id = %session.id, "Handling locally");

            // TODO: Use actual local generator when implemented
            // For now, fall back to Claude
            let claude_request = ClaudeRequest::with_context(session.conversation.get_messages());
            let response = server
                .claude_client()
                .send_message(&claude_request)
                .await?;

            let text = response.text();

            (text, "local_fallback".to_string())
        }
    };

    let elapsed_ms = start_time.elapsed().as_millis() as u64;

    // Log metrics
    let query_hash = crate::metrics::MetricsLogger::hash_query(&user_message);
    let metric = RequestMetric::new(
        query_hash,
        routing_decision,
        None, // pattern_id
        None, // confidence
        None, // forward_reason
        elapsed_ms,
        ResponseComparison {
            local_response: None,
            claude_response: response_text.clone(),
            quality_score: 1.0,
            similarity_score: None,
            divergence: None,
        },
        None, // router_confidence
        None, // validator_confidence
    );
    server.metrics_logger().log(&metric)?;

    // Add response to conversation history
    session.conversation.add_assistant_message(response_text.clone());

    // Update session
    session.touch();
    server
        .session_manager()
        .update(&session.id, session.clone())?;

    // Build Claude-compatible response
    let response = MessageResponse {
        id: format!("msg_{}", uuid::Uuid::new_v4()),
        response_type: "message".to_string(),
        role: "assistant".to_string(),
        content: vec![ContentBlock {
            block_type: "text".to_string(),
            text: response_text,
        }],
        model: request.model,
        stop_reason: "end_turn".to_string(),
        session_id: session.id,
    };

    Ok(Json(response))
}

/// Handle GET /v1/session/:id - Retrieve session state
async fn get_session(
    State(server): State<Arc<AgentServer>>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionInfo>, AppError> {
    let session = server
        .session_manager()
        .get_or_create(Some(&session_id))?;

    let info = SessionInfo {
        id: session.id,
        created_at: session.created_at.to_rfc3339(),
        last_activity: session.last_activity.to_rfc3339(),
        message_count: session.conversation.message_count(),
    };

    Ok(Json(info))
}

/// Session information
#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub created_at: String,
    pub last_activity: String,
    pub message_count: usize,
}

/// Handle DELETE /v1/session/:id - Delete session
async fn delete_session(
    State(server): State<Arc<AgentServer>>,
    Path(session_id): Path<String>,
) -> Result<StatusCode, AppError> {
    if server.session_manager().delete(&session_id) {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError(anyhow::anyhow!("Session not found")))
    }
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub uptime_seconds: u64,
    pub active_sessions: usize,
}

/// Handle GET /health - Health check endpoint
pub async fn health_check(
    State(server): State<Arc<AgentServer>>,
) -> Result<Json<HealthStatus>, AppError> {
    // TODO: Track actual uptime
    let status = HealthStatus {
        status: "healthy".to_string(),
        uptime_seconds: 0, // Placeholder
        active_sessions: server.session_manager().active_count(),
    };

    Ok(Json(status))
}

/// Handle GET /metrics - Prometheus metrics endpoint
pub async fn metrics_endpoint(
    State(_server): State<Arc<AgentServer>>,
) -> Result<Response, AppError> {
    // TODO: Implement Prometheus metrics
    let metrics = "# HELP shammah_queries_total Total number of queries\n\
                   # TYPE shammah_queries_total counter\n\
                   shammah_queries_total 0\n";

    Ok((StatusCode::OK, metrics).into_response())
}

/// Application error wrapper for proper HTTP error responses
pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self.0, "Request failed");

        let error_message = self.0.to_string();
        let body = serde_json::json!({
            "error": {
                "message": error_message,
                "type": "api_error"
            }
        });

        (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
