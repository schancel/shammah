# Architecture

This document describes the technical architecture of Shammah, a local-first Constitutional AI proxy that uses a 3-model ensemble trained on your Claude usage.

## Overview

Shammah sits between the user and Claude API, intelligently routing requests using three specialized neural networks. Over time, it learns from your actual Claude usage to handle 95% of requests locally.

**Key Innovation:** All models are **trained from scratch** on YOUR Claude interactions, not pre-trained models.

## The 3-Model Ensemble

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

### Phase 1 (Current)
- **Language:** Rust
- **ML:** None yet (using placeholders)
- **Storage:** JSONL (metrics collection)

### Phase 2 (Training)
- **Training:** PyTorch or Candle (Rust-native)
- **Format:** ONNX (cross-platform)
- **Optimization:** Quantization, pruning

### Phase 3+ (Inference)
- **Runtime:** ONNX Runtime or CoreML
- **Hardware:** Apple Neural Engine + GPU
- **Optimization:** INT8 quantization, graph optimization

## Performance Targets

### Phase 1 (Current)
- Forward rate: 70-80% (templates)
- Response time (local): <50ms
- Response time (forward): ~1-2s

### Phase 2 (Initial Models)
- Forward rate: 30-40%
- Response time (local): 500ms-2s
- Accuracy: >90% (compared to Claude)

### Phase 3+ (Optimized)
- Forward rate: 5-10%
- Response time (local): 200-500ms
- Accuracy: >95%
- Cost reduction: 76%

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

## Future Optimizations

### Phase 4: Apple Neural Engine
- Convert to CoreML format
- Utilize ANE for all models
- Target <100ms end-to-end latency

### Phase 5: Continuous Learning
- Online learning from user feedback
- Adaptive thresholds
- Personalized to user's domain

### Phase 6: Multi-Modal
- Image understanding
- Code execution
- Web search integration
