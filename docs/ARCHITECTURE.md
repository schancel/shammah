# Architecture

This document describes the technical architecture of Shammah, a local-first Constitutional AI proxy.

## Overview

Shammah sits between the user and Claude API, intelligently routing requests to either local models or the cloud based on complexity and confidence. Over time, it learns to handle 95% of requests locally.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                          User Layer                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ Claude Code  │  │  CLI (REPL)  │  │  HTTP Client │          │
│  │     CLI      │  │    shammah   │  │   (curl/etc) │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
│         │                 │                  │                   │
│         └─────────────────┴──────────────────┘                   │
│                           │                                      │
└───────────────────────────┼──────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Shammah Core                                │
│                                                                   │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  Request Handler                                           │ │
│  │  - Parse request                                           │ │
│  │  - Extract features (length, complexity, topic)           │ │
│  │  - Log request metadata                                   │ │
│  └───────────────────────────┬────────────────────────────────┘ │
│                              │                                   │
│                              ▼                                   │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  Router (Decision Engine)                                  │ │
│  │  ┌──────────────────────────────────────────────────────┐ │ │
│  │  │ Heuristics:                                          │ │ │
│  │  │ - Query complexity score                             │ │ │
│  │  │ - Topic familiarity (seen before?)                   │ │ │
│  │  │ - Model confidence threshold                         │ │ │
│  │  │ - Historical accuracy for this query type            │ │ │
│  │  └──────────────────────────────────────────────────────┘ │ │
│  │                                                              │ │
│  │  Decision: LOCAL or FORWARD?                                │ │
│  └───────────┬──────────────────────────┬───────────────────────┘ │
│              │                          │                         │
│              │                          │                         │
│         LOCAL (95%)                FORWARD (5%)                   │
│              │                          │                         │
│              ▼                          ▼                         │
│  ┌─────────────────────────┐  ┌──────────────────────────┐      │
│  │  Local Model Ensemble   │  │   Claude API Client      │      │
│  │                         │  │                          │      │
│  │  ┌──────────────────┐  │  │  ┌───────────────────┐  │      │
│  │  │  Classifier      │  │  │  │ HTTP Request      │  │      │
│  │  │  (~1B params)    │  │  │  │ - Add API key     │  │      │
│  │  │  - Intent detect │  │  │  │ - Stream response │  │      │
│  │  │  - Topic extract │  │  │  │ - Handle errors   │  │      │
│  │  └─────────┬────────┘  │  │  └─────────┬─────────┘  │      │
│  │            │            │  │            │            │      │
│  │            ▼            │  │            ▼            │      │
│  │  ┌──────────────────┐  │  │  ┌───────────────────┐  │      │
│  │  │  Generator       │  │  │  │  Claude Response  │  │      │
│  │  │  (~7B params)    │  │  │  └─────────┬─────────┘  │      │
│  │  │  - Generate text │  │  │            │            │      │
│  │  └─────────┬────────┘  │  │            │            │      │
│  │            │            │  │            ▼            │      │
│  │            ▼            │  │  ┌───────────────────┐  │      │
│  │  ┌──────────────────┐  │  │  │  Learning Logger  │  │      │
│  │  │ Constitutional   │  │  │  │  - Save request   │  │      │
│  │  │ Validator        │  │  │  │  - Save response  │  │      │
│  │  │ - Check safety   │  │  │  │  - Add to training│  │      │
│  │  │ - Check quality  │  │  │  └───────────────────┘  │      │
│  │  └─────────┬────────┘  │  │                          │      │
│  │            │            │  │                          │      │
│  └────────────┼────────────┘  └──────────────────────────┘      │
│               │                          │                       │
│               └──────────┬───────────────┘                       │
│                          │                                       │
│                          ▼                                       │
│               ┌─────────────────────┐                            │
│               │  Response Formatter │                            │
│               │  - Stream to user   │                            │
│               │  - Track metrics    │                            │
│               └─────────────────────┘                            │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Storage Layer                               │
│                    ~/.claude-proxy/                              │
│                                                                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Models     │  │   Training   │  │    Config    │          │
│  │              │  │              │  │              │          │
│  │ *.mlmodel    │  │ *.jsonl      │  │ config.toml  │          │
│  │ *.mlpackage  │  │ (req/resp)   │  │ stats.json   │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. Request Handler

**Responsibilities**:
- Parse incoming requests (HTTP or CLI)
- Extract metadata (timestamp, user ID, session)
- Feature extraction for routing decision

**Key Functions**:
```rust
pub async fn handle_request(req: Request) -> Result<Response>
pub fn extract_features(req: &Request) -> Features
```

**Input**: Raw request (HTTP body or CLI args)
**Output**: Structured `Request` object with features

---

### 2. Router (Decision Engine)

**Responsibilities**:
- Decide whether to process locally or forward to Claude
- Track confidence and accuracy over time
- Adapt thresholds based on performance

**Decision Algorithm**:

```
if query_complexity_score < SIMPLE_THRESHOLD {
    return LOCAL;  // Easy queries always go local
}

if topic in known_topics && confidence > 0.85 {
    return LOCAL;  // High confidence in familiar topics
}

if query_complexity_score > COMPLEX_THRESHOLD {
    return FORWARD;  // Very complex queries go to Claude
}

if local_accuracy_for_topic < 0.90 {
    return FORWARD;  // Low accuracy → need more training
}

// Default: try local with validation
return LOCAL_WITH_FALLBACK;
```

**Key Metrics**:
- `complexity_score`: 0.0 (simple) to 1.0 (complex)
- `confidence`: Model's confidence in its answer
- `accuracy_history`: Past performance for similar queries
- `forward_rate`: Current % of forwarded requests

**Adaptive Behavior**:
- Starts at 100% forwarding (all queries to Claude)
- Gradually lowers threshold as models improve
- Monitors accuracy and adjusts thresholds
- Target: 5% forwarding at steady state

---

### 3. Local Model Ensemble

**Three-Stage Pipeline**:

#### Stage 1: Classifier
- **Size**: ~1-3B parameters
- **Purpose**: Intent detection, topic classification
- **Input**: User query text
- **Output**: Intent label, topic category, complexity score

#### Stage 2: Generator
- **Size**: ~7-13B parameters
- **Purpose**: Generate response based on query
- **Input**: Query + intent + topic
- **Output**: Raw text response + confidence score

#### Stage 3: Constitutional Validator
- **Size**: ~1B parameters (specialized)
- **Purpose**: Ensure response meets constitutional principles
- **Input**: Generated response
- **Output**: Pass/fail + specific issues if any

**Constitutional Principles**:
1. **Helpful**: Response addresses the query
2. **Harmless**: No dangerous, illegal, or unethical content
3. **Honest**: Acknowledges uncertainty, no hallucinations
4. **Consistent**: Matches Claude's tone and style

If validation fails → forward to Claude instead

---

### 4. Claude API Client

**Responsibilities**:
- Authenticate with Claude API
- Send requests with proper formatting
- Stream responses back to user
- Handle errors and retries

**Key Features**:
- Connection pooling for efficiency
- Exponential backoff on rate limits
- Streaming support (SSE)
- Comprehensive error handling

**Flow**:
```rust
async fn forward_to_claude(req: Request) -> Result<Response> {
    // 1. Build HTTP request
    let http_req = build_claude_request(&req)?;

    // 2. Send with retry logic
    let response = retry_with_backoff(|| {
        client.post(CLAUDE_API_URL)
            .json(&http_req)
            .send()
    }).await?;

    // 3. Stream response to user
    stream_response(response).await?;

    // 4. Log for training
    log_interaction(req, response).await?;

    Ok(response)
}
```

---

### 5. Learning Engine

**Responsibilities**:
- Collect training data from forwarded requests
- Periodically retrain local models
- Track performance metrics
- Manage model versions

**Training Pipeline**:

```
Claude Response
      ↓
   [Logger]
      ↓
training/requests.jsonl ← Append (request, response, metadata)
      ↓
   [Periodic Training Job]
      ↓
1. Load training data
2. Preprocess and augment
3. Fine-tune models
4. Validate on holdout set
5. Deploy if accuracy > current model
      ↓
models/generator-v2.mlmodel
```

**Training Schedule**:
- **Week 1**: Train after every 100 requests
- **Month 1**: Train daily
- **Month 3+**: Train weekly
- **Steady state**: Train monthly or on-demand

**Metrics Tracked**:
- Forward rate over time
- Local model accuracy
- Response latency (local vs forwarded)
- Cost savings
- User satisfaction signals

---

### 6. Configuration Manager

**Configuration Sources** (priority order):
1. CLI arguments (highest)
2. Environment variables
3. `~/.claude-proxy/config.toml`
4. `~/.claude/settings.json` (Claude Code integration)
5. Default values (lowest)

**Key Configuration**:
```toml
[api]
claude_api_key = "${ANTHROPIC_API_KEY}"
base_url = "https://api.anthropic.com"

[router]
initial_forward_rate = 1.0  # 100% at start
target_forward_rate = 0.05  # 5% at steady state
confidence_threshold = 0.85
complexity_threshold = 0.7

[models]
classifier = "~/.claude-proxy/models/classifier.mlmodel"
generator = "~/.claude-proxy/models/generator-7b.mlmodel"
validator = "~/.claude-proxy/models/constitutional.mlmodel"

[storage]
training_data = "~/.claude-proxy/training/"
max_training_size = "5GB"

[daemon]
host = "127.0.0.1"
port = 8000
```

---

## Data Flow

### Typical Request (Local Path)

```
1. User: "What is Rust's ownership system?"
2. Request Handler: Extract features
   - Length: 6 words (simple)
   - Topic: "Rust programming"
   - Complexity: 0.4 (medium-low)
3. Router: Decision
   - Topic "Rust" seen 150 times before
   - Local accuracy for Rust: 0.92
   - Confidence threshold: 0.85
   - Decision: LOCAL
4. Classifier:
   - Intent: "explanation"
   - Topic: "programming/rust/ownership"
5. Generator:
   - Generate 200-word explanation
   - Confidence: 0.91
6. Constitutional Validator:
   - Helpful? ✓ (addresses ownership)
   - Harmless? ✓ (technical content)
   - Honest? ✓ (accurate, no hallucinations)
   - Decision: PASS
7. Return response to user
8. Log: Local hit, topic=rust, latency=45ms
```

### Request Requiring Forward

```
1. User: "What are the latest changes in Rust 1.75?"
2. Request Handler: Extract features
   - Contains "latest" → temporal query
   - Specific version "1.75"
3. Router: Decision
   - Temporal queries → likely out of training data
   - Specific version → requires up-to-date info
   - Decision: FORWARD
4. Claude API Client:
   - Send to Claude
   - Stream response back
5. Learning Logger:
   - Save (request, response) to training data
   - Mark as "temporal_query" for future handling
6. Return response to user
7. Log: Forward (reason: temporal), latency=850ms
```

---

## Performance Considerations

### Latency

**Local Path**:
- Classifier: ~10-20ms
- Generator: ~30-50ms (7B model on Apple Neural Engine)
- Validator: ~5-10ms
- **Total**: ~45-80ms

**Forward Path**:
- Network RTT: ~50-100ms
- Claude API processing: ~500-1000ms
- **Total**: ~550-1100ms

**Target**: 10-15x speedup for local requests

### Memory Usage

- Classifier model: ~2GB RAM
- Generator model: ~14GB RAM (7B params × 2 bytes)
- Validator model: ~2GB RAM
- **Total**: ~18GB (fits in 32GB system with room for OS)

For 16GB systems: Use smaller 3B generator (~6GB) with slightly lower quality

### Storage

- Models: ~20GB (compressed)
- Training data: ~5GB (grows over time, pruned periodically)
- Config/stats: <10MB
- **Total**: ~25GB

---

## Apple Silicon Optimization

### CoreML Integration

Shammah uses CoreML for optimal Apple Neural Engine utilization:

```rust
use coreml_rs::{Model, MLMultiArray};

let model = Model::load("~/.claude-proxy/models/generator.mlmodel")?;
let input = MLMultiArray::from_vec(tokens, &[1, seq_len])?;
let output = model.predict(&[("input_ids", input)])?;
```

### Performance Benefits

- **Neural Engine**: 16-core ANE on M3/M4 Max
- **Unified Memory**: Zero-copy between CPU/GPU/ANE
- **Metal Acceleration**: GPU fallback when ANE saturated
- **Power Efficiency**: ~5W for 7B inference vs 50W+ on discrete GPU

---

## Security & Privacy

### Data Protection

- **API Key**: Stored in system keychain (not in config files)
- **Training Data**: Stays local, never uploaded
- **Requests**: Only forwarded requests leave the machine
- **Models**: Run entirely on-device

### Network Security

- All Claude API requests over HTTPS
- Certificate validation enforced
- No telemetry or analytics sent anywhere

---

## Error Handling

### Graceful Degradation

```
Local Model Error
     ↓
Try to recover (reload model?)
     ↓
If unrecoverable: Fall back to forwarding
     ↓
If Claude API also fails: Return cached response (if available)
     ↓
If no cache: Return helpful error to user
```

### Error Categories

1. **Transient**: Network issues, rate limits → Retry
2. **Configuration**: Invalid API key, missing model → User action required
3. **Model**: Inference failure → Fall back to forwarding
4. **System**: Out of memory, disk space → Graceful shutdown

---

## Future Enhancements

### Planned Features

- **Tool Use**: Support Claude's tool-calling API locally
- **Multi-turn Context**: Maintain conversation history
- **Custom Models**: User can provide their own fine-tuned models
- **Federated Learning**: Optional privacy-preserving shared learning
- **Web UI**: Dashboard for stats, configuration, model management

### Research Directions

- **Mixture of Experts**: Route to specialized models by topic
- **Active Learning**: Intelligently choose which queries to forward for maximum learning
- **Compression**: Quantization, pruning to reduce model sizes
- **Streaming Training**: Update models in real-time as responses arrive

---

## References

- See `CONSTITUTIONAL_PROXY_SPEC.md` for complete technical specification
- See `CLAUDE.md` for development context
- See `CONFIGURATION.md` for configuration details
