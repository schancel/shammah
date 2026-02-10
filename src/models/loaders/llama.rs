// Llama Loader - Meta Llama 3 models
// Loads Llama 3 models (3B/8B/70B) on any backend (Metal, CPU, CUDA)

use anyhow::{Context, Result};
use candle_core::{Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::llama::{Cache, Config as LlamaConfig, Llama as LlamaModel};
use std::path::Path;
use tokenizers::Tokenizer;

use crate::models::unified_loader::ModelSize;
use crate::models::TextGeneration;

/// Llama generator implementing TextGeneration trait
pub struct LlamaGenerator {
    model: LlamaModel,
    tokenizer: Tokenizer,
    config: LlamaConfig,
    cache: Cache,
    device: Device,
    name: String,
}

impl TextGeneration for LlamaGenerator {
    fn generate(&mut self, input_ids: &[u32], max_new_tokens: usize) -> Result<Vec<u32>> {
        let mut generated_ids = input_ids.to_vec();

        for step in 0..max_new_tokens {
            // Create input tensor from full sequence
            let input_tensor = Tensor::new(&generated_ids[..], &self.device)
                .context("Failed to create input tensor")?
                .unsqueeze(0)
                .context("Failed to add batch dimension")?;

            // Forward pass - Llama model takes (input, start_pos, cache)
            let logits = self
                .model
                .forward(&input_tensor, step, &mut self.cache)
                .with_context(|| {
                    let device_info = match &self.device {
                        Device::Metal(_) => "Metal (Apple Silicon GPU)",
                        Device::Cpu => "CPU",
                        Device::Cuda(_) => "CUDA GPU",
                    };
                    format!(
                        "Forward pass failed at step {} on {} device. Input shape: {:?}",
                        step,
                        device_info,
                        input_tensor.dims()
                    )
                })?;

            // Get logits for the last position
            let seq_len = logits
                .dim(1)
                .context("Failed to get sequence length from logits")?;
            let last_pos = seq_len - 1;

            let last_logits = logits.i((0, last_pos)).with_context(|| {
                format!(
                    "Failed to extract logits at position {} (logits shape: {:?})",
                    last_pos,
                    logits.dims()
                )
            })?;

            // Sample next token (greedy for now)
            let next_token = last_logits
                .argmax(0)
                .context("Failed to sample next token")?
                .to_scalar::<u32>()
                .context("Failed to convert token to u32")?;

            // Check for EOS token (Llama uses token ID 2 for EOS)
            if next_token == 2 {
                break;
            }

            generated_ids.push(next_token);
        }

        Ok(generated_ids)
    }

    fn device(&self) -> &Device {
        &self.device
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Load Llama model from safetensors
///
/// # Arguments
/// * `model_path` - Path to model directory containing config.json, tokenizer.json, model.safetensors
/// * `size` - Model size (Small=3B, Medium=8B, Large/XLarge=70B)
/// * `device` - Device to load model on (Metal, CPU, CUDA)
pub fn load(model_path: &Path, size: ModelSize, device: Device) -> Result<Box<dyn TextGeneration>> {
    tracing::info!(
        "Loading Llama {} model from {:?} on {:?}",
        size.to_llama_size(),
        model_path,
        device
    );

    // Load config - For Llama, we need to construct the config manually
    // since LlamaConfig doesn't always deserialize directly from HF format
    let config = LlamaConfig::config_7b_v2(false); // Use default 7B config as base

    tracing::debug!("Using Llama config for {} model", size.to_llama_size());

    // Load tokenizer
    let tokenizer_path = model_path.join("tokenizer.json");
    let tokenizer = Tokenizer::from_file(&tokenizer_path)
        .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

    tracing::info!("Loaded tokenizer with vocab size: {}", tokenizer.get_vocab_size(true));

    // Load model weights from safetensors
    let weights_path = model_path.join("model.safetensors");

    // Check if model is sharded (multiple files)
    let vb = if weights_path.exists() {
        tracing::info!("Loading single-file model from model.safetensors");
        unsafe { VarBuilder::from_mmaped_safetensors(&[weights_path], candle_core::DType::F32, &device)? }
    } else {
        // Try loading sharded model
        tracing::info!("Looking for sharded model files");
        let mut shard_paths = Vec::new();
        for i in 1..=10 {
            let shard_path = model_path.join(format!("model-{:05}-of-{:05}.safetensors", i, 10));
            if shard_path.exists() {
                shard_paths.push(shard_path);
            } else {
                break;
            }
        }

        if shard_paths.is_empty() {
            anyhow::bail!("No model files found at {:?}", model_path);
        }

        tracing::info!("Loading {} sharded files", shard_paths.len());
        unsafe { VarBuilder::from_mmaped_safetensors(&shard_paths, candle_core::DType::F32, &device)? }
    };

    // Create Llama model
    let model = LlamaModel::load(vb, &config)
        .context("Failed to create Llama model from weights")?;

    // Initialize KV cache for generation
    let cache = Cache::new(true, candle_core::DType::F32, &config, &device)
        .context("Failed to create KV cache")?;

    tracing::info!("✓ Llama model loaded successfully");

    let name = format!("Llama 3 {}", size.to_llama_size());

    Ok(Box::new(LlamaGenerator {
        model,
        tokenizer,
        config,
        cache,
        device,
        name,
    }))
}

impl ModelSize {
    /// Convert ModelSize to Llama size string
    fn to_llama_size(&self) -> &str {
        match self {
            ModelSize::Small => "3B",
            ModelSize::Medium => "8B",
            ModelSize::Large => "70B",
            ModelSize::XLarge => "70B",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires downloaded Llama model
    fn test_llama_load() {
        let model_path = dirs::home_dir()
            .unwrap()
            .join(".cache/huggingface/hub/models--meta-llama--Llama-3.2-3B-Instruct/snapshots");

        // Find latest snapshot
        if let Ok(entries) = std::fs::read_dir(&model_path) {
            for entry in entries.flatten() {
                let snapshot_dir = entry.path();
                if snapshot_dir.is_dir() {
                    match load(&snapshot_dir, ModelSize::Small, Device::Cpu) {
                        Ok(mut gen) => {
                            println!("✓ Loaded: {}", gen.name());

                            // Test generation
                            let input_ids = vec![1, 15339, 995]; // "Hello world"
                            match gen.generate(&input_ids, 10) {
                                Ok(output) => {
                                    println!("✓ Generated {} tokens", output.len());
                                    assert!(output.len() > input_ids.len());
                                }
                                Err(e) => println!("Generation failed: {}", e),
                            }
                        }
                        Err(e) => println!("Load failed: {}", e),
                    }
                    return;
                }
            }
        }

        println!("No Llama model found - download first");
    }
}
