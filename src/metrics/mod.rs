// Metrics module
// Public interface for logging and tracking metrics

mod logger;
mod similarity;
mod trends;
mod types;

pub use logger::MetricsLogger;
pub use similarity::semantic_similarity;
pub use trends::{Trend, TrainingTrends};
pub use types::{RequestMetric, ResponseComparison};
