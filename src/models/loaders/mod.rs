// Model family-specific loaders
// Each loader implements loading for a specific architecture (Qwen, Gemma, Llama, etc.)

pub mod qwen;

// Future loaders (Phase 3-5)
// #[cfg(target_os = "macos")]
// pub mod coreml;
// pub mod gemma;
// pub mod llama;
// pub mod mistral;
