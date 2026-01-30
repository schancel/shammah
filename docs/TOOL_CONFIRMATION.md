# Tool Confirmation Feature

## Overview

The tool confirmation feature adds interactive prompts before executing tools in Shammah. This gives users control over what actions Claude performs on their behalf, with options to approve once, remember the approval for the session, or deny execution.

## How It Works

### Confirmation Flow

When Claude requests to use a tool in interactive mode:

1. **Generate Signature**: A unique signature is created from the tool name and context
2. **Check Cache**: If the signature is already approved this session, execute immediately
3. **Prompt User**: If not cached, display a confirmation prompt with three options:
   - **Option 1**: Approve once (execute now, ask again next time)
   - **Option 2**: Approve and remember (execute now and skip prompt for similar future calls)
   - **Option 3**: Deny (skip tool execution and send error to Claude)
4. **Cache on Remember**: If option 2 selected, add signature to session cache
5. **Execute or Deny**: Proceed based on user choice

### Tool Signatures

Each tool execution generates a context-specific signature that identifies the operation:

| Tool | Signature Format | Example |
|------|------------------|---------|
| `bash` | `{command} in {working_dir}` | `cargo test in /Users/foo/project` |
| `read` | `reading {file_path}` | `reading /path/to/file.txt` |
| `grep` | `pattern '{pattern}' in {path}` | `pattern 'fn main' in src/` |
| `glob` | `pattern {pattern}` | `pattern **/*.rs` |
| `web_fetch` | `fetching {url}` | `fetching https://docs.rs/tokio` |
| `save_and_exec` | `{command} in {working_dir}` | `cargo build in /Users/foo/project` |

## Example Prompts

### Bash Command

```
  Tool Execution Request:
  ─────────────────────────
  Tool: bash
  Command: cargo fmt
  Description: Format code with rustfmt

  Do you want to proceed?
  ❯ 1. Yes
    2. Yes, and don't ask again for cargo fmt in /Users/shammah/repos/claude-proxy
    3. No

  Choice [1-3]: _
```

### Read File

```
  Tool Execution Request:
  ─────────────────────────
  Tool: read
  File: /Users/shammah/repos/claude-proxy/src/main.rs

  Do you want to proceed?
  ❯ 1. Yes
    2. Yes, and don't ask again for reading /Users/shammah/repos/claude-proxy/src/main.rs
    3. No

  Choice [1-3]: _
```

### Web Fetch

```
  Tool Execution Request:
  ─────────────────────────
  Tool: web_fetch
  URL: https://docs.rs/tokio
  Prompt: Get the latest version number

  Do you want to proceed?
  ❯ 1. Yes
    2. Yes, and don't ask again for fetching https://docs.rs/tokio
    3. No

  Choice [1-3]: _
```

## User Interaction

### Approving Once (Option 1)

When you select option 1:
- Tool executes immediately
- No cache entry is created
- Next time Claude tries the same operation, you'll be prompted again

**Use case**: Testing or one-off operations you don't want to repeat

### Approving and Remembering (Option 2)

When you select option 2:
- Tool executes immediately
- Signature is added to session cache
- Future identical operations execute without prompting
- Cache persists only for current session (cleared on exit)

**Use case**: Repetitive operations you trust, like formatting code or reading specific files

### Denying (Option 3)

When you select option 3:
- Tool execution is skipped
- Error message sent to Claude: "Tool execution denied by user"
- Claude receives feedback and can adapt its approach
- No cache entry is created

**Use case**: Operations you don't want to perform, or when Claude is going in the wrong direction

## Session Cache Behavior

### What Gets Cached

Only signatures from "approve and remember" (option 2) are cached. Each signature includes:
- Tool name
- Context-specific parameters (command, file path, URL, etc.)
- Working directory (for tools that depend on it)

### Cache Lifetime

- **Created**: When user selects option 2
- **Persists**: For the duration of the current Shammah session
- **Cleared**: When session ends (Ctrl+C, `/quit`, or process exit)
- **Not Saved**: Cache is in-memory only, never written to disk

### Cache Matching

Signatures must match **exactly** for cache hits:

✅ **Cache Hit Examples:**
- Same command in same directory
- Same file path
- Same URL

❌ **Cache Miss Examples:**
- `cargo test` vs `cargo test --all` (different commands)
- `./src/main.rs` vs `/absolute/path/src/main.rs` (different paths)
- Running from different working directory

## Non-Interactive Mode

When Shammah runs in non-interactive mode (pipes, scripts, daemon):
- **No prompts shown**
- **All tools execute automatically**
- **No caching needed**

This ensures Shammah works seamlessly in automation contexts.

## Technical Details

### Implementation

The confirmation system is implemented in:
- `src/tools/executor.rs`: `ToolSignature`, `ToolConfirmationCache`, `generate_tool_signature()`
- `src/cli/repl.rs`: `ConfirmationResult`, `confirm_tool_execution()`, `display_tool_params()`

### Key Components

**ToolSignature**
```rust
pub struct ToolSignature {
    pub tool_name: String,
    pub context_key: String,
}
```

**ToolConfirmationCache**
```rust
pub struct ToolConfirmationCache {
    approved: HashSet<ToolSignature>,
}
```

**ConfirmationResult**
```rust
pub enum ConfirmationResult {
    ApproveOnce,
    ApproveAndRemember(ToolSignature),
    Deny,
}
```

### Integration Point

Confirmation happens in `execute_tool_loop()` before each tool execution:

1. Generate signature from tool use and working directory
2. Check if signature is pre-approved in cache
3. If not approved and interactive mode, prompt user
4. Handle result: approve, remember, or deny
5. Execute tool or return error

## Examples

### Example 1: Repetitive Command

```
User: Format the code
→ Claude requests: bash -c "cargo fmt"

  Tool Execution Request:
  Tool: bash
  Command: cargo fmt

  Choice [1-3]: 2    # Approve and remember

✓ Approved (won't ask again this session)
✓ Success

User: Format the code again
→ Claude requests: bash -c "cargo fmt"
✓ Success           # No prompt, uses cached approval
```

### Example 2: Different Commands

```
User: Run tests
→ Claude requests: bash -c "cargo test"

  Choice [1-3]: 2    # Approve and remember

✓ Approved
✓ Success

User: Run all tests
→ Claude requests: bash -c "cargo test --all"

  Choice [1-3]: 1    # Different command, new prompt needed
```

### Example 3: Denial and Recovery

```
User: Delete all temporary files
→ Claude requests: bash -c "find . -name '*.tmp' -delete"

  Choice [1-3]: 3    # Deny - too risky

✗ Denied by user

Claude: I understand you don't want to delete those files.
        Would you like me to just list them instead?
```

## Testing

Run the demo to see signature generation:

```bash
cargo run --example tool_confirmation_demo
```

Run tests:

```bash
cargo test --lib confirmation
cargo test --lib signature
cargo test --lib approval
```

## Future Enhancements

Potential improvements not currently implemented:

1. **Persistent Cache**: Save approvals to `~/.shammah/tool_approvals.json`
2. **Pattern-Based Rules**: "Always allow cargo commands"
3. **Timeout**: Auto-deny if no response in 30 seconds
4. **Batch Approval**: "Approve all remaining tools"
5. **Review Mode**: `/approvals` command to view/clear cache
6. **Config File**: `allow_tools = ["read", "glob"]` in config.toml
7. **Regex Patterns**: Approve commands matching regex
8. **Risk Levels**: Auto-approve "safe" tools, always prompt for "dangerous" ones

## Design Principles

1. **Safe by Default**: Prompt for every new operation
2. **User Control**: User decides what executes
3. **Session Scope**: Cache clears on exit (no persistent state)
4. **Context Aware**: Signatures include relevant parameters
5. **Non-Blocking**: Non-interactive mode works without prompts
6. **Transparent**: Clear indication of what will execute
7. **Recoverable**: Denial doesn't break the conversation

## Security Considerations

### What This Protects Against

- **Unintended Commands**: User sees and approves all bash commands
- **Sensitive File Access**: User controls which files can be read
- **External Requests**: User controls which URLs are fetched
- **Destructive Operations**: User can deny dangerous commands

### What This Doesn't Protect Against

- **Approved Malicious Commands**: If user approves "rm -rf /", it will run
- **Session Hijacking**: If attacker gains access during session, cached approvals are valid
- **Social Engineering**: If Claude convinces user to approve bad commands
- **Code Injection**: This doesn't prevent vulnerabilities in tool implementations

### Best Practices

1. **Read Carefully**: Always read the full command before approving
2. **Be Conservative**: Use option 1 (approve once) for unfamiliar operations
3. **Review Context**: Check working directory and file paths
4. **Ask Questions**: If unsure, deny and ask Claude to explain
5. **Start Fresh**: Restart session to clear cache if concerned
6. **Trust Gradually**: Build up cached approvals over time

## Troubleshooting

### Prompt Not Appearing

**Symptom**: Tools execute without prompts

**Causes**:
- Running in non-interactive mode (piped input)
- Tool signature already cached from earlier approval

**Solution**:
- Check if stdout is a terminal
- Restart session to clear cache

### Invalid Input Loop

**Symptom**: "Invalid choice" message repeats

**Cause**: Entering invalid input (not 1, 2, 3, y, n, yes, no)

**Solution**: Enter a valid choice (1-3)

### Ctrl+C Not Working

**Symptom**: Can't exit prompt with Ctrl+C

**Cause**: Input handler may be blocking

**Solution**:
- Press Ctrl+D (EOF) to treat as denial
- Enter "3" to deny and continue

## FAQ

**Q: Can I skip prompts for all tools?**
A: No. Each tool execution requires either a prompt or cached approval. Use option 2 to cache approvals.

**Q: Do cached approvals persist across sessions?**
A: No. Cache is cleared when session ends. This is by design for security.

**Q: Can I approve multiple tools at once?**
A: Not currently. Each tool is prompted separately. Future enhancement may add batch approval.

**Q: What if I accidentally approve something dangerous?**
A: The tool executes immediately. You can only prevent future executions by restarting the session.

**Q: Can I review what's been approved?**
A: Not currently. Future enhancement may add `/approvals` command to list cached signatures.

**Q: Does this work in daemon mode?**
A: No. Daemon mode is non-interactive, so all tools execute automatically.

## Summary

The tool confirmation feature gives you fine-grained control over Claude's actions while maintaining a smooth interactive experience. By caching approvals within a session, you can balance security with convenience.

Key takeaways:
- ✓ You control what executes
- ✓ Session-level caching reduces repetitive prompts
- ✓ Non-interactive mode unaffected
- ✓ Clear, context-specific signatures
- ✓ Three flexible approval options
