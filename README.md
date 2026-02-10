# Shammah

> **שָׁמָה** (Shammah) - Hebrew: "watchman" or "guardian"

A local-first AI coding assistant powered by pre-trained Qwen models that continuously improves through weighted LoRA fine-tuning. Works offline, starts instantly, and gets better at coding with every interaction.

## What is Shammah?

Shammah provides **immediate, high-quality AI assistance** using pre-trained Qwen models, then continuously improves based on your feedback. Unlike traditional approaches requiring months of training, Shammah works well from day 1 and gets better at your specific coding patterns over time.

**Key Innovation:** Weighted LoRA fine-tuning lets you flag critical feedback (like strategy mistakes) to have more impact on future responses.

## Key Features

### 🚀 **Instant Quality** - Pre-trained Qwen Models
- **Works from day 1** - No training period required
- **Adaptive sizing** - Automatically selects model based on your Mac's RAM
  - 8GB Mac → Qwen-1.5B (fast, efficient)
  - 16GB Mac → Qwen-3B (balanced)
  - 32GB Mac → Qwen-7B (powerful)
  - 64GB+ Mac → Qwen-14B (maximum capability)
- **Instant startup** - REPL appears in <100ms with background model loading
- **Metal acceleration** - Uses Apple Silicon GPU automatically
- **Offline capable** - Works without internet after first download

### 📈 **Continuous Improvement** - Weighted LoRA Fine-Tuning
- **Learn from interactions** - Model adapts to your coding style and patterns
- **Weighted examples** - Flag critical feedback for stronger impact
  - 🔴 High weight (10x): "This strategy is wrong, never do this"
  - 🟡 Medium weight (3x): "This could be better, prefer approach X"
  - 🟢 Normal weight (1x): "This is good, remember this pattern"
- **Domain adaptation** - Specializes in your frameworks, libraries, and patterns
- **No degradation** - Base model quality preserved, only adds specialized knowledge
- **Efficient** - Trains only 0.1-1% of parameters, takes minutes not hours

### 🛠️ **Tool Execution** - Claude Can Inspect and Modify Code
- **Read files** - Inspect code, configs, documentation
- **Search codebase** - Glob patterns (`**/*.rs`) and regex (`TODO.*`)
- **Run commands** - Execute tests, build, run scripts
- **Web research** - Fetch documentation and examples
- **Self-improvement** - Modify own code and restart

### ✅ **Interactive Tool Confirmation** - Full Control
- **Approve once or remember** - Session or persistent approvals
- **Pattern-based** - Wildcards (`*.rs`) or regex matching
- **Manage patterns** - `/patterns` commands
- **Safe by default** - Requires approval for new patterns

### 📊 **HTTP Daemon Mode** - Multi-Client Server
- **Claude-compatible** - Drop-in replacement for Claude API
- **Session management** - Automatic cleanup, concurrent clients
- **Prometheus metrics** - Monitor usage and performance
- **Production-ready** - Run as service or in containers

## How It Works

```
User runs shammah
    ↓
REPL appears instantly (<100ms)
    ↓
Background: Download/load Qwen model
    ↓
┌─────────────────────────────────────┐
│   User Query                        │
└──────────┬──────────────────────────┘
           │
           v
┌──────────────────────────────────────┐
│  Router with Model Check             │
│  - Crisis detection (safety)         │
│  - Model ready? Use local            │
│  - Model loading? Forward to Claude  │
└──────────┬───────────────────────────┘
           │
    Model Ready?
           │
    ├─ NO  → Forward to Claude API
    └─ YES → Continue
           │
           v
    ┌──────────────────────────┐
    │  Qwen Model              │
    │  + LoRA Adapters         │
    │  (your customizations)   │
    └──────────┬───────────────┘
           │
           v
    ┌──────────────────────────┐
    │  Response to User        │
    └──────────┬───────────────┘
           │
           v
    User Feedback?
           │
    ├─ 🔴 Critical issue → High-weight training (10x)
    ├─ 🟡 Could improve → Medium-weight training (3x)
    └─ 🟢 Looks good → Normal-weight training (1x)
           │
           v
    ┌──────────────────────────┐
    │  LoRA Fine-Tuning        │
    │  (background, non-blocking)│
    └──────────────────────────┘
```

## Quick Start

### Installation

```bash
# Clone repository
git clone https://github.com/shammah/claude-proxy
cd claude-proxy

# Build with release optimizations
cargo build --release

# Install (optional)
cargo install --path .
```

### First Run

```bash
# Start Shammah (REPL appears instantly)
./target/release/shammah

# First time only: Model downloads in background
⏳ Downloading Qwen-2.5-3B (first time only)...
[=====>    ] 45% (2.1GB / 4.7GB)

# You can start asking questions immediately!
> How do I implement a binary search tree in Rust?

# Response from Claude while model downloads...

✓ Model ready - future queries will use local generation

> Explain Rust ownership to me
# Now using local Qwen model!
```

### HuggingFace Token Setup

**Important:** Qwen models require a HuggingFace authentication token to download. Follow these steps:

1. **Create a HuggingFace account** at https://huggingface.co/join (free)

2. **Generate an access token**:
   - Go to https://huggingface.co/settings/tokens
   - Click "New token"
   - Name: "Shammah" (or any name you prefer)
   - Type: "Read" (not "Write")
   - Click "Generate token"
   - Copy the token (starts with `hf_...`)

3. **Save token to file**:
   ```bash
   mkdir -p ~/.cache/huggingface
   echo "hf_YOUR_TOKEN_HERE" > ~/.cache/huggingface/token
   chmod 600 ~/.cache/huggingface/token
   ```

4. **Verify setup**:
   ```bash
   cat ~/.cache/huggingface/token  # Should show your token
   ```

That's it! Shammah will now be able to download Qwen models automatically.

**Note:** Without a token, Shammah will gracefully forward all queries to the Claude API instead.

### Configuration

```bash
# Configure API key (for forwarding and feedback)
export ANTHROPIC_API_KEY=your_key_here

# Or create config file
mkdir -p ~/.shammah
cat > ~/.shammah/config.toml <<EOF
api_key = "your_key_here"
streaming_enabled = true

[model]
# Optional: Force specific model size
# size = "1.5B"  # Options: "1.5B", "3B", "7B", "14B"
# If not specified, auto-selects based on RAM

[lora]
# LoRA fine-tuning configuration
rank = 16          # Low-rank dimension (4-64)
alpha = 32.0       # Scaling factor
learning_rate = 1e-4
batch_size = 4
auto_train = true  # Train on feedback automatically

# Weighted feedback thresholds
high_weight = 10.0    # Critical issues (strategy errors)
medium_weight = 3.0   # Improvements (better approaches)
normal_weight = 1.0   # Good examples (remember this)
EOF
```

### Basic Usage

```bash
# Interactive REPL mode
shammah

> Can you help me optimize this function?
> Read my src/main.rs and suggest improvements
> Run the tests to see if my changes work

# Single query mode
shammah query "What's the best way to handle errors in Rust?"

# Piped input
echo "Explain closures in Rust" | shammah

# HTTP daemon mode (multi-client server)
shammah daemon --bind 127.0.0.1:8000

# Test daemon
curl -X POST http://127.0.0.1:8000/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen-2.5-3b",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

## Providing Feedback for Training

Shammah learns from your feedback. You can weight examples to control how much impact they have:

### 🔴 High-Weight Feedback (Critical Issues)

Use this when the model makes **strategy mistakes** or does things that should **never** be done:

```bash
> /feedback high
Why: The model tried to use unwrap() in production code, which can panic.
This is a critical error - always use proper error handling.

# This feedback will have 10x impact on training
# Model will strongly learn to avoid this pattern
```

**Examples of critical issues:**
- Using `.unwrap()` or `.expect()` without good reason
- Suggesting `unsafe` code when safe alternatives exist
- Recommending inefficient algorithms for large datasets
- Security vulnerabilities (SQL injection, XSS, etc.)
- Anti-patterns specific to your codebase

### 🟡 Medium-Weight Feedback (Improvements)

Use this when the approach works but **could be better**:

```bash
> /feedback medium
Why: The model used manual iteration, but iterator methods would be cleaner.
Prefer .filter().map() chains over manual loops in Rust.

# This feedback will have 3x impact on training
# Model will learn your preferred style
```

**Examples of improvements:**
- Style preferences (iterator chains vs loops)
- Better library choices (use X instead of Y)
- More idiomatic patterns
- Performance optimizations
- Better variable naming

### 🟢 Normal-Weight Feedback (Good Examples)

Use this to **reinforce good behavior**:

```bash
> /feedback normal
This is exactly the right way to handle this pattern.
Remember this approach for similar situations.

# This feedback will have 1x impact on training
# Model learns this pattern normally
```

Or simply mark as good:
```bash
> /good
# Quick way to mark last response as good
```

### Automatic Training

By default, Shammah automatically fine-tunes in the background:
- Collects weighted examples during your session
- Trains in batches (every 10-20 examples)
- Non-blocking (doesn't interrupt your work)
- Saves adapters to `~/.shammah/adapters/`

You can also manually trigger training:
```bash
> /train
Training LoRA adapter on 47 weighted examples...
✓ Training complete (epoch 3/3, loss: 0.234)
✓ Adapter saved to ~/.shammah/adapters/coding_2026-02-06.safetensors

> /train status
Current adapter: coding_2026-02-06
Examples collected: 127 (23 high-weight, 45 medium-weight, 59 normal)
Last training: 5 minutes ago
Next auto-train: 8 examples remaining
```

## Tool Execution

Shammah gives Claude powerful tools to help you code:

```bash
# Read files
> Can you read my Cargo.toml and tell me about dependencies?
🔧 Tool: Read
   File: Cargo.toml
   Status: ✓ Success
[Shows file contents and analysis]

# Search codebase
> Find all TODO comments in Rust files
🔧 Tool: Glob
   Pattern: **/*.rs
   Found: 15 files
🔧 Tool: Grep
   Pattern: TODO.*
   Matches: 23
[Shows all TODOs with file locations]

# Run commands
> Run the test suite
🔧 Tool: Bash
   Command: cargo test
   Confirm? [y/N/always]: y
[Shows test output]

# Web research
> What's the latest stable Rust version?
🔧 Tool: WebFetch
   URL: https://www.rust-lang.org/
[Fetches and parses page]
```

### Tool Confirmation

For safety, tools require confirmation:

```
🔧 Tool Confirmation Required
   Tool: Bash
   Command: rm -rf /tmp/cache

Options:
  1. Approve once
  2. Approve for this session (bash: rm -rf /tmp/cache)
  3. Approve pattern for session (bash: rm -rf *)
  4. Remember exact command (persistent)
  5. Remember pattern (persistent)
  6. Deny

Choice [1-6]:
```

Manage saved patterns:
```bash
> /patterns              # List all saved patterns
> /patterns add          # Create new pattern interactively
> /patterns remove ID    # Remove specific pattern
> /patterns clear        # Clear all patterns
```

## Advanced Usage

### Model Management

```bash
# Check which model is being used
> /model status
Current model: Qwen-2.5-3B (Qwen/Qwen2.5-3B-Instruct)
RAM: 16GB (selected 3B model)
Device: Metal (Apple Silicon GPU)
Status: Ready
LoRA adapter: coding_2026-02-06 (127 examples)

# Switch models (requires download if not cached)
> /model select 7B
Switching to Qwen-2.5-7B...
⏳ Downloading model (first time)...
✓ Model ready

# View download cache
> /model cache
Models cached in ~/.cache/huggingface/hub/:
  ✓ Qwen-2.5-1.5B (1.5GB)
  ✓ Qwen-2.5-3B (3.0GB)
  ✓ Qwen-2.5-7B (7.0GB)
  ✗ Qwen-2.5-14B (not downloaded)
```

### LoRA Adapter Management

```bash
# List adapters
> /adapters list
Available adapters:
  1. coding_2026-02-06 (127 examples, 3.2MB)
  2. python_async_2026-02-05 (89 examples, 2.8MB)
  3. rust_advanced_2026-02-04 (156 examples, 4.1MB)

Current: coding_2026-02-06

# Switch adapters
> /adapters load rust_advanced_2026-02-04
✓ Loaded adapter: rust_advanced_2026-02-04

# Create new adapter
> /adapters new embedded_systems
✓ Created new adapter: embedded_systems
  This adapter will learn from your embedded systems work.

# Export/share adapters
> /adapters export rust_advanced
✓ Exported to: ~/Downloads/rust_advanced.safetensors
  Share this file with teammates to share learned patterns!

# Import adapter
> /adapters import ~/Downloads/team_patterns.safetensors
✓ Imported as: team_patterns
```

### Session Management

```bash
# Save session for later
> /save session.json
✓ Conversation saved to session.json

# Restore session
shammah --restore-session session.json

# Save and restart (preserves conversation)
> /restart
Saving session...
Building new binary...
✓ Restarting into new version...
[REPL restarts with conversation intact]
```

## Performance

### Startup Time
- **REPL available**: <100ms (instant)
- **Model loading** (background): 2-3 seconds from cache
- **First-run download**: 1.5-14GB (depending on model size)

### Response Time
- **Local generation**: 500ms-2s (depending on model size)
- **With LoRA adapter**: +50-100ms overhead
- **Forwarded to Claude**: Standard Claude API latency

### Resource Usage
- **RAM**: 3-28GB (depending on model size)
- **Disk**: 1.5-14GB for base model + ~5MB per adapter
- **CPU**: Minimal (uses GPU when available)

## Model Selection Guide

Shammah automatically selects the best model for your Mac, but you can override:

| Mac RAM | Default Model | Speed | Quality | Use Case |
|---------|--------------|-------|---------|----------|
| 8GB | Qwen-1.5B | Very Fast | Good | Quick queries, code completion |
| 16GB | Qwen-3B | Fast | Great | General coding, documentation |
| 32GB | Qwen-7B | Medium | Excellent | Complex reasoning, architecture |
| 64GB+ | Qwen-14B | Slower | Outstanding | Advanced tasks, large contexts |

**Recommendation:** Start with the default (auto-selected), then switch if needed.

## Continuous Learning Timeline

Unlike traditional approaches requiring months to collect training data, Shammah provides immediate value and improves continuously:

**Day 1:**
- ✅ High-quality responses (pre-trained Qwen)
- ✅ All coding queries work well
- 🔄 Start collecting feedback for fine-tuning

**Week 1:**
- ✅ Model learns your code style
- ✅ Adapts to your preferred libraries/frameworks
- 🔄 Building specialized adapter

**Month 1:**
- ✅ Specialized for your domain (Rust/Python/etc.)
- ✅ Remembers your critical feedback patterns
- ✅ Handles your specific codebase patterns

**Month 3+:**
- ✅ Highly specialized to your work
- ✅ Recognizes anti-patterns you've flagged
- ✅ Follows your architectural preferences
- ✅ Multiple adapters for different domains

## Why Shammah?

### vs. Claude API Directly
- ✅ **Works offline** - No network required after setup
- ✅ **Faster responses** - Local inference, no API latency
- ✅ **Learns your patterns** - Adapts to your specific needs
- ✅ **Privacy** - Your code stays on your machine

### vs. Training Custom Models
- ✅ **Immediate quality** - Works well from day 1
- ✅ **No training period** - Pre-trained Qwen models
- ✅ **Efficient learning** - LoRA trains only 0.1% of parameters
- ✅ **No expensive compute** - Trains on your Mac

### vs. Other Local AI
- ✅ **Tool execution** - Can inspect and modify code
- ✅ **Weighted learning** - Flag critical feedback
- ✅ **Instant startup** - Progressive bootstrap (<100ms)
- ✅ **Metal acceleration** - Uses Apple Silicon GPU

## Configuration Reference

Full configuration in `~/.shammah/config.toml`:

```toml
# API key for forwarding and feedback
api_key = "your_anthropic_api_key"

# Enable streaming responses
streaming_enabled = true

# Crisis keywords (safety mechanism)
crisis_keywords_path = "~/.shammah/crisis_keywords.txt"

# Metrics storage
metrics_dir = "~/.shammah/metrics"

[model]
# Model size selection (optional, auto-selects if not specified)
# Options: "1.5B", "3B", "7B", "14B"
# size = "3B"

# Device preference
# Options: "auto", "metal", "cpu"
device = "auto"

# Model cache location (optional, uses HF default if not specified)
# cache_dir = "~/.cache/huggingface/hub"

[lora]
# LoRA fine-tuning configuration
rank = 16              # Low-rank dimension (4-64)
alpha = 32.0           # Scaling factor (typically 2*rank)
dropout = 0.1          # Regularization (0.0-0.3)
learning_rate = 1e-4   # Training learning rate
batch_size = 4         # Examples per training batch
epochs = 3             # Training epochs per batch

# Target modules for LoRA (attention layers)
target_modules = ["q_proj", "v_proj"]

# Automatic training
auto_train = true      # Train automatically in background
auto_train_threshold = 10  # Train after N new examples

# Weighted feedback
high_weight = 10.0     # Critical issues (10x impact)
medium_weight = 3.0    # Improvements (3x impact)
normal_weight = 1.0    # Good examples (1x impact)

# Adapter storage
adapters_dir = "~/.shammah/adapters"

[server]
# HTTP daemon mode configuration
enabled = false
bind_address = "127.0.0.1:8000"
max_sessions = 100
session_timeout_minutes = 60
auth_enabled = false
api_keys = []
```

## Troubleshooting

### Model won't download
```bash
# Check network connection
curl -I https://huggingface.co

# Try manual download
python -c "from huggingface_hub import snapshot_download; snapshot_download('Qwen/Qwen2.5-3B-Instruct')"

# Check disk space
df -h ~/.cache/huggingface
```

### Out of memory
```bash
# Switch to smaller model
shammah
> /model select 1.5B

# Or force CPU (slower but uses less RAM)
export SHAMMAH_DEVICE=cpu
shammah
```

### Training not improving
```bash
# Check training examples
> /train status
Examples: 47

# Make sure you're providing weighted feedback
> /feedback high
Explain why this pattern is problematic...

# Manually trigger training
> /train
```

### Slow responses
```bash
# Check if using GPU
> /model status
Device: Metal ✓

# If CPU, enable Metal
> /model device metal

# Or use smaller model
> /model select 1.5B
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

Key areas for contribution:
- Additional tool implementations
- LoRA training optimizations
- Multi-GPU support
- Quantization for lower memory usage
- Additional model backends

## License

MIT OR Apache-2.0

---

**Shammah** - Your AI coding watchman that learns and improves with you. 🛡️
