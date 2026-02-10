// Model family-specific loaders
// Each loader implements loading for a specific architecture (Qwen, Gemma, Llama, etc.)

pub mod gemma;
pub mod qwen;

#[cfg(target_os = "macos")]
pub mod coreml;

// Future loaders (Phase 5)
// pub mod llama;
// pub mod mistral;
