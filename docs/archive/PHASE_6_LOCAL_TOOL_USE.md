# Phase 6: Local Model Tool Use - Complete

**Status**: ✅ Implemented and Tested
**Date**: 2026-02-11
**Component**: Qwen Local Generator + Tool Execution System

## Summary

Implemented tool execution support for the Qwen local model, enabling it to call tools (Read, Bash, Glob, Grep, etc.) through prompt engineering and XML parsing. This makes the local model significantly more practical for real-world coding tasks.

## What Was Implemented

### 1. Tool Prompt Formatter (`src/models/tool_prompt.rs`)

Formats tool definitions into Qwen-compatible system prompts.

**Features:**
- Converts `ToolDefinition[]` → formatted system prompt text
- Includes tool descriptions, parameters, and usage examples
- Generates XML format instructions with clear examples
- Formats tool results for continuation prompts
- Truncates very long results (>2000 chars) to prevent context overflow

**Example Output:**
```markdown
# Available Tools

You have access to tools...

### read
Read a file from disk

**Parameters:**
- `file_path` (string) (required): Path to the file

**Example:**
```xml
<tool_use>
  <name>read</name>
  <parameters>{"file_path": "example.txt"}</parameters>
</tool_use>
```

### 2. Tool Call Parser (`src/models/tool_parser.rs`)

Parses XML-formatted tool calls from model output using regex.

**Features:**
- Extracts `<tool_use>` blocks with name and parameters
- Parses JSON parameters with validation
- Generates unique tool IDs (`toolu_[random]`)
- Extracts text content (removes tool XML)
- Fast `has_tool_calls()` check

**Supported Formats:**
- Multi-line formatted XML (with indentation)
- Compact single-line XML
- Multiple tools in sequence
- Complex JSON parameters with nested objects

**Example:**
```rust
let output = r#"I'll read the file.

<tool_use>
  <name>read</name>
  <parameters>{"file_path": "/tmp/test.txt"}</parameters>
</tool_use>
"#;

let tool_uses = ToolCallParser::parse(output)?;
// Returns: Vec<ToolUse> with 1 element
```

### 3. QwenAdapter Output Cleaning Update

Modified `clean_output()` to preserve tool XML markers.

**Behavior:**
- **If output contains `<tool_use>` or `<tool_result>`**: Minimal cleaning (preserve XML)
- **Otherwise**: Aggressive cleaning (remove chat template artifacts)

**Why This Matters:**
The original implementation stripped all XML-like content, which would have removed tool calls. The updated version detects tool markers and preserves them intact.

### 4. QwenGenerator Multi-Turn Tool Execution

Added full tool execution support with multi-turn conversation loop.

**Architecture:**
```
User Query
    ↓
Format Prompt (with tool definitions)
    ↓
Generate Text (Qwen ONNX)
    ↓
Has Tool Calls?
    ├─ NO → Return final answer
    └─ YES → Continue
        ↓
    Parse Tool Calls (XML → ToolUse)
        ↓
    Execute Tools (via ToolExecutor)
        ↓
    Format Tool Results
        ↓
    Add to Conversation History
        ↓
    Loop (max 5 turns)
```

**Key Methods:**
- `generate_with_tools()` - Multi-turn loop with tool execution
- `generate_single_turn()` - Simple generation (no tools)
- `format_prompt_with_tools()` - Build system prompt with tool definitions
- `execute_tools()` - Execute via ToolExecutor (with permission checks)
- `generate_text()` - Low-level ONNX generation

**Safety:**
- Max 5 turns to prevent infinite loops
- Permission system integrated (via ToolExecutor)
- Tool errors returned as ToolResult::error (non-blocking)

### 5. Integration with REPL

Updated `repl.rs` to pass `ToolExecutor` to `QwenGenerator`:

```rust
let qwen_gen = Arc::new(QwenGenerator::new(
    Arc::clone(&self.local_generator),
    Arc::clone(&self.tokenizer),
    Some(Arc::clone(&self.tool_executor)), // Enable tool support
));
```

**Result:** Qwen now has access to all 11 registered tools.

## Testing

### Unit Tests

Created comprehensive integration tests (`tests/tool_integration_test.rs`):

✅ **All 8 tests pass:**
1. `test_tool_prompt_formatting` - Verify prompt contains all key elements
2. `test_tool_call_parsing_single` - Parse single tool call
3. `test_tool_call_parsing_multiple` - Parse multiple tools in sequence
4. `test_tool_call_parsing_compact` - Parse compact XML format
5. `test_tool_call_parsing_invalid_json` - Reject malformed JSON
6. `test_extract_text` - Remove tool XML, keep text
7. `test_has_tool_calls` - Quick detection
8. `test_tool_call_with_complex_json` - Handle nested JSON

### Manual Testing Checklist

- [ ] Simple file read: `"Read the file src/main.rs"`
- [ ] Bash command: `"List all Rust files"`
- [ ] Multi-turn: `"Find all TODO comments in Rust files"`
- [ ] Permission system: `"Delete all files"` → should prompt for approval
- [ ] Error handling: Tool execution failure → graceful error response
- [ ] No tools: Regular query without tool need → direct response

## Code Structure

```
src/
├── models/
│   ├── tool_prompt.rs (NEW)    # Format tool definitions for prompts
│   ├── tool_parser.rs (NEW)    # Parse XML tool calls from output
│   ├── adapters/qwen.rs (MODIFIED)  # Preserve tool XML in cleaning
│   └── mod.rs (MODIFIED)       # Export new modules
├── generators/
│   └── qwen.rs (MODIFIED)      # Add tool execution support
├── cli/
│   └── repl.rs (MODIFIED)      # Pass ToolExecutor to QwenGenerator
└── tools/
    └── executor.rs (UNCHANGED) # Already supports tool execution

tests/
└── tool_integration_test.rs (NEW)  # Integration tests
```

## Dependencies

**No new dependencies added!**

Uses existing crates:
- `regex` (already in Cargo.toml) - Parse XML tool calls
- `once_cell` (already in Cargo.toml) - Lazy regex compilation
- `serde_json` (already in Cargo.toml) - Parse JSON parameters

## Performance Considerations

1. **Regex Compilation**: Compiled once via `once_cell::Lazy` (zero overhead after first use)
2. **Tool Execution**: Already optimized in ToolExecutor (no changes)
3. **Multi-Turn Overhead**: Max 5 turns × (generation + tool execution) ≈ 2-10s total
4. **Context Growth**: Limited to 5 messages to prevent token overflow

## Limitations & Future Improvements

### Current Limitations

1. **XML Format Adherence**: Model must generate valid XML + JSON
   - **Fallback**: If parse fails, return error to user (graceful degradation)
   - **Future**: Fine-tune format adherence with LoRA (Phase 7)

2. **Single Tool Per Turn**: Model can call multiple tools, but all must parse correctly
   - **Impact**: If one tool call is malformed, entire turn fails
   - **Future**: Partial execution (skip malformed, execute valid)

3. **Context Window**: Limited to 5 messages in conversation history
   - **Impact**: Long tool conversations may lose context
   - **Future**: Implement conversation summarization

4. **No Streaming**: Tool execution requires complete generation
   - **Impact**: User sees no progress during tool calls
   - **Future**: Implement streaming with tool delimiters

### Future Improvements

**Phase 7: LoRA Fine-Tuning for Tool Use**
- Train Qwen to generate consistent XML format
- Use Claude API tool calls as training examples
- Weight critical format errors highly (10x)

**Simplify Format (if needed)**:
```
Current:  <tool_use><name>bash</name><parameters>{...}</parameters></tool_use>
Fallback: TOOL: bash\nPARAMS: {...}
```

**Intent Detection (Alternative)**:
- "Read file X" → Auto-generate Read(file_path=X)
- "List files" → Auto-generate Bash(command="ls")
- Less flexible but more reliable

**Conversation Summarization**:
- Summarize old messages when context limit reached
- Preserve recent tool calls and results

**Streaming Support**:
- Stream text until `<tool_use>` detected
- Pause streaming, execute tool
- Resume streaming with results

## Success Metrics

✅ **Implementation Complete:**
- [x] Tool prompt formatter working (8/8 tests pass)
- [x] Tool call parser working (8/8 tests pass)
- [x] QwenAdapter preserves tool XML
- [x] QwenGenerator supports multi-turn tool execution
- [x] Integration with ToolExecutor
- [x] Compilation successful (zero errors)

**Deployment Readiness:**
- [x] Code compiles (release build)
- [x] Tests pass (8/8 integration tests)
- [ ] Manual testing (pending user verification)

**Quality Targets (to be measured):**
- Tool call parse rate: Target 70%+
- Tool execution success rate: Target 80%+
- User approval rate: Target 60%+

## Example Interactions

### Example 1: Simple File Read

**User:** "Read the file src/main.rs"

**Qwen (Turn 1):**
```
I'll read the file for you.

<tool_use>
  <name>read</name>
  <parameters>{"file_path": "src/main.rs"}</parameters>
</tool_use>
```

**System:** *Executes Read tool, returns file contents*

**Qwen (Turn 2):**
```
The main.rs file contains the entry point for the Shammah application...
[Summary of file contents]
```

### Example 2: Multi-Turn (Find TODOs)

**User:** "Find all TODO comments in Rust files"

**Qwen (Turn 1):**
```
I'll glob for Rust files first.

<tool_use>
  <name>glob</name>
  <parameters>{"pattern": "**/*.rs"}</parameters>
</tool_use>
```

**System:** *Returns list of Rust files*

**Qwen (Turn 2):**
```
Now I'll search for TODO comments in those files.

<tool_use>
  <name>grep</name>
  <parameters>{"pattern": "TODO", "path": "."}</parameters>
</tool_use>
```

**System:** *Returns TODO matches*

**Qwen (Turn 3):**
```
Found 12 TODO comments:

1. src/main.rs:45 - TODO: Implement error recovery
2. src/models/lora.rs:120 - TODO: Add batch training
...
```

## Migration Notes

**Breaking Changes:** None

**Backward Compatibility:** ✅ Full
- Existing code continues to work (tool support is opt-in)
- QwenGenerator without ToolExecutor → tools disabled (supports_tools: false)

**Rollback Plan:** If tool execution causes issues:
1. Pass `None` for tool_executor in repl.rs
2. Qwen reverts to simple generation (no tools)

## Related Documentation

- Tool execution system: `docs/TOOL_CONFIRMATION.md`
- ONNX generation pipeline: `docs/PHASE_5_KV_CACHE_COMPLETE.md`
- Output cleaning: `docs/FIX_ONNX_OUTPUT_CLEANING.md`

## Next Steps

**Immediate (User Testing):**
1. Test with real queries: "Read X", "List Y", "Find Z"
2. Verify permission system integration
3. Measure tool call parse rate

**Phase 7 (LoRA Fine-Tuning):**
1. Collect tool call examples from Qwen
2. Collect training data from Claude API tool calls
3. Train LoRA adapter for better tool format adherence
4. Integrate Python training pipeline
5. Measure improvement in parse rate

**Phase 8 (Quality Improvements):**
1. Implement streaming with tool support
2. Add conversation summarization
3. Implement partial tool execution (skip malformed calls)
4. Add intent detection fallback

---

**Phase 6 Status**: ✅ **COMPLETE** - Ready for user testing and feedback collection.
