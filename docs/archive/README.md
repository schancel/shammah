# Archived Documentation

**Created:** 2026-02-11
**Reason:** Documentation cleanup and organization

This directory contains documentation for completed development phases. These documents describe work that has been successfully implemented and merged into the main codebase.

## Why Archive?

As the project progresses, phase documentation accumulates. While valuable for historical context, keeping all phase docs in the main `docs/` folder creates clutter and makes it harder to find current, actionable documentation.

Archived docs are:
- ✅ **Complete** - The work described has been fully implemented
- ✅ **Merged** - Changes are in the main codebase
- ✅ **Tested** - Functionality has been verified
- ✅ **Historical** - Useful for understanding past decisions

## Archived Documents

### ONNX Migration and Integration

- **ONNX_MIGRATION_STATUS.md** - Migration from Candle to ONNX Runtime
  - Completed: Full ONNX Runtime integration
  - Result: Working local model generation with KV cache

- **PHASE_4_COMPILATION_STATUS.md** - ONNX compilation fixes
  - Completed: Project compiles with ONNX Runtime
  - Result: No compilation errors

- **PHASE_5_ONNX_INFERENCE_PLAN.md** - Plan for ONNX inference
  - Completed: Inference implementation
  - Result: Autoregressive generation working

- **PHASE_5_KV_CACHE_COMPLETE.md** - KV cache implementation
  - Completed: Full KV cache support for 28 layers
  - Result: Efficient multi-token generation

### Local Model Tool Use

- **PHASE_6_LOCAL_TOOL_USE.md** - Tool execution for local models
  - Completed: XML + JSON tool format, parser, multi-turn execution
  - Result: Local models can call Read, Bash, Grep, etc.

### LoRA Training

- **PHASE_7_LORA_TRAINING.md** - LoRA fine-tuning pipeline
  - Completed: Python training script, weighted examples, JSONL queue
  - Result: Training infrastructure ready (Python deps pending install)

### Daemon Architecture

- **PHASE_8_DAEMON_ARCHITECTURE_PROGRESS.md** - Daemon mode development
  - Completed: Initial daemon architecture
  - Result: Auto-spawning daemon with PID management

- **PHASE_8_DAEMON_ONLY_CLI.md** - Daemon-only CLI design
  - Completed: Client/daemon split
  - Result: CLI communicates via daemon

### Tool Pass-Through

- **TOOL_PASS_THROUGH_IMPLEMENTATION.md** - Initial tool pass-through work
  - Completed: OpenAI format conversion, bidirectional messages
  - Result: Foundation for client-side tool execution

- **TOOL_PASS_THROUGH_COMPLETE.md** - Complete tool pass-through
  - Completed: Full client-side tool execution in daemon mode
  - Result: Tools execute with proper context and permissions

### Commands and Features

- **LOCAL_COMMAND_IMPLEMENTATION.md** - `/local` command
  - Completed: Force local model generation command
  - Result: Users can explicitly request local model

## Current Documentation

For up-to-date information, see:

- **STATUS.md** - Current project status and TODO list
- **docs/ROADMAP.md** - Detailed future work planning
- **docs/ARCHITECTURE.md** - System architecture
- **docs/DAEMON_MODE.md** - Daemon architecture
- **docs/TOOL_CONFIRMATION.md** - Tool permission system
- **docs/TUI_ARCHITECTURE.md** - Terminal UI rendering

## Using Archived Docs

These documents are still valuable for:

1. **Understanding design decisions** - Why certain approaches were chosen
2. **Learning implementation details** - How features were built
3. **Troubleshooting** - Understanding how things work under the hood
4. **Contributing** - Learning the project's development history

When working on related features, archived docs can provide useful context and prevent repeating past mistakes.

---

**Note:** These documents are snapshots from specific points in development. For the most current information, always refer to the main documentation and source code.
