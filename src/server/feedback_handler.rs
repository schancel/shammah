// Feedback endpoint handler for training examples
//
// Allows clients to submit weighted training examples via HTTP API.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::models::WeightedExample;

/// Request body for /v1/feedback endpoint
#[derive(Debug, Deserialize)]
pub struct FeedbackRequest {
    /// Original query
    pub query: String,
    /// Model response
    pub response: String,
    /// Weight for this example (1.0 = normal, 3.0 = medium, 10.0 = high)
    pub weight: f64,
    /// Optional feedback note
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback: Option<String>,
}

/// Response body for /v1/feedback endpoint
#[derive(Debug, Serialize)]
pub struct FeedbackResponse {
    /// Status: "queued", "error"
    pub status: String,
    /// Optional message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Handle POST /v1/feedback - Submit training example
pub async fn handle_feedback(
    State(training_tx): State<Arc<mpsc::UnboundedSender<WeightedExample>>>,
    Json(request): Json<FeedbackRequest>,
) -> Result<Json<FeedbackResponse>, Response> {
    info!(
        weight = request.weight,
        query_len = request.query.len(),
        response_len = request.response.len(),
        "Received feedback submission"
    );

    // Validate weight
    if request.weight <= 0.0 {
        warn!(weight = request.weight, "Invalid weight (must be > 0)");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(FeedbackResponse {
                status: "error".to_string(),
                message: Some("Weight must be greater than 0".to_string()),
            }),
        )
            .into_response());
    }

    // Create weighted example
    let example = WeightedExample {
        query: request.query,
        response: request.response,
        weight: request.weight,
        feedback: request.feedback,
    };

    // Send to training worker
    if let Err(e) = training_tx.send(example) {
        warn!(error = %e, "Failed to send example to training worker");
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(FeedbackResponse {
                status: "error".to_string(),
                message: Some("Training worker unavailable".to_string()),
            }),
        )
            .into_response());
    }

    info!("Feedback queued successfully");

    Ok(Json(FeedbackResponse {
        status: "queued".to_string(),
        message: None,
    }))
}

/// Training status information
#[derive(Debug, Serialize)]
pub struct TrainingStatusResponse {
    /// Queue length (examples waiting to be processed)
    pub queue_length: usize,
    /// Whether training is currently active
    pub training_active: bool,
    /// Optional last training timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_training: Option<String>,
}

/// Handle GET /v1/training/status - Get training queue status
pub async fn handle_training_status() -> Json<TrainingStatusResponse> {
    // TODO: Implement actual status tracking
    // For now, return placeholder data
    Json(TrainingStatusResponse {
        queue_length: 0,
        training_active: false,
        last_training: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_request_parsing() {
        let json = r#"{
            "query": "What is 2+2?",
            "response": "4",
            "weight": 10.0,
            "feedback": "Critical: Missing explanation"
        }"#;

        let request: FeedbackRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.query, "What is 2+2?");
        assert_eq!(request.weight, 10.0);
    }
}
