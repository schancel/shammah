// Model family-specific loaders
// Each loader implements loading for a specific architecture (Qwen, Gemma, Llama, etc.)

// ONNX Runtime loader (Phase 2: new unified backend)
pub mod onnx;
pub mod onnx_config;

// Candle-based loaders (to be removed in Phase 4)
pub mod gemma;
pub mod llama;
pub mod mistral;
pub mod qwen;

#[cfg(target_os = "macos")]
pub mod coreml;
