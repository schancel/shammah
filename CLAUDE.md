# CLAUDE.md - AI Assistant Context

This document provides context for AI assistants (like Claude Code) working on the Shammah project.

## Project Context

**Project Name**: Shammah (שָׁמָה - "watchman/guardian")
**Purpose**: Local-first AI coding assistant with continuous improvement
**Core Innovation**: Pre-trained Qwen models + weighted LoRA fine-tuning for domain adaptation

### The Problem

Traditional AI coding assistants require:
- Constant internet connection
- High API costs for every query
- No learning from your specific patterns
- Months of training before becoming useful
- Privacy concerns (code sent to cloud)

### The Solution

Shammah provides **immediate quality** with **continuous improvement**:
1. Uses pre-trained Qwen models (works well from day 1)
2. Loads instantly with progressive bootstrap (<100ms startup)
3. Learns from weighted feedback via LoRA fine-tuning
4. Adapts to your coding style, frameworks, and patterns
5. Works offline after initial model download
6. Preserves privacy (code stays on your machine)

### Key Metrics

- **Startup Time**: <100ms (instant REPL)
- **First-Run Experience**: 0ms blocked (background download)
- **Quality Day 1**: High (pre-trained Qwen)
- **Quality Month 1**: Specialized (LoRA adapted to your domain)
- **RAM Support**: 8GB to 64GB+ Macs (adaptive model selection)

## Architecture Overview

### New Design: Pre-trained Qwen + LoRA Adaptation

Shammah uses **pre-trained Qwen models** with **weighted LoRA fine-tuning** instead of training from scratch:

```
User Request
    ↓
┌─────────────────────────────────────┐
│ Router with Model Check              │
│ - Crisis detection (safety)          │
│ - Model ready? Use local             │
│ - Model loading? Forward to Claude   │
└──────────┬──────────────────────────┘
           │
    Model Ready?
           │
    ├─ NO  → Forward to Claude API
    └─ YES → Continue
           │
           v
    ┌──────────────────────────────────┐
    │ Pre-trained Qwen Model           │
    │ (1.5B / 3B / 7B / 14B)          │
    │ + LoRA Adapters                  │
    │   (your learned patterns)        │
    └──────────┬───────────────────────┘
           │
           v
    ┌──────────────────────────────────┐
    │ Response to User                 │
    └──────────┬───────────────────────┘
           │
           v
    User Feedback?
           │
    ├─ 🔴 High-weight (10x)
    ├─ 🟡 Medium-weight (3x)
    └─ 🟢 Normal-weight (1x)
           │
           v
    ┌──────────────────────────────────┐
    │ Background LoRA Fine-Tuning      │
    │ - Collects weighted examples     │
    │ - Trains in batches (non-blocking)│
    │ - Saves adapters incrementally   │
    └──────────────────────────────────┘
```

### Why This Approach?

**Pre-trained Qwen vs. Training from Scratch:**
- ✅ **Immediate quality** - Works well from day 1
- ✅ **No cold start** - No months of data collection
- ✅ **Proven performance** - Qwen models are battle-tested
- ✅ **Broad knowledge** - Trained on diverse coding data

**LoRA vs. Full Fine-Tuning:**
- ✅ **Efficient** - Trains only 0.1-1% of parameters
- ✅ **Fast** - Minutes instead of hours
- ✅ **Low memory** - Works on consumer hardware
- ✅ **Multiple adapters** - Switch between domains easily
- ✅ **No degradation** - Base model quality preserved

**Weighted Examples vs. Uniform Training:**
- ✅ **Prioritize critical feedback** - Strategy errors get 10x weight
- ✅ **Faster adaptation** - Learn from mistakes more quickly
- ✅ **User control** - You decide what's important
- ✅ **Efficient learning** - Focus on what matters

### Core Components

#### 1. **Progressive Bootstrap** (`src/models/bootstrap.rs`)

**Purpose:** Instant startup with background model loading

**GeneratorState:**
- `Initializing` - Selecting model based on RAM
- `Downloading` - Downloading from HuggingFace Hub (first run)
- `Loading` - Loading weights into memory
- `Ready` - Model ready for use
- `Failed` - Load failed with error
- `NotAvailable` - Offline mode

**Bootstrap Flow:**
```rust
1. REPL appears instantly (<100ms)
2. Background task spawned
3. Check cache (HF Hub: ~/.cache/huggingface/)
4. Download if needed (with progress)
5. Load model weights
6. Update state to Ready
7. Future queries use local
```

**Key Files:**
- `src/models/bootstrap.rs` - BootstrapLoader, GeneratorState
- `src/models/download.rs` - ModelDownloader with HF Hub integration
- `src/models/model_selector.rs` - RAM-based model selection

#### 2. **Qwen Model Integration** (`src/models/qwen_loader.rs`)

**Purpose:** Load pre-trained Qwen2 models from safetensors

**Model Selection:**
- 8GB Mac → Qwen-2.5-1.5B (3GB RAM, fast)
- 16GB Mac → Qwen-2.5-3B (6GB RAM, balanced)
- 32GB Mac → Qwen-2.5-7B (14GB RAM, powerful)
- 64GB+ Mac → Qwen-2.5-14B (28GB RAM, maximum)

**Features:**
- Uses candle-transformers' built-in Qwen2 support
- Automatic tokenizer loading (tokenizer.json)
- Metal acceleration on Apple Silicon
- Graceful CPU fallback

**Key Files:**
- `src/models/qwen_loader.rs` - QwenLoader, LoadedQwenModel
- `src/models/generator_new.rs` - Unified GeneratorModel (Qwen + custom)
- `src/models/common.rs` - GeneratorConfig enum

#### 3. **LoRA Fine-Tuning** (`src/models/lora.rs`)

**Purpose:** Efficient domain-specific adaptation with weighted examples

**LoRAConfig:**
```rust
pub struct LoRAConfig {
    rank: usize,              // Low-rank dimension (4-64)
    alpha: f64,               // Scaling factor (1.0-32.0)
    dropout: f64,             // Regularization (0.0-0.3)
    target_modules: Vec<String>, // Layers to adapt

    // Weighted training
    high_weight: f64,         // Critical issues (10x)
    medium_weight: f64,       // Improvements (3x)
    normal_weight: f64,       // Good examples (1x)
}
```

**Weighted Training:**
- **High-weight (10x)**: Critical strategy errors
  - Example: "Never use .unwrap() in production"
  - Example: "This algorithm is O(n²), use O(n log n)"
  - Impact: Model strongly learns to avoid this

- **Medium-weight (3x)**: Style preferences
  - Example: "Prefer iterator chains over manual loops"
  - Example: "Use library X instead of library Y"
  - Impact: Model learns your preferred approach

- **Normal-weight (1x)**: Good examples
  - Example: "This is exactly right"
  - Example: "Remember this pattern"
  - Impact: Model learns normally

**Training Flow:**
```rust
1. User provides feedback with weight
2. Example stored in training buffer
3. Buffer reaches threshold (e.g., 10 examples)
4. Background training triggered (non-blocking)
5. LoRA adapter trained for N epochs
6. Adapter saved to ~/.shammah/adapters/
7. Adapter loaded for future queries
8. Process repeats continuously
```

**Key Files:**
- `src/models/lora.rs` - LoRAAdapter, LoRAConfig, weighted training
- `src/models/generator_new.rs` - fine_tune(), save_lora(), load_lora()

#### 4. **Router with Graceful Degradation** (`src/router/decision.rs`)

**Purpose:** Decide when to use local vs. Claude, handle model loading

**ForwardReasons:**
- `Crisis` - Safety issue detected
- `ModelNotReady` - Model still loading (progressive bootstrap)
- `NoMatch` - No local pattern match
- `LowConfidence` - Threshold router uncertain

**New Method:**
```rust
fn route_with_generator_check(
    query: &str,
    generator_is_ready: bool,
) -> RouteDecision
```

**Behavior:**
- Checks if generator loaded before considering local
- Forwards to Claude gracefully during bootstrap
- Enables seamless transition: Claude → local
- No blocking or errors during model load

**Key Files:**
- `src/router/decision.rs` - Router, RouteDecision, route_with_generator_check()
- `src/router/hybrid_router.rs` - Hybrid threshold + neural routing

#### 5. **Tool Execution System** (`src/tools/`)

**Purpose:** Enable Claude to inspect and modify code

**Tools:**
- `Read` - Read file contents (code, configs, docs)
- `Glob` - Find files by pattern (`**/*.rs`)
- `Grep` - Search with regex (`TODO.*`)
- `WebFetch` - Fetch URLs (documentation, examples)
- `Bash` - Execute commands (tests, build, etc.)
- `Restart` - Self-improvement (modify code, rebuild, restart)

**Features:**
- Multi-turn execution (tools → results → more tools)
- Real-time output visibility
- Infinite loop detection
- Conversation state validation
- Permission system with patterns
- XML-structured results

**Key Files:**
- `src/tools/executor.rs` - ToolExecutor, multi-turn loop
- `src/tools/implementations/` - Individual tool implementations
- `src/tools/permissions.rs` - PermissionManager, approval patterns

#### 6. **Claude Client** (`src/claude/`)

**Purpose:** Forward queries to Claude API, collect training data

**Features:**
- HTTP client with retry logic
- Streaming support (SSE parsing)
- Tool definitions sent with requests
- Logs (query, response) for LoRA training
- Graceful fallback when streaming unavailable

**Key Files:**
- `src/claude/client.rs` - ClaudeClient, send_message(), send_message_stream()
- `src/claude/types.rs` - API request/response types

#### 7. **Configuration** (`src/config/`)

**Purpose:** User preferences and API key management

**Config File (`~/.shammah/config.toml`):**
```toml
api_key = "your_anthropic_api_key"
streaming_enabled = true

[model]
# Optional: Force specific model size
# size = "3B"
device = "auto"  # "auto", "metal", "cpu"

[lora]
rank = 16
alpha = 32.0
learning_rate = 1e-4
batch_size = 4
auto_train = true
auto_train_threshold = 10

# Weighted feedback
high_weight = 10.0
medium_weight = 3.0
normal_weight = 1.0

adapters_dir = "~/.shammah/adapters"
```

**Key Files:**
- `src/config/mod.rs` - Config loading and validation

### Technology Stack

**Language:** Rust
- Memory safety without GC
- High performance
- Excellent Apple Silicon support

**ML Framework:** Candle
- Rust-native ML framework
- Metal backend for Apple Silicon
- Built-in Qwen2 support
- SafeTensors format

**Models:**
- Base: Qwen-2.5-1.5B/3B/7B/14B (pre-trained)
- Adapters: LoRA (domain-specific, ~5MB each)

**Storage:**
- Models: `~/.cache/huggingface/hub/` (standard HF cache)
- Adapters: `~/.shammah/adapters/`
- Config: `~/.shammah/config.toml`
- Metrics: `~/.shammah/metrics/`

**Dependencies:**
- `hf-hub` - HuggingFace Hub integration
- `indicatif` - Progress bars
- `candle-transformers` - Qwen2 support
- `tokenizers` - Tokenization
- `tokio` - Async runtime

## Key Design Decisions

### 1. Pre-trained vs. Training from Scratch

**Decision:** Use pre-trained Qwen models

**Rationale:**
- Immediate quality (works day 1)
- No cold start period (no months waiting for data)
- Proven performance (Qwen is well-tested)
- Broad knowledge base (trained on diverse code)
- LoRA provides domain adaptation without full retraining

**Trade-offs:**
- Pro: Instant value for users
- Pro: No expensive compute for initial training
- Pro: Smaller download than training from scratch
- Con: Slightly larger models than custom-trained ones
- Con: Includes knowledge not specific to user's domain (acceptable)

### 2. Weighted LoRA Training

**Decision:** Allow users to weight training examples

**Rationale:**
- Critical feedback (strategy errors) needs more impact
- Not all examples are equally important
- Faster adaptation to user's specific needs
- User control over what model learns

**Implementation:**
```rust
// High-weight example (10x impact)
lora.add_example(
    query,
    response,
    feedback,
    weight: 10.0,  // Critical issue
);

// This example will be sampled 10x more during training
// Model learns to avoid this pattern strongly
```

**Trade-offs:**
- Pro: Faster learning from critical feedback
- Pro: User control and transparency
- Pro: More efficient training (focus on important patterns)
- Con: Requires user to categorize feedback (worth it)

### 3. Progressive Bootstrap

**Decision:** Instant REPL startup with background model loading

**Rationale:**
- Professional UX (no waiting)
- Users can start querying immediately
- Model downloads don't block
- Graceful degradation (forward to Claude while loading)

**Implementation:**
```rust
// REPL appears instantly
let state = Arc::new(RwLock::new(GeneratorState::Initializing));

// Spawn background task
tokio::spawn(async move {
    loader.load_generator_async().await
});

// User can query immediately
// Routes forward to Claude until model ready
```

**Trade-offs:**
- Pro: 20-50x faster startup (2-5s → <100ms)
- Pro: First-run download doesn't block (5-30min → 0ms)
- Pro: Better user experience
- Con: Slightly more complex state management (acceptable)

### 4. Storage Location

**Decision:** Store everything in `~/.shammah/`

**Structure:**
```
~/.shammah/
├── config.toml              # User configuration
├── adapters/                # LoRA adapters
│   ├── coding_2026-02-06.safetensors
│   ├── python_async.safetensors
│   └── rust_advanced.safetensors
├── metrics/                 # Training data
│   └── 2026-02-06.jsonl
└── tool_patterns.json       # Approved tool patterns

~/.cache/huggingface/hub/    # Base models (HF standard)
├── models--Qwen--Qwen2.5-1.5B-Instruct/
├── models--Qwen--Qwen2.5-3B-Instruct/
└── models--Qwen--Qwen2.5-7B-Instruct/
```

**Rationale:**
- Simple, single directory for Shammah data
- Standard HF cache for base models (community convention)
- Clear separation: base models vs. adapters
- Easy to backup/share adapters

### 5. Command Name

**Decision:** Use `shammah` as the binary name

**Rationale:**
- Distinct from `claude` command
- Meaningful (Hebrew "watchman")
- Easy to type and remember

### 6. Three Operating Modes

**Interactive REPL:**
```bash
shammah
> How do I use lifetimes in Rust?
```

**Single Query:**
```bash
shammah query "What is 2+2?"
```

**HTTP Daemon:**
```bash
shammah daemon --bind 127.0.0.1:8000
```

## Development Guidelines

### Code Style

- **Formatting**: Always use `cargo fmt` before committing
- **Linting**: Run `cargo clippy` and address warnings
- **Documentation**: Doc comments for all public items
- **Error Messages**: User-friendly, actionable

### Error Handling

```rust
use anyhow::{Context, Result};

fn load_config() -> Result<Config> {
    let path = config_path()
        .context("Failed to determine config path")?;

    let contents = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config from {}", path.display()))?;

    toml::from_str(&contents)
        .context("Failed to parse config TOML")
}
```

### Testing

- **Unit tests**: Test individual functions
- **Integration tests**: Test cross-module behavior
- **Examples**: Demonstrate features

### Logging

```rust
use tracing::{debug, info, warn, error};

#[instrument]
async fn load_model(config: &QwenConfig) -> Result<GeneratorModel> {
    info!("Loading Qwen model");
    debug!(?config, "Model configuration");

    let model = QwenLoader::load(config)
        .context("Failed to load model")?;

    info!("Model loaded successfully");
    Ok(model)
}
```

### Git Workflow

**Commit After:**
- Implementing complete feature
- Fixing a bug
- Adding/updating documentation
- Refactoring (maintains functionality)

**Include in Commit:**
- Code changes
- Test updates
- Documentation updates
- Design document updates (if needed)

**Commit Message Format:**
```
feat: add weighted LoRA training

Enables users to weight training examples (10x/3x/1x) for faster
adaptation to critical feedback patterns.

Changes:
- Add weight parameter to LoRA training API
- Implement weighted sampling in training loop
- Add /feedback high|medium|normal commands
- Update documentation

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

## Current Project Status

**Phase**: Qwen Integration Complete (Phases 1-4) ✅
**Next**: Implement actual LoRA fine-tuning (currently placeholders)

### What's Done

- ✅ **Model Download** - HF Hub integration with progress
- ✅ **Model Selection** - RAM-based automatic selection
- ✅ **Qwen Loading** - Load pre-trained models from safetensors
- ✅ **Progressive Bootstrap** - Instant startup with background loading
- ✅ **Router Graceful Degradation** - Forward during model load
- ✅ **LoRA Placeholders** - API designed, ready for implementation
- ✅ **Tool Execution** - 6 tools working reliably
- ✅ **Streaming Responses** - Real-time output
- ✅ **Documentation** - Comprehensive phase docs

### What's Next

**Immediate (Current Sprint):**
- [ ] Implement actual LoRA fine-tuning (not placeholders)
- [ ] Add weighted example storage
- [ ] Implement background training loop
- [ ] Add /feedback commands for weighted training
- [ ] Test LoRA convergence on coding examples

**Near-term:**
- [ ] Multiple adapter support (switch domains)
- [ ] Adapter sharing (export/import)
- [ ] Training metrics visualization
- [ ] Automatic adapter selection

**Future:**
- [ ] Multi-model support (switch between Qwen sizes)
- [ ] Quantization for lower memory usage
- [ ] Batch inference for multiple queries
- [ ] CoreML export for Neural Engine

## Reference Documents

- **README.md**: User-facing documentation
- **CLAUDE.md**: This file (AI assistant context)
- **QWEN_INTEGRATION_COMPLETE.md**: Phases 1-4 implementation summary
- **PHASE_3_BOOTSTRAP_COMPLETE.md**: Progressive bootstrap details
- **PHASE_4_LORA_PLACEHOLDERS.md**: LoRA design (placeholders)
- **docs/TOOL_CONFIRMATION.md**: Tool permission system

## Questions?

If you're unsure about something:

1. Check this file (CLAUDE.md) for context
2. Check README.md for user perspective
3. Look at existing code for patterns
4. Ask the user if still unclear

## Key Principles

1. **Immediate Quality**: Pre-trained models work day 1
2. **Continuous Improvement**: LoRA fine-tuning adapts to user
3. **User Control**: Weighted feedback, manual overrides
4. **Privacy First**: Local inference, offline capability
5. **Professional UX**: Instant startup, graceful degradation
6. **Rust Best Practices**: Safe, idiomatic, performant code

---

This document evolves with the project. Keep it updated with new design decisions and context that helps AI assistants work effectively on Shammah.
