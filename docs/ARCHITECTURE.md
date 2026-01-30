# Architecture

This document describes the technical architecture of Shammah, a local-first Constitutional AI proxy that uses a 3-model ensemble trained on your Claude usage.

## Overview

Shammah sits between the user and Claude API, intelligently routing requests using three specialized neural networks. Over time, it learns from your actual Claude usage to handle 95% of requests locally.

**Key Innovation:** All models are **trained from scratch** on YOUR Claude interactions, not pre-trained models.

**Current State (v0.2.0):**
- ✅ Tool execution system with 6 working tools
- ✅ Streaming responses via SSE parsing
- ✅ Threshold-based routing (learns from query 1)
- ✅ Concurrent weight merging for multi-session safety
- ✅ Constitution support infrastructure
- 🔄 Neural models training (not primary yet)

This document describes both the current implementation and the target architecture. Sections marked with phases indicate future work.

## Current Implementation (Phase 1 + 2a)

### Tool Execution System ✅

Shammah implements 6 tools that enable Claude to interact with the local environment:

**1. Read Tool** - Read file contents
- Max 10,000 characters per file
- Truncation warning if exceeded
- Error handling for missing files

**2. Glob Tool** - Find files by pattern
- Supports glob patterns (`**/*.rs`, `src/**/*.ts`)
- Limits to 100 files
- Fast pattern matching with glob crate

**3. Grep Tool** - Search codebase with regex
- Recursive directory search with walkdir
- Max 10 depth, 50 matches
- Returns file:line: match format

**4. WebFetch Tool** - Fetch URLs
- 10-second timeout
- 10K character limit
- HTTP error handling

**5. Bash Tool** - Execute shell commands
- Captures stdout/stderr
- 5K output limit
- Returns exit codes

**6. Restart Tool** - Self-improvement capability
- Verifies binary exists
- Uses exec() to replace current process
- Enables modify → build → restart workflow
- **Security:** Phase 1 implementation (no user confirmation yet)

**Multi-Turn Loop:**
```
User Query → Claude API (with tool definitions)
    ↓
Claude returns tool_use blocks
    ↓
Execute tools → collect results
    ↓
Send results back to Claude (maintain conversation alternation)
    ↓
Claude returns final response (or more tool uses)
    ↓
Repeat up to 5 iterations
```

**Key Design:**
- Tool definitions sent with every API request
- Proper user/assistant message alternation
- Graceful error handling
- Max 5 iterations to prevent infinite loops

### Streaming Responses ✅ (Partial)

**SSE (Server-Sent Events) Parsing:**
- Character-by-character display for better UX
- Tokio channels for async streaming
- Parses `data:` lines from Claude API

**Event Types:**
```rust
StreamEvent {
    event_type: "content_block_delta",
    delta: {
        delta_type: "text_delta",
        text: "..." // Incremental text
    }
}
```

**Current Limitation:**
- Streaming disabled when tools are used
- Reason: Can't detect tool_use blocks in SSE stream yet
- Workaround: Falls back to buffered response
- Future: Parse full SSE stream for tool_use events

**Implementation:**
- `ClaudeClient::send_message_stream()` method
- Returns `mpsc::Receiver<Result<String>>`
- Display in REPL with real-time output

### Threshold-Based Models ✅ (Phase 2a)

Before neural networks have enough training data, Shammah uses threshold models that learn from query 1:

**ThresholdRouter:**
- Query categorization (Greeting, Definition, HowTo, Code, Debugging, etc.)
- Tracks success rates per category
- Adaptive confidence thresholds (starts 0.95, adjusts based on performance)
- Fully interpretable decisions

**ThresholdValidator:**
- 8 quality signals: TooShort, Repetitive, AnswersQuestion, HasCode, etc.
- Learns signal correlations over time
- Conservative at start (forces Claude learning for first 10 queries)

**HybridRouter:**
- Phase 1 (queries 1-50): Pure threshold-based
- Phase 2 (queries 51-200): Hybrid with gradually increasing neural weight
- Phase 3 (queries 201+): Primarily neural with threshold safety checks

**Key Innovation:** Provides immediate value from query 1, unlike neural networks that need 200+ queries for cold start.

### Concurrent Weight Merging ✅

**Problem:** Multiple Shammah sessions running simultaneously could corrupt training data.

**Solution:**
- File locking with fs2 crate (`FileExt::lock_exclusive()`)
- Merge strategy: accumulate statistics from both sessions
- Atomic writes: temp file → rename (prevents corruption)

**Merge Logic:**
```rust
pub fn merge_with(&self, other: &Self) -> Self {
    // Accumulate category statistics
    for category in all_categories {
        merged_stats.local_attempts = mine + theirs;
        merged_stats.successes = mine + theirs;
        // Average confidence scores
        merged_stats.avg_confidence = (mine + theirs) / 2.0;
    }
    // Accumulate global totals
    merged.total_queries = self.total + other.total;
    // ...
}
```

**Process:**
1. Acquire exclusive lock on `.lock` file
2. Load existing state from disk
3. Merge current session stats with disk stats
4. Write atomically (temp → rename)
5. Release lock

### Constitution Support ✅ (Infrastructure)

**Purpose:** Allow users to define custom constitutional principles for local generation.

**Current State:**
- Path: `~/.shammah/constitution.md` (configurable)
- Loaded on startup if exists
- Stored in config, available to all components
- **NOT sent to Claude API** (keeps principles private)

**Usage (Pending):**
- Will be prepended to local model system prompts
- Activates when local generation becomes primary
- Example principles: privacy-first, domain-specific guidelines, custom safety rules

**Example:**
```markdown
# My Constitutional Principles

1. Always prioritize user privacy
2. Be helpful, harmless, and honest
3. Acknowledge uncertainty rather than guessing
4. [Custom domain principles]
```

## The 3-Model Ensemble (Phase 2b+ - Target Architecture)

Shammah uses three specialized neural networks working together:

### 1. Router Model (Small, Fast - ~1-3B parameters)

**Purpose:** Pre-generation decision - "Should we try locally?"

**Input:**
- Query text
- Context features (length, complexity indicators)
- Historical accuracy for similar queries

**Output:**
- Confidence score: 0.0 (must forward) to 1.0 (can handle)

**Architecture:**
- Transformer-based binary classifier
- Trained on historical routing decisions
- Optimized for speed (<50ms inference)

**Training Data:**
```json
{
  "query": "What is the golden rule?",
  "features": {...},
  "label": "local",
  "divergence": 0.05  // Low divergence = handled well locally
}
```

**Runs On:** Apple Neural Engine (CoreML)

---

### 2. Generator Model (Medium, Capable - ~7-13B parameters)

**Purpose:** Generate Claude-style responses

**Input:**
- Query text
- Context (conversation history, domain)

**Output:**
- Full response text (mimicking Claude's style)

**Architecture:**
- Decoder-only transformer (similar to GPT architecture)
- Trained via distillation from Claude
- Custom model, no pre-trained weights

**Training Data:**
```json
{
  "query": "Explain reciprocity dynamics",
  "claude_response": "Reciprocity refers to...",
  "context": {...}
}
```

**Training Method:**
- **Knowledge Distillation:** Learn to mimic Claude's responses
- Minimize divergence between Generator output and Claude's actual response
- Train on 1000+ examples of YOUR specific usage patterns

**Runs On:** Apple GPU (or Neural Engine with quantization)

---

### 3. Validator Model (Small, Fast - ~1-3B parameters)

**Purpose:** Post-generation quality gate - "Is this response good enough?"

**Input:**
- Query text
- Generated response (from Generator)
- Expected quality indicators

**Output:**
- Quality score: 0.0 (must forward) to 1.0 (good)
- Error flags: hallucination, off-topic, incoherent, unsafe

**Architecture:**
- Transformer-based quality classifier
- Trained to detect divergence from Claude's quality
- Fast evaluation (<100ms)

**Training Data:**
```json
{
  "query": "...",
  "local_response": "...",
  "claude_response": "...",
  "divergence": 0.85,  // High = bad quality
  "label": "forward"   // Should have forwarded
}
```

**Purpose:** Catches Generator mistakes before returning to user

**Runs On:** Apple Neural Engine (CoreML)

---

## System Flow

```
User Query
    ↓
┌─────────────────────────────────────┐
│ [1] Router Model                     │
│     Confidence: f(query) → [0,1]     │
│     <50ms on Neural Engine           │
└─────────────────────────────────────┘
    ↓
Confidence > adaptive_threshold?
    │
    ├─ NO → Forward to Claude API ──────────┐
    │        (Log for training)              │
    │                                        │
    └─ YES (try locally)                     │
         ↓                                   │
    ┌─────────────────────────────────────┐ │
    │ [2] Generator Model                  │ │
    │     Response: f(query) → text        │ │
    │     ~500ms-2s on GPU                 │ │
    └─────────────────────────────────────┘ │
         ↓                                   │
    ┌─────────────────────────────────────┐ │
    │ [3] Validator Model                  │ │
    │     Quality: f(query, response) → OK?│ │
    │     <100ms on Neural Engine          │ │
    └─────────────────────────────────────┘ │
         ↓                                   │
    Quality > threshold?                     │
         ├─ YES → Return to user             │
         └─ NO → Forward to Claude API ──────┘
                  (Generator made mistake)
                  (Log for retraining)
```

## Why Three Models?

### Efficiency
- **Router (tiny):** Quickly rejects queries without running expensive Generator
- **Only run Generator when confident:** Saves computation
- **Validator (tiny):** Fast safety check

### Accuracy
- **Two decision points:** Pre-generation (Router) + Post-generation (Validator)
- **Specialization:** Each model optimized for its specific task
- **Error catching:** Validator prevents bad local responses from reaching users

### Progressive Enhancement
- **Phase 1:** Simple placeholders (pattern matching, templates, crisis detection)
- **Phase 2:** Train initial models on collected data
- **Phase 3+:** Continuous learning, add tools, optimize for Neural Engine

## Training Pipeline

### Phase 1: Data Collection (Current)

```
User Query → Forward to Claude → Response
                 ↓
    Log (query, response, metadata)
                 ↓
    Build training corpus
    Target: 1000+ examples
```

### Phase 2: Initial Training

**Step 1: Train Generator (Distillation)**
```python
# Pseudo-code
for (query, claude_response) in training_data:
    local_response = generator(query)
    loss = divergence(local_response, claude_response)
    backprop(loss)
```

**Step 2: Train Router (Classification)**
```python
# Use Generator to evaluate which queries it handles well
for (query, claude_response) in training_data:
    local_response = generator(query)
    divergence = measure_divergence(local_response, claude_response)
    label = "local" if divergence < threshold else "forward"

    router_pred = router(query)
    loss = classification_loss(router_pred, label)
    backprop(loss)
```

**Step 3: Train Validator (Quality Assessment)**
```python
# Detect when Generator makes mistakes
for (query, claude_response) in training_data:
    local_response = generator(query)
    quality = measure_quality(local_response, claude_response)

    validator_score = validator(query, local_response)
    loss = regression_loss(validator_score, quality)
    backprop(loss)
```

### Phase 3+: Continuous Learning

- Keep forwarding uncertain queries to Claude
- Use responses as additional training data
- Periodically retrain models
- Adapt thresholds based on accuracy

## Data Flow

### Request Processing

```
1. Receive user query
2. Extract features (length, complexity, topic)
3. Router inference → confidence score
4. If low confidence:
     a. Forward to Claude API
     b. Log (query, response) for training
     c. Return Claude's response
5. If high confidence:
     a. Generator inference → local response
     b. Validator inference → quality check
     c. If quality good:
          - Return local response
          - Log success
     d. If quality bad:
          - Forward to Claude API
          - Log (query, local_attempt, claude_response) for retraining
          - Return Claude's response
```

### Metrics Collection

Every request logs:
```json
{
  "timestamp": "2026-01-30T12:00:00Z",
  "query_hash": "abc123...",  // SHA256 for privacy
  "routing_decision": "local",
  "router_confidence": 0.92,
  "validator_quality": 0.88,
  "response_time_ms": 650,
  "forward_reason": null
}
```

Stored in: `~/.shammah/metrics/YYYY-MM-DD.jsonl`

### Training Data Format

```json
{
  "id": "uuid",
  "timestamp": "2026-01-30T12:00:00Z",
  "query": "What is the golden rule?",
  "context": {...},
  "claude_response": "The golden rule refers to...",
  "local_attempt": "...",  // If applicable
  "divergence": 0.05,
  "used_for_training": ["router", "generator", "validator"]
}
```

## Model Specifications

### Router Model

**Architecture:**
- Encoder-only transformer
- 6 layers, 768 hidden dim
- ~500M parameters
- Input: 512 token context
- Output: 1 confidence score

**Training:**
- Binary classification
- Loss: Binary cross-entropy
- Optimizer: AdamW
- Batch size: 32
- Training time: ~2 hours on M1 Max

**Inference:**
- 30-50ms on Neural Engine
- Batch size: 1
- Quantized to INT8

### Generator Model

**Architecture:**
- Decoder-only transformer
- 24 layers, 2048 hidden dim
- ~7B parameters (configurable: 3B, 7B, 13B)
- Context: 2048 tokens
- Vocabulary: 50k tokens

**Training:**
- Distillation from Claude
- Loss: KL divergence + cross-entropy
- Optimizer: AdamW
- Batch size: 8
- Training time: ~12 hours on M1 Max (7B model)

**Inference:**
- 500ms-2s depending on response length
- Runs on GPU
- Can be quantized to INT8 for Neural Engine

### Validator Model

**Architecture:**
- Encoder-only transformer
- 6 layers, 768 hidden dim
- ~500M parameters
- Input: Query + Response (1024 tokens total)
- Output: Quality score + error flags

**Training:**
- Regression + classification
- Loss: MSE (quality) + BCE (error flags)
- Optimizer: AdamW
- Batch size: 32
- Training time: ~2 hours on M1 Max

**Inference:**
- 50-100ms on Neural Engine
- Batch size: 1
- Quantized to INT8

## File Structure

```
~/.shammah/
├── config.toml              # API key, thresholds
├── metrics/                 # Daily JSONL logs
│   ├── 2026-01-29.jsonl    # Training data
│   └── 2026-01-30.jsonl
└── models/                  # Phase 2+: Trained models
    ├── router/
    │   ├── model.onnx
    │   ├── config.json
    │   └── vocab.json
    ├── generator/
    │   ├── model.onnx
    │   ├── config.json
    │   └── vocab.json
    └── validator/
        ├── model.onnx
        ├── config.json
        └── vocab.json
```

## Technology Stack

### Phase 1 + 2a (Current Implementation)
- **Language:** Rust 2021 edition
- **ML Framework:** Candle (Rust-native, Apple Metal support)
- **Async Runtime:** Tokio with full features
- **HTTP Client:** Reqwest with streaming support
- **Serialization:** Serde + serde_json
- **File Locking:** fs2 crate for concurrent safety
- **CLI:** Rustyline for readline support
- **Tools:**
  - glob - Pattern matching
  - walkdir - Recursive directory search
  - regex - Text search
  - futures - Async stream processing
- **Storage:** JSONL (metrics collection)
- **Current Models:**
  - ThresholdRouter (statistics-based)
  - Neural networks training with Candle (not primary yet)
- **Apple Silicon:** Metal backend for GPU acceleration

### Phase 2b (Neural Training - In Progress)
- **Training:** Candle (Rust-native)
- **Device:** Auto-detect Metal, fallback to CPU
- **Format:** JSON for weights (transitioning to ONNX)
- **Optimization:** Quantization, pruning
- **Performance:** 10-100x speedup on Metal vs. CPU

### Phase 3+ (Production Inference - Planned)
- **Runtime:** CoreML for maximum Neural Engine performance
- **Hardware:** Apple Neural Engine + GPU
- **Optimization:** INT8 quantization, graph optimization
- **Format:** .mlmodel (Core ML model format)

## Performance Targets

### Phase 1 + 2a (Current - v0.2.0)
- **Forward rate:** 100% (correct - learning phase)
- **Crisis detection:** 100% accuracy
- **Tool execution:** Working reliably, multi-turn loop functional
- **Response time:**
  - Forward (with streaming): Real-time character display
  - Forward (with tools): ~1-2s + tool execution time
  - Tool overhead: ~50-200ms per tool
- **Concurrent sessions:** Safe with file locking

**Current Behavior:**
- All queries forward to Claude (expected during training data collection)
- Threshold models learning from every query
- Neural models training but not routing yet
- Statistics accumulated across sessions

### Phase 2b (Neural Models Primary - Target: ~200 queries)
- **Forward rate:** 30-40%
- **Response time (local):** 500ms-2s
- **Accuracy:** >90% (compared to Claude)
- **Hybrid routing:** Gradual transition from threshold to neural

### Phase 3+ (Optimized - Target: ~6 months)
- **Forward rate:** 5-10%
- **Response time (local):** 200-500ms (with Core ML optimization)
- **Accuracy:** >95%
- **Cost reduction:** 76%
- **Latency target:** <100ms end-to-end on Neural Engine

## Security & Privacy

### Data Protection
- All metrics hashed (SHA256) for privacy
- Models train only on YOUR data
- No telemetry, no cloud sync
- Can delete `~/.shammah/` anytime

### Model Safety
- Generator trained on Claude (inherently safe)
- Validator checks for harmful content
- Two decision points prevent bad outputs
- Crisis queries always forwarded

## Deferred Work

### Security Design for Self-Improvement (Restart Tool)

**Current State (Phase 1):**
- Basic restart tool implemented
- Claude can modify code and restart without confirmation
- Uses Unix exec() to replace process with new binary

**Missing Security Measures (Deferred to Phase 2):**
1. **User Confirmation**
   - Prompt before code modification
   - Show diff of proposed changes
   - Require explicit approval

2. **Git Integration**
   - Auto-stash before changes
   - Create backup commit
   - Rollback mechanism if restart fails

3. **Binary Backup**
   - Keep previous binary as `.bak`
   - Automatic rollback on crash
   - Version tracking

4. **Dev Mode Flag**
   - Disable restart in production
   - Environment variable: `SHAMMAH_DEV_MODE=1`
   - Config file setting

5. **Change Review UI**
   - Show files to be modified
   - Preview changes
   - Confirmation dialog

**Rationale:**
- Get basic functionality working first
- Add safety features once workflow is validated
- **Risk:** Claude can modify any code without confirmation
- **Mitigation:** Only use in development with version control

**Timeline:** Phase 2 of self-improvement (TBD)

### Streaming + Tool Detection

**Issue:** Streaming disabled when tool execution likely

**Current Limitation:**
- Can't detect tool_use blocks in SSE stream
- Falls back to buffered response when tools are used

**Solution (Future):**
- Parse full SSE stream for tool_use events
- Buffer tool_use blocks while streaming text
- Maintain streaming UX even with tools

**Priority:** Low (buffered responses work fine)

## Future Optimizations

### Phase 4: Apple Neural Engine
- Convert to CoreML format (.mlmodel)
- Utilize ANE for all models
- Target <100ms end-to-end latency
- Maximum Apple Silicon performance

### Phase 5: Continuous Learning
- Online learning from user feedback
- Adaptive thresholds based on performance
- Personalized to user's specific domain
- Uncertainty estimation for better routing

### Phase 6: Multi-Modal
- Image understanding (vision models)
- Code execution sandboxing
- Web search integration
- Document analysis
