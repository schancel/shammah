// Router module
// Public interface for routing decisions

mod decision;
mod model_router;

pub use decision::{ForwardReason, RouteDecision, Router};
pub use model_router::ModelRouter;
