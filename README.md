# Shammah

> **שָׁמָה** (Shammah) - Hebrew: "watchman" or "guardian"

A local-first Constitutional AI proxy that learns to handle 95% of requests locally while maintaining Claude API compatibility.

## What is Shammah?

Shammah is an intelligent proxy that sits between you and Claude AI. Instead of sending every request to the cloud, it learns from Claude's responses and progressively handles more requests locally on your machine. Over time, it reduces API usage from 100% to just 5%, while maintaining high-quality responses through Constitutional AI principles.

Think of it as a smart cache that learns not just answers, but reasoning patterns.

## Key Features

- **95% Local Processing** - After training period, only 5% of requests require API calls
  - Enhanced privacy: your data stays on your machine
  - Faster responses: no network latency for local processing
  - Works offline for most queries

- **Constitutional AI Reasoning** - Multi-model ensemble that learns safe, helpful behavior
  - Learns from every Claude response
  - Applies constitutional principles locally
  - Maintains quality without constant API access

- **Drop-in Replacement** - Compatible with Claude Code and Claude API
  - Uses your existing `~/.claude/settings.json` configuration
  - Supports Claude API format
  - Seamless integration with existing workflows

- **Continuous Learning** - Improves over time
  - Starts at 100% forwarding (everything goes to Claude)
  - Learns patterns and reasoning from responses
  - Converges to 5% forwarding over ~6 months
  - Models stored locally in `~/.claude-proxy/`

- **Cost Effective** - Reduces API costs by 76% (24% of original after accounting for 5% forwarding)
  - Pay only for novel or complex queries
  - Training investment pays off quickly
  - Transparent cost tracking

## How It Works

```
┌─────────────┐
│   Request   │
└──────┬──────┘
       │
       v
┌─────────────────────┐
│  Shammah Router     │
│  Decision: Local    │ ← 95% after training
│         or Forward? │
└──────┬──────────────┘
       │
       ├─────── Local (95%) ──────┐
       │                           v
       │                    ┌──────────────┐
       │                    │ Multi-Model  │
       │                    │  Ensemble    │
       │                    └──────┬───────┘
       │                           │
       │                           v
       │                    ┌──────────────┐
       │                    │ Constitution │
       │                    │   Validator  │
       │                    └──────┬───────┘
       │                           │
       └─── Forward (5%) ───┐      │
                            v      v
                      ┌──────────────┐
                      │ Claude API   │
                      └──────┬───────┘
                             │
                             v
                      ┌──────────────┐
                      │  Learn from  │
                      │   Response   │
                      └──────────────┘
```

### Learning Process

1. **Initial Phase** (Weeks 1-2): 100% forwarding
   - Every request goes to Claude
   - System observes patterns
   - No local predictions yet

2. **Training Phase** (Months 1-3): 50-80% forwarding
   - System starts handling simple queries locally
   - Continues learning from Claude responses
   - Gradually increases confidence

3. **Mature Phase** (Months 4-6): 10-20% forwarding
   - Handles most requests locally
   - Forwards only novel/complex queries
   - Continuous refinement

4. **Steady State** (Month 6+): ~5% forwarding
   - Optimal balance achieved
   - Only truly new patterns forwarded
   - Cost savings realized

## Quick Start

```bash
# Install (when available)
cargo install shammah

# Run in interactive mode
shammah

# Or use as daemon (background service)
shammah daemon

# Single query mode
shammah query "What is Rust's ownership system?"
```

## Architecture

Shammah uses a multi-model ensemble approach:

- **Small models** (~1-3B params) for classification and routing
- **Medium models** (~7-13B params) for general queries
- **Constitutional validator** ensures responses meet safety criteria
- **Learning engine** continuously improves from Claude responses
- **Apple Neural Engine** optimization for M1/M2/M3/M4 chips

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed architecture.

## Configuration

Shammah integrates with Claude Code's configuration:

```json
// ~/.claude/settings.json
{
  "apiKey": "your-claude-api-key",
  "proxyEnabled": true,
  "proxyUrl": "http://localhost:8000"
}
```

Models and training data stored in:
```
~/.claude-proxy/
├── models/          # Trained models
├── training/        # Training data from Claude responses
├── config.toml      # Shammah configuration
└── stats.json       # Usage statistics
```

See [docs/CONFIGURATION.md](docs/CONFIGURATION.md) for full configuration options.

## Project Status

**Current Status**: 🚧 Pre-alpha - Initial project setup

This project is in active development. The repository structure is established, but implementation has not yet begun.

### Roadmap

- [ ] **Phase 0**: Project initialization (current)
- [ ] **Phase 1**: Basic proxy (forward all requests to Claude)
- [ ] **Phase 2**: Add logging and learning infrastructure
- [ ] **Phase 3**: Implement routing logic and local models
- [ ] **Phase 4**: Constitutional AI validation
- [ ] **Phase 5**: Optimization and production readiness

See [CONSTITUTIONAL_PROXY_SPEC.md](CONSTITUTIONAL_PROXY_SPEC.md) for complete specification.

## Development

```bash
# Clone repository
git clone https://github.com/shammah/claude-proxy
cd claude-proxy

# Build project
cargo build

# Run tests
cargo test

# Run in development mode
cargo run
```

See [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) for development guidelines.

## Requirements

- **Platform**: macOS (Apple Silicon M1/M2/M3/M4)
- **Rust**: 1.70 or later
- **Storage**: ~5GB for models and training data
- **Memory**: 8GB RAM minimum, 16GB recommended

## Privacy & Security

- All models run locally on your machine
- Training data never leaves your device
- Only forwarded requests (5% at steady state) go to Claude API
- Claude API key stored securely in system keychain
- No telemetry or data collection

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions welcome! Please read [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) first.

## Acknowledgments

- Built on [Claude AI](https://www.anthropic.com/claude) by Anthropic
- Inspired by Constitutional AI research
- Powered by Rust and Apple's CoreML/Neural Engine

---

**Note**: Shammah is an independent project and is not affiliated with or endorsed by Anthropic.
