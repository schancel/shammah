# CLAUDE.md - AI Assistant Context

This document provides context for AI assistants (like Claude Code) working on the Shammah project.

## Project Context

**Project Name**: Shammah (שָׁמָה - "watchman/guardian")
**Purpose**: Local-first Constitutional AI proxy
**Core Innovation**: Learns to handle 95% of requests locally while maintaining Claude API compatibility

### The Problem

Users of Claude API face:
- High API costs for repetitive or simple queries
- Privacy concerns sending all data to cloud
- Network latency for every request
- Inability to work offline
- No learning/improvement over time

### The Solution

Shammah acts as an intelligent proxy that:
1. Initially forwards 100% of requests to Claude API
2. Learns from every Claude response (patterns, reasoning, style)
3. Gradually handles more requests locally using trained models
4. Reaches steady state of ~5% forwarding (only novel/complex queries)
5. Reduces costs by 76% while maintaining response quality

### Key Metrics

- **Target**: 95% local processing, 5% API forwarding (steady state)
- **Timeline**: ~6 months from 100% → 5% forwarding
- **Cost Reduction**: 76% (accounting for training costs)
- **Quality**: Maintain Claude-level responses through Constitutional AI

## Architecture Overview

### High-Level Design

```
User Request
    ↓
Router (decides: local or forward?)
    ↓
    ├─→ Local Path (95%)
    │   ├─→ Small model: classify intent
    │   ├─→ Medium model: generate response
    │   └─→ Constitutional validator: ensure safety/quality
    │
    └─→ Forward Path (5%)
        ├─→ Send to Claude API
        ├─→ Return response to user
        └─→ Log for training (learn from this)
```

### Core Components

1. **Router** (`src/router/`)
   - Decides whether to handle locally or forward
   - Uses confidence scoring and complexity heuristics
   - Tracks accuracy over time

2. **Claude Client** (`src/claude/`)
   - HTTP client for Claude API
   - Handles authentication and streaming
   - Logs requests/responses for training

3. **Local Ensemble** (`src/models/`)
   - Small models: classification, intent detection
   - Medium models: response generation
   - Constitutional validator: safety checking

4. **Learning Engine** (`src/learning/`)
   - Processes Claude responses into training data
   - Retrains models periodically
   - Tracks performance metrics

5. **Configuration** (`src/config/`)
   - Reads `~/.claude/settings.json` (Claude Code integration)
   - Manages `~/.claude-proxy/` storage
   - Environment variables and CLI args

### Technology Stack

- **Language**: Rust (memory safety, performance, Apple Silicon optimization)
- **ML Framework**: CoreML (native Apple Neural Engine support)
- **Models**:
  - Small: ~1-3B parameter models (classification)
  - Medium: ~7-13B parameter models (generation)
- **API**: Compatible with Claude API format
- **Storage**: Local filesystem (`~/.claude-proxy/`)
- **Async**: Tokio runtime
- **HTTP**: Reqwest client
- **CLI**: Clap for argument parsing

## Key Design Decisions

### 1. Claude Code Compatibility

**Decision**: Use `~/.claude/settings.json` for configuration
**Rationale**: Seamless integration with Claude Code CLI tool
**Implication**: Must respect Claude Code's config format and behavior

### 2. Storage Location

**Decision**: Store everything in `~/.claude-proxy/`
**Rationale**:
- Clear separation from Claude Code
- User can easily find/delete data
- Standard Unix convention for user data

**Structure**:
```
~/.claude-proxy/
├── models/
│   ├── classifier.mlmodel
│   ├── generator-7b.mlmodel
│   └── constitutional.mlmodel
├── training/
│   ├── requests.jsonl
│   └── responses.jsonl
├── config.toml
└── stats.json
```

### 3. Command Name

**Decision**: Use `shammah` as the binary name
**Rationale**:
- Distinct from `claude` command
- Memorable and meaningful (Hebrew "watchman")
- Easy to type

### 4. Three Operating Modes

**Interactive REPL**:
```bash
shammah
> How do I use lifetimes in Rust?
```

**Daemon Mode** (background service):
```bash
shammah daemon
# Runs HTTP server on localhost:8000
# Claude Code connects via proxy settings
```

**Single Query**:
```bash
shammah query "What is the time complexity of quicksort?"
```

### 5. Learning Strategy

**Decision**: Continuous learning from all forwarded requests
**Rationale**: Every API call is a training opportunity

**Process**:
1. Forward request to Claude
2. Receive response
3. Log (request, response, metadata) to training set
4. Periodically retrain models
5. Update router confidence thresholds

### 6. Constitutional AI

**Decision**: Always validate local responses with constitutional principles
**Rationale**: Maintain safety and quality even as we reduce API usage

**Principles** (from CONSTITUTIONAL_PROXY_SPEC.md):
- Helpful: Response must address the query
- Harmless: No harmful, illegal, or unethical content
- Honest: Acknowledge uncertainty, don't make things up
- Consistent: Style should match Claude's tone

## Development Guidelines

### Code Style

- **Formatting**: Always use `cargo fmt` before committing
- **Linting**: Run `cargo clippy` and address warnings
- **Documentation**: Doc comments for all public items
- **Error Messages**: User-friendly, actionable error messages

### Error Handling

- Use `anyhow::Result` for application code
- Use `thiserror` for library-style errors with custom types
- Always provide context with `.context()` or `.with_context()`
- Never use `.unwrap()` or `.expect()` in production code paths

Example:
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

- **Unit tests**: Test individual functions in module files
- **Integration tests**: Test cross-module behavior in `tests/`
- **Property tests**: Use `proptest` for complex logic
- **Mocking**: Use `mockito` for external API calls

Minimum coverage: 80% for new code

### Logging

Use `tracing` for structured logging:

```rust
use tracing::{debug, info, warn, error, instrument};

#[instrument]
async fn forward_request(req: Request) -> Result<Response> {
    info!("Forwarding request to Claude API");
    debug!(?req, "Request details");

    let response = claude_client.send(req).await
        .context("Failed to forward request")?;

    info!(status = %response.status, "Received response");
    Ok(response)
}
```

### Commit Messages

Follow conventional commits:

```
feat: add routing logic for local vs forwarded requests
fix: handle timeout errors in Claude API client
docs: update architecture documentation
test: add integration tests for learning engine
refactor: simplify configuration loading
```

### No Code in CLAUDE.md

**IMPORTANT**: This file (CLAUDE.md) should contain context, architecture, and guidelines, but NOT implementation code. Code belongs in `src/`, `tests/`, or `examples/`.

This document is for helping AI assistants understand the project - it's a "map" not the "territory."

## Current Project Phase

**Phase**: Pre-implementation (Phase 0)
**Status**: Repository initialized, no functionality yet
**Next Steps**:
1. Implement basic proxy that forwards all requests
2. Add logging infrastructure
3. Build Claude API client

### What's Done

- ✅ Repository structure created
- ✅ Cargo.toml with dependencies
- ✅ Documentation files (CLAUDE.md, README.md, ARCHITECTURE.md, etc.)
- ✅ Placeholder source files
- ✅ License and .gitignore

### What's NOT Done Yet

- ❌ No actual functionality
- ❌ No API client implementation
- ❌ No routing logic
- ❌ No models
- ❌ No learning engine
- ❌ No tests

## Working with This Project

### For AI Assistants (Claude Code, etc.)

When working on this project:

1. **Read the spec first**: `CONSTITUTIONAL_PROXY_SPEC.md` is the authoritative design document
2. **Check current phase**: Don't implement Phase 3 features when we're in Phase 1
3. **Follow Rust conventions**: Prefer standard patterns over clever tricks
4. **Think about Apple Silicon**: This targets M1/M2/M3/M4 Macs specifically
5. **Test as you go**: Write tests alongside implementation code
6. **Document trade-offs**: When you make a design choice, explain why in code comments

### Common Tasks

**Adding a new module**:
1. Create file in `src/` (e.g., `src/metrics.rs`)
2. Add `pub mod metrics;` to `src/lib.rs`
3. Implement with proper error handling and tests
4. Add documentation to relevant `docs/` files

**Implementing a feature**:
1. Read the spec section for that feature
2. Design the API (functions, structs, traits)
3. Write tests first (TDD)
4. Implement the feature
5. Run `cargo test`, `cargo clippy`, `cargo fmt`
6. Update documentation

**Debugging an issue**:
1. Check logs (structured with `tracing`)
2. Add more instrumentation if needed
3. Write a failing test that reproduces the issue
4. Fix the issue
5. Verify test passes

## Reference Documents

- **CONSTITUTIONAL_PROXY_SPEC.md**: Complete technical specification (authoritative)
- **README.md**: User-facing documentation
- **docs/ARCHITECTURE.md**: Detailed architecture breakdown
- **docs/CONFIGURATION.md**: Configuration options and setup
- **docs/DEVELOPMENT.md**: Development workflow and guidelines

## Questions?

If you're unsure about something:

1. Check `CONSTITUTIONAL_PROXY_SPEC.md` first
2. Check this file (CLAUDE.md) for context
3. Look at existing code for patterns
4. Ask the user if still unclear

## Key Principles

1. **Local-first**: Prioritize privacy and performance
2. **Gradual improvement**: Learn continuously, don't expect perfection
3. **Constitutional AI**: Maintain safety and quality at all times
4. **User experience**: Fast, reliable, transparent
5. **Rust best practices**: Safe, idiomatic, performant code

---

This document should evolve as the project grows. Keep it updated with new design decisions and context that helps AI assistants work effectively on Shammah.
