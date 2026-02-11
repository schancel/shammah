# Phase 4 Compilation Status

**Date**: 2026-02-10
**Current Commit**: `0329303` - "fix: major progress on Phase 4 compilation errors"
**Status**: Core infrastructure fixed, ~57 errors remaining in dependent code

---

## Overview

Phase 4 (Remove Candle Dependencies) is in progress. The core model infrastructure has been successfully migrated to ONNX-only, but dependent application code still has compilation errors.

**Error Progression**:
```
Initial (after deletions):  38 errors
After core fixes:           22 errors
After stub enhancements:     8 errors
Current:                   ~57 errors (type mismatches in dependent code)
```

---

## ✅ Completed Work

### Core Type System (100% Fixed)

**1. `src/models/common.rs`**
- ❌ Removed: `use candle_core::Device`
- ❌ Removed: `get_device()`, `get_device_with_preference()` implementations using Candle
- ✅ Added: Stub functions that return helpful errors
- ✅ Kept: `DevicePreference` enum (marked deprecated)
- ✅ Status: Compiles cleanly

**2. `src/config/backend.rs`**
- ❌ Removed: Candle Device checks in `is_available()`
- ✅ Simplified: Assumes platform support = availability
- ✅ Note: ONNX Runtime will handle actual device detection
- ✅ Status: Compiles cleanly

**3. `src/models/generator_new.rs`**
- ❌ Removed: `use candle_core::{Device, Tensor}`
- ❌ Removed: `device()` method from `TextGeneration` trait
- ❌ Removed: `LegacyGenerator` struct (depended on Candle generator)
- ✅ Updated: `GeneratorModel::new()` to error on RandomInit
- ✅ Added: Missing imports (Path, Saveable)
- ✅ Status: Compiles cleanly

**4. `src/models/unified_loader.rs`**
- ✅ Added: Missing imports (BackendDevice, ModelDownloader, TextGeneration, QwenSize)
- ✅ Updated: `load()` method returns helpful error (use load_onnx instead)
- ✅ Status: Compiles cleanly

**5. `src/models/tokenizer.rs` (recreated as stub)**
- ✅ Created: New stub module with TextTokenizer
- ✅ Methods: new(), default(), encode(), decode()
- ✅ All methods return helpful errors
- ✅ Status: Compiles cleanly

**6. `src/models/persistence.rs`**
- ❌ Removed: `candle_nn::VarMap` parameter
- ✅ Stubbed: `save_model_with_metadata()` with helpful error
- ✅ Status: Compiles cleanly

### Stub Types (100% Complete)

**Created/Enhanced in `src/models/mod.rs`:**
- `RouterModel`: new(), load(), predict()
- `ValidatorModel`: new(), load(), validate()
- `ModelEnsemble`: new(), generate(), stats()
- `EnsembleStats`: (unit struct)
- `Quality`: enum with Low/Medium/High
- `RouteDecision`: enum with Local/Remote

**Enhanced in `src/models/lora.rs`:**
- `WeightedExample`: critical(), improvement(), normal()
- `ExampleBuffer`: add(), len(), is_empty()
- `LoRATrainer`: new(), train()
- `TrainingCoordinator`: new(), add_example(), should_train(), train()
- `TrainingStats`: new()

**Enhanced in `src/models/tokenizer.rs`:**
- `TextTokenizer`: new(), default(), encode(), decode()

All stubs return clear error messages explaining they were removed in Phase 4.

### Exports (100% Fixed)

**Updated `src/models/mod.rs` exports:**
```rust
// Deprecated but exported for compatibility
#[allow(deprecated)]
pub use common::{
    device_info, get_device_with_preference, is_metal_available,
    DevicePreference, GeneratorConfig, ModelConfig, Saveable,
};

// Stub types
pub use tokenizer::TextTokenizer;
pub struct RouterModel;
pub struct ValidatorModel;
pub struct ModelEnsemble;
// ... etc
```

---

## ⚠️ Remaining Errors (~57)

### Error Categories

**1. Function Signature Mismatches (most common)**
- Code calling removed functions with wrong argument counts
- Example: `WeightedExample::critical(query, response)` called with 3 args

**2. Type Annotation Issues**
- Type inference failures where Candle types were removed
- Missing explicit type annotations

**3. Async/Await Mismatches**
- Some stubs need async versions
- Example: `generate()` expected to return Future

**4. Field Access on Stub Types**
- Code trying to access fields that don't exist on stubs
- Example: `model.device` when device field removed

**5. Iterator/Try Trait Issues**
- Code expecting `()` to be an iterator or implement Try
- Result of stub function used incorrectly

### Files with Remaining Errors

**CLI Code:**
- `src/cli/repl.rs` - Multiple errors
- `src/cli/setup_wizard.rs` - Device/model selection code
- `src/cli/repl_event/event_loop.rs` - Tool execution
- `src/cli/repl_event/tool_execution.rs` - Tool handling

**Router Code:**
- `src/router/hybrid_router.rs` - RouterModel usage
- `src/router/model_router.rs` - Model routing logic

**Training Code:**
- `src/training/batch_trainer.rs` - Training coordination
- `src/training/checkpoint.rs` - Model checkpointing

**Local Generator:**
- `src/local/generator.rs` - Local model code
- `src/local/mod.rs` - Local module integration

**Generators:**
- `src/generators/qwen.rs` - Qwen-specific code

**Tools:**
- `src/tools/implementations/*.rs` - Various tool implementations

---

## 🎯 Strategy for Remaining Fixes

### Phase A: High-Priority Files (Core Functionality)

**1. Bootstrap/Initialization**
- `src/models/bootstrap.rs` - Model loading (critical)
- Fix device preference handling
- Update to use ONNX loader

**2. CLI REPL**
- `src/cli/repl.rs` - Main user interface
- Update model initialization
- Fix device selection display

**3. Router Integration**
- `src/router/hybrid_router.rs` - Routing logic
- `src/router/model_router.rs` - Model selection
- Update to work with stubs or disable

### Phase B: Medium-Priority Files (Features)

**4. Training/Batch**
- `src/training/batch_trainer.rs`
- Disable or stub out Candle training

**5. Local Generator**
- `src/local/generator.rs`
- `src/local/mod.rs`
- Disable or update to ONNX

**6. Setup Wizard**
- `src/cli/setup_wizard.rs`
- Update device selection UI

### Phase C: Low-Priority Files (Nice-to-Have)

**7. Tool Implementations**
- Various tool files
- Can be disabled temporarily

**8. Generator Implementations**
- `src/generators/qwen.rs`
- Update or disable

---

## 📝 Fixing Guidelines

### Common Patterns

**Pattern 1: Function Argument Count Mismatch**
```rust
// Error: this function takes 2 arguments but 3 supplied
WeightedExample::critical(query, response, metadata)

// Fix: Remove extra argument
WeightedExample::critical(query, response)
```

**Pattern 2: Type Inference Failure**
```rust
// Error: type annotations needed
let result = function_call();

// Fix: Add explicit type
let result: Result<String> = function_call();
```

**Pattern 3: Async Mismatch**
```rust
// Error: `()` is not a future
let result = stub_function().await;

// Fix: Make stub async
pub async fn stub_function() -> Result<()> {
    anyhow::bail!("Removed in Phase 4")
}
```

**Pattern 4: Device Access**
```rust
// Error: no field `device` on type `GeneratorModel`
let device = model.device();

// Fix: Remove or comment out
// let device = model.device(); // Removed in Phase 4
```

---

## 🔄 Rollback Information

**Checkpoint Tag**: `pre-onnx-migration`
**Last Working Commit**: Before Phase 1

**Rollback Command**:
```bash
git reset --hard pre-onnx-migration
cargo build
```

**Current Progress Commits**:
```
0329303 - fix: major progress on Phase 4 compilation errors
ad62c0d - wip: Phase 4 - Remove Candle dependencies (partial)
75ff839 - feat: add ONNX model loading to UnifiedModelLoader (Phase 3)
0d92001 - feat: add ONNX Runtime infrastructure (Phases 1-2)
```

---

## 📊 Lines of Code Impact

**Deleted**: ~2,700 lines (Candle loaders, LoRA impl)
**Modified**: ~500 lines (core types, stubs)
**Added**: ~300 lines (ONNX infrastructure, stubs)
**Net Change**: -2,900 lines

---

## ✅ Success Criteria

**Core Infrastructure (DONE)**:
- ✅ No Candle dependencies in Cargo.toml
- ✅ Core model types compile cleanly
- ✅ Stub types provide clear errors
- ✅ ONNX infrastructure in place

**Application Code (IN PROGRESS)**:
- ⏳ All files compile without errors
- ⏳ Tests pass (or disabled appropriately)
- ⏳ Main binary compiles
- ⏳ Interactive mode works (with ONNX placeholders)

**Future Work (BLOCKED)**:
- ⏸️ Implement actual ONNX inference (placeholder exists)
- ⏸️ Test CoreML execution provider
- ⏸️ Phase 5: LoRA support via Python

---

## 🎯 Next Actions

1. **Continue systematic error fixing** (Option A)
   - Fix high-priority files first (bootstrap, repl, router)
   - Use patterns documented above
   - Commit progress incrementally

2. **Test compilation after each major fix**
   - `cargo check` to verify progress
   - Track error count reduction

3. **Document any architectural decisions**
   - If disabling features, document why
   - If changing APIs, note in commit messages

---

## 📞 Notes for Future Development

**When implementing ONNX inference:**
- Update `LoadedOnnxModel::generate()` in `src/models/loaders/onnx.rs`
- Implement session creation with execution providers
- Add autoregressive generation loop
- Handle KV cache, stop tokens, sampling

**When adding LoRA (Phase 5):**
- Create Python training scripts in `scripts/`
- Implement adapter loading in `onnx.rs`
- Update `AdapterManager` with real implementation

**When re-enabling features:**
- Training: Needs complete rewrite for ONNX/Python
- Local generator: Needs ONNX-based implementation
- Router/Validator: May need ML-based replacement or simple heuristics
