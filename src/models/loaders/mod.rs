// Model family-specific loaders
// Each loader implements loading for a specific architecture (Qwen, Gemma, Llama, etc.)

pub mod gemma;
pub mod llama;
pub mod mistral;
pub mod qwen;

#[cfg(target_os = "macos")]
pub mod coreml;
