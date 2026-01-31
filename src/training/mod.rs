// Training module - Batch training and checkpoint management

pub mod batch_trainer;
pub mod checkpoint;

pub use batch_trainer::{BatchTrainer, TrainingExample, TrainingResult};
pub use checkpoint::{Checkpoint, CheckpointManager};
