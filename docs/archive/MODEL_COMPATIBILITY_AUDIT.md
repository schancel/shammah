# Model Compatibility Audit

**Date:** 2026-02-17
**Issue:** Setup wizard allows invalid model/device combinations

## Problems Found

### 1. `is_model_available()` Always Returns True

**File:** `src/cli/setup_wizard.rs:18-22`

```rust
fn is_model_available(_family: ModelFamily, _device: BackendDevice) -> bool {
    // All model families work with all devices via ONNX Runtime
    // The device selection just chooses the execution provider (CoreML/Metal/CPU)
    true  // ← ALWAYS RETURNS TRUE!
}
```

**Impact:** Users can select any combination, even if it won't work.

### 2. Wrong Repository Format for ONNX Models

**File:** `src/models/unified_loader.rs:440-459`

Most models point to PyTorch repos instead of ONNX repos:

| Model | Current Repo | Correct Repo | Works? |
|-------|-------------|--------------|--------|
| Qwen2 | `Qwen/Qwen2.5-{size}-Instruct` | `onnx-community/Qwen2.5-{size}-Instruct` | ❌ Wrong format |
| Gemma2 | `google/gemma-2-{size}-it` | `onnx-community/gemma-2-{size}-it` (if exists) | ❌ Wrong format |
| Llama3 | `meta-llama/Llama-3.2-{size}-Instruct` | `onnx-community/Llama-3.2-{size}-Instruct` (if exists) | ❌ Wrong format |
| Mistral | `mistralai/Mistral-7B-Instruct-v0.3` | `onnx-community/Mistral-7B-Instruct-v0.3` (if exists) | ❌ Wrong format |
| Phi | `onnx-community/Phi-4-mini-instruct-ONNX` | ✓ Correct | ✅ Works |
| DeepSeek | `onnx-community/DeepSeek-R1-Distill-Qwen-1.5B-ONNX` | ✓ Correct | ✅ Works |

### 3. CoreML Mistral Points to Non-Existent Repo

**File:** `src/models/unified_loader.rs:434-437`

```rust
(ModelFamily::Mistral, BackendDevice::CoreML) => {
    // Apple's official CoreML conversion (7B)
    "apple/mistral-coreml".to_string()  // ← DOESN'T EXIST (404 errors)
}
```

**Error in logs:**
```
ERROR Failed to download required file config.json: status code 404
ERROR Failed to download required file tokenizer.json: status code 404
```

### 4. CoreML Doesn't Work (Known Issue)

**Per MEMORY.md:**
- CoreML loads but has runtime tensor mismatch errors
- Only viable for experimental testing
- Metal/CPU are the working backends

## Compatibility Matrix (Current State)

### CoreML (macOS only) - ⚠️ EXPERIMENTAL (Has runtime issues)

| Model | Small | Medium | Large | XLarge | Status |
|-------|-------|--------|-------|--------|--------|
| Qwen2 | ✓ 0.6B | ❌ | ❌ | ❌ | Limited support |
| Llama3 | ✓ 1B | ✓ 3B | ✓ 8B | ❌ | Community conversions |
| Gemma2 | ✓ 270M | ❌ | ❌ | ❌ | Limited support |
| Mistral | ❌ | ❌ | ❌ | ❌ | **Repo doesn't exist** |
| Phi | ❌ | ❌ | ❌ | ❌ | Not supported |
| DeepSeek | ❌ | ❌ | ❌ | ❌ | Not supported |

### ONNX (Metal/CPU) - ✅ RECOMMENDED

| Model | Small | Medium | Large | XLarge | Status |
|-------|-------|--------|-------|--------|--------|
| Qwen2 | ✓ 1.5B | ✓ 3B | ✓ 7B | ✓ 14B | **Repo path wrong** |
| Llama3 | ? | ? | ? | ? | **Unknown (need to verify)** |
| Gemma2 | ? | ? | ? | ? | **Unknown (need to verify)** |
| Mistral | ? 7B | ? | ? 22B | ? | **Unknown (need to verify)** |
| Phi | ✓ 3.8B | ✓ 3.8B | ✓ 14B | ✓ 14B | ✅ **Works** |
| DeepSeek | ✓ 1.5B | ✓ 1.5B | ✓ 1.5B | ✓ 1.5B | ✅ **Works** (only 1.5B exists) |

## Required Fixes

### Fix 1: Implement Proper `is_model_available()` Validation

```rust
fn is_model_available(family: ModelFamily, device: BackendDevice) -> bool {
    match (family, device) {
        // CoreML support (limited, experimental)
        #[cfg(target_os = "macos")]
        (ModelFamily::Qwen2, BackendDevice::CoreML) => true,  // Only Small size
        #[cfg(target_os = "macos")]
        (ModelFamily::Llama3, BackendDevice::CoreML) => true, // Small/Medium/Large
        #[cfg(target_os = "macos")]
        (ModelFamily::Gemma2, BackendDevice::CoreML) => true, // Only Small size
        #[cfg(target_os = "macos")]
        (ModelFamily::Mistral, BackendDevice::CoreML) => false, // BROKEN - repo doesn't exist
        #[cfg(target_os = "macos")]
        (ModelFamily::Phi, BackendDevice::CoreML) => false, // Not supported
        #[cfg(target_os = "macos")]
        (ModelFamily::DeepSeek, BackendDevice::CoreML) => false, // Not supported

        // ONNX support (Metal/CPU)
        (ModelFamily::Qwen2, _) => true,  // Need to fix repo path
        (ModelFamily::Phi, _) => true,    // Works
        (ModelFamily::DeepSeek, _) => true, // Works
        (ModelFamily::Llama3, _) => false, // Need to verify ONNX availability
        (ModelFamily::Gemma2, _) => false, // Need to verify ONNX availability
        (ModelFamily::Mistral, _) => false, // Need to verify ONNX availability

        _ => false,
    }
}
```

### Fix 2: Update Repository Paths for ONNX Models

**File:** `src/models/unified_loader.rs`

```rust
(ModelFamily::Qwen2, _) => {
    // FIX: Use onnx-community instead of Qwen
    format!("onnx-community/Qwen2.5-{}-Instruct", size_str)
}
```

### Fix 3: Remove Broken Mistral CoreML Mapping

```rust
#[cfg(target_os = "macos")]
(ModelFamily::Mistral, BackendDevice::CoreML) => {
    // REMOVED: apple/mistral-coreml doesn't exist
    anyhow::bail!(
        "Mistral CoreML models are not available.\n\n\
         Please select Metal or CPU as your device,\n\
         or choose a different model family."
    )
}
```

### Fix 4: Verify ONNX Availability for Remaining Models

Need to check if these exist:
- `onnx-community/gemma-2-*-it`
- `onnx-community/Llama-3.2-*-Instruct`
- `onnx-community/Mistral-*-Instruct-v0.3`

## Testing Plan

1. **Verify ONNX Community Repos:**
   - Check which models actually exist on onnx-community
   - Update compatibility matrix

2. **Test Each Working Combination:**
   - Qwen2 + ONNX (after repo fix)
   - Phi + ONNX
   - DeepSeek + ONNX

3. **Verify Setup Wizard Blocks Invalid Combinations:**
   - Try Mistral + CoreML → Should show error
   - Try Phi + CoreML → Should show error
   - Try unsupported combinations → Should show error

## Recommended Immediate Actions

1. ✅ Fix `is_model_available()` to block known-broken combinations
2. ✅ Fix Qwen2 repository path (`onnx-community/` prefix)
3. ✅ Remove broken Mistral CoreML mapping
4. ⏳ Verify which other models have ONNX versions
5. ⏳ Update STATUS.md with accurate compatibility info

## Long-Term Recommendations

1. **Add validation in setup wizard:**
   - Don't offer CoreML (it's broken per MEMORY.md)
   - Or clearly mark CoreML as "Experimental - May not work"

2. **Create model availability test:**
   - Script that checks if repos exist before adding to wizard
   - Automated testing of model downloads

3. **Update documentation:**
   - Clear compatibility matrix in README
   - Setup wizard should show which models work with which devices
