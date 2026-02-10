// Gemma Loader - Google Gemma 2 models
// Loads Gemma 2 models (2B/9B/27B) on any backend (Metal, CPU, CUDA)

use anyhow::{Context, Result};
use candle_core::{Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::gemma2::{Config as Gemma2Config, Model as Gemma2Model};
use std::path::Path;
use tokenizers::Tokenizer;

use crate::models::unified_loader::ModelSize;
use crate::models::TextGeneration;

/// Gemma generator implementing TextGeneration trait
pub struct GemmaGenerator {
    model: Gemma2Model,
    tokenizer: Tokenizer,
    config: Gemma2Config,
    device: Device,
    name: String,
}

impl TextGeneration for GemmaGenerator {
    fn generate(&mut self, input_ids: &[u32], max_new_tokens: usize) -> Result<Vec<u32>> {
        // Clear KV cache from any previous generation
        self.model.clear_kv_cache();

        let mut generated_ids = input_ids.to_vec();

        for step in 0..max_new_tokens {
            // Determine what to pass: full sequence on first iteration, only new token afterwards
            let (input_for_forward, seqlen_offset) = if step == 0 {
                // First iteration: pass full prompt
                (&generated_ids[..], 0)
            } else {
                // Subsequent iterations: pass only the last token
                (&generated_ids[generated_ids.len() - 1..], generated_ids.len() - 1)
            };

            let input_tensor = Tensor::new(input_for_forward, &self.device)
                .context("Failed to create input tensor")?
                .unsqueeze(0)
                .context("Failed to add batch dimension")?;

            // Forward pass with appropriate seqlen_offset
            let logits = self
                .model
                .forward(&input_tensor, seqlen_offset)
                .with_context(|| {
                    let device_info = match &self.device {
                        Device::Metal(_) => "Metal (Apple Silicon GPU)",
                        Device::Cpu => "CPU",
                        Device::Cuda(_) => "CUDA GPU",
                    };
                    format!(
                        "Forward pass failed at step {} on {} device (seqlen_offset={}). \
                         Input shape: {:?}",
                        step,
                        device_info,
                        seqlen_offset,
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
                    "Failed to extract logits at position {} (logits shape: {:?}, input shape: {:?})",
                    last_pos,
                    logits.dims(),
                    input_tensor.dims()
                )
            })?;

            // Sample next token (greedy for now)
            let next_token = last_logits
                .argmax(0)
                .context("Failed to compute argmax")?
                .to_scalar::<u32>()
                .context("Failed to convert token to scalar")?;

            // Check for EOS token (Gemma typically uses token ID 1 for EOS)
            if next_token == self.tokenizer.token_to_id("<eos>").unwrap_or(1) {
                break;
            }

            generated_ids.push(next_token);

            // Stop if max position embeddings reached
            if generated_ids.len() >= self.config.max_position_embeddings {
                break;
            }
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

/// Load Gemma model from cache directory with specified device
///
/// # Arguments
/// * `model_path` - Directory containing config.json, tokenizer.json, and model weights
/// * `size` - Model size variant (Small=2B, Medium=9B, Large/XLarge=27B)
/// * `device` - Device to run on (Metal, CPU, CUDA)
///
/// # Returns
/// Boxed TextGeneration implementation
pub fn load(model_path: &Path, size: ModelSize, device: Device) -> Result<Box<dyn TextGeneration>> {
    let size_str = size.to_size_string(crate::models::unified_loader::ModelFamily::Gemma2);
    tracing::info!("Loading Gemma {} from {:?} on {:?}", size_str, model_path, device);

    // 1. Load model configuration
    let config_path = model_path.join("config.json");

    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "config.json not found in {:?}\n\
             \n\
             This usually means the model download was incomplete.\n\
             \n\
             Try:\n\
             1. Set up your HuggingFace token (see README.md)\n\
             2. Delete cache: rm -rf {:?}\n\
             3. Restart Shammah to re-download",
            config_path,
            model_path
        ));
    }

    let config: Gemma2Config = serde_json::from_reader(
        std::fs::File::open(&config_path)
            .with_context(|| format!("Failed to open config at {:?}", config_path))?,
    )
    .context("Failed to parse Gemma2 config.json")?;

    tracing::debug!(
        "Loaded config: vocab_size={}, hidden_size={}, num_layers={}",
        config.vocab_size,
        config.hidden_size,
        config.num_hidden_layers
    );

    // 2. Load tokenizer
    let tokenizer_path = model_path.join("tokenizer.json");
    let tokenizer = Tokenizer::from_file(&tokenizer_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to load tokenizer from {:?}: {}",
            tokenizer_path,
            e
        )
    })?;

    tracing::debug!(
        "Loaded tokenizer with vocab size: {}",
        tokenizer.get_vocab_size(true)
    );

    // 3. Load model weights from safetensors
    let weights_paths = find_safetensors_files(model_path)?;
    tracing::info!("Loading weights from {} file(s)", weights_paths.len());

    // Use F16 for Metal (GPU optimized), F32 for CPU
    let dtype = match &device {
        Device::Metal(_) => {
            tracing::info!("Using F16 precision for Metal GPU");
            candle_core::DType::F16
        }
        _ => {
            tracing::info!("Using F32 precision for CPU/CUDA");
            candle_core::DType::F32
        }
    };

    let vb = unsafe {
        VarBuilder::from_mmaped_safetensors(&weights_paths, dtype, &device)?
    };

    // 4. Build Gemma model architecture with loaded weights
    // Use flash attention on CUDA for better performance
    let use_flash_attn = matches!(device, Device::Cuda(_));

    let model = Gemma2Model::new(use_flash_attn, &config, vb)
        .context("Failed to build Gemma2 model from weights")?;

    tracing::info!("Successfully loaded Gemma {}", size_str);

    let name = format!("Gemma 2 {}", size_str);

    Ok(Box::new(GemmaGenerator {
        model,
        tokenizer,
        config,
        device,
        name,
    }))
}

/// Find safetensors files in cache directory
///
/// Handles both single file (model.safetensors) and sharded files
/// (model-00001-of-00002.safetensors, etc.)
fn find_safetensors_files(cache_dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    // Try single file first
    let single_file = cache_dir.join("model.safetensors");
    if single_file.exists() {
        tracing::debug!("Found single safetensors file");
        return Ok(vec![single_file]);
    }

    // Try sharded files - collect all shards
    let mut shards = Vec::new();
    for entry in std::fs::read_dir(cache_dir).context("Failed to read cache directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if let Some(filename) = path.file_name() {
            if let Some(name) = filename.to_str() {
                if name.starts_with("model-") && name.ends_with(".safetensors") {
                    shards.push(path);
                }
            }
        }
    }

    if !shards.is_empty() {
        // Sort shards by name to ensure correct order
        shards.sort();
        tracing::info!("Found {} sharded safetensors files", shards.len());
        return Ok(shards);
    }

    Err(anyhow::anyhow!(
        "No safetensors files found in {:?}. Expected model.safetensors or model-*-of-*.safetensors",
        cache_dir
    ))
}

/// Check if model is loadable from cache directory
pub fn is_loadable(cache_dir: &Path) -> bool {
    let has_config = cache_dir.join("config.json").exists();
    let has_tokenizer = cache_dir.join("tokenizer.json").exists();
    let has_weights = find_safetensors_files(cache_dir).is_ok();

    has_config && has_tokenizer && has_weights
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::common::get_device;

    #[test]
    fn test_is_loadable_missing_files() {
        let temp_dir = std::env::temp_dir().join("test_gemma_missing");
        std::fs::create_dir_all(&temp_dir).ok();

        // Should return false when files are missing
        assert!(!is_loadable(&temp_dir));

        // Cleanup
        std::fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    #[ignore] // Requires actual downloaded model
    fn test_load_gemma_model() {
        let device = get_device().unwrap();
        let cache_dir = dirs::home_dir()
            .unwrap()
            .join(".cache/huggingface/hub/models--google--gemma-2-2b-it/snapshots");

        // Find the latest snapshot
        if let Ok(entries) = std::fs::read_dir(&cache_dir) {
            for entry in entries.flatten() {
                let snapshot_dir = entry.path();
                if snapshot_dir.is_dir() && is_loadable(&snapshot_dir) {
                    let result = load(&snapshot_dir, ModelSize::Small, device);
                    match result {
                        Ok(mut generator) => {
                            println!("Successfully loaded model from {:?}", snapshot_dir);

                            // Try generating tokens
                            let input_ids = vec![1, 2, 3]; // Dummy token IDs
                            let output = generator.generate(&input_ids, 5);
                            match output {
                                Ok(tokens) => println!("Generated {} tokens", tokens.len()),
                                Err(e) => println!("Generation error: {}", e),
                            }
                        }
                        Err(e) => {
                            println!("Failed to load model: {}", e);
                        }
                    }
                    return;
                }
            }
        }

        println!("No Gemma model found in cache - run download test first");
    }
}
