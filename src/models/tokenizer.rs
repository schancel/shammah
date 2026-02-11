// Tokenizer module stub (Phase 4: Candle-based implementation removed)
//
// The original tokenizer module used Candle's tensor operations.
// For ONNX-based models, use the tokenizers crate directly with LoadedOnnxModel.

use anyhow::Result;

/// Text tokenizer (stub for compatibility)
///
/// Phase 4: This is a stub. Use tokenizers crate directly with ONNX models:
///
/// ```rust,ignore
/// use tokenizers::Tokenizer;
/// let tokenizer = Tokenizer::from_file("tokenizer.json")?;
/// let encoding = tokenizer.encode(text, true)?;
/// let tokens = encoding.get_ids();
/// ```
#[derive(Debug, Clone)]
pub struct TextTokenizer;

impl TextTokenizer {
    pub fn new(_vocab_size: usize) -> Result<Self> {
        anyhow::bail!(
            "TextTokenizer removed in Phase 4 (Candle-based).\n\
             Use tokenizers crate directly:\n\
             \n\
             use tokenizers::Tokenizer;\n\
             let tokenizer = Tokenizer::from_file(\"tokenizer.json\")?;"
        )
    }

    pub fn default() -> Self {
        Self
    }

    pub fn encode(&self, _text: &str) -> Result<Vec<u32>> {
        anyhow::bail!("TextTokenizer removed in Phase 4 (Candle-based)")
    }

    pub fn decode(&self, _tokens: &[u32]) -> Result<String> {
        anyhow::bail!("TextTokenizer removed in Phase 4 (Candle-based)")
    }
}
