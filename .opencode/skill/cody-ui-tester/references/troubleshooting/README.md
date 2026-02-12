# Troubleshooting

Common issues and solutions when using the Cody UI Test Harness.

## Table of Contents

1. [Compilation Issues](#compilation-issues)
2. [Runtime Errors](#runtime-errors)
3. [Command Failures](#command-failures)
4. [Screenshot Problems](#screenshot-problems)
5. [Integration Issues](#integration-issues)

---

## Compilation Issues

### Error: "no method named `click_at` found"

**Problem:** The `click_at` method was removed from the API.

**Solution:** Use `click` with a target label instead:
```json
{"click": {"target": "Button Label"}}
```

### Error: "feature `snapshot` is not enabled"

**Problem:** Screenshot support requires specific features.

**Solution:** Check your Cargo.toml:
```toml
[dependencies]
egui_kittest = { version = "0.33.3", features = ["snapshot", "wgpu"] }
```

Then rebuild:
```bash
cargo clean
cargo build --bin ui_tester
```

### Error: "wgpu not supported on this platform"

**Problem:** WGPU requires GPU support.

**Solutions:**
1. Run on a machine with GPU support
2. Use software rendering (may require additional setup)
3. For CI/CD, use a headless GPU like `swiftshader`

---

## Runtime Errors

### Error: "Missing snapshot" / "Run UPDATE_SNAPSHOTS=1"

**Problem:** The snapshot feature expects baseline images.

**Solution for testing:** This is expected behavior. The screenshot is still saved to `/tmp/cody-screenshots/`. To suppress the error:

```bash
# Option 1: Create an empty test structure
mkdir -p tests/snapshots
touch tests/snapshots/.gitkeep

# Option 2: Ignore the error - screenshots are still saved
# The error is just complaining about snapshot comparison, not the screenshot itself
```

### Error: "radv is not a conformant Vulkan implementation"

**Problem:** AMD RADV Vulkan driver warning (harmless).

**Solution:** This is just a warning. The harness will still work. To suppress:
```bash
RADV_PERFTEST=aco cargo run --bin ui_tester
```

### Error: "thread panicked at ... exceeded max_steps"

**Problem:** UI keeps repainting (infinite loop or animation).

**Solution:** Use `run_steps` instead of `run`:
```rust
// In the harness code, modify:
harness.run_steps(10);  // Run exactly 10 steps
```

Or increase max_steps in the harness builder.

---

## Command Failures

### "Element not found"

**Problem:** The target element doesn't exist or isn't visible.

**Solutions:**

1. **Check the label spelling:**
```bash
# First, get the actual labels
echo '{"id":"1","command":"get_state"}' | cargo run --bin ui_tester
```

2. **Wait for element to appear:**
```json
{"id":"1","command":{"wait":{"ms":500}}}
{"id":"2","command":{"click":{"target":"Button"}}}
```

3. **Check if element is visible:**
```json
{"id":"1","command":"get_state"}
# Check response for the element and its rect
```

### "Cannot convert character to key"

**Problem:** The `type` command only supports alphanumeric characters.

**Solution:** Only type supported characters:
- Letters: a-z, A-Z
- Numbers: 0-9
- Space

For special characters, use `key_press`:
```json
{"key_press": {"key": "Enter"}}
```

### Command times out

**Problem:** Command exceeded the 10-second timeout.

**Solutions:**

1. **For long operations, break them up:**
```json
{"screenshot": {"name": "step1"}}
{"wait": {"ms": 5000}}
{"screenshot": {"name": "step2"}}
```

2. **Check if the harness is stuck:**
Look at stderr output - if you see repeating messages, the UI might be in an infinite loop.

---

## Screenshot Problems

### Screenshots are black/blank

**Problem:** The UI isn't rendering properly in headless mode.

**Solutions:**

1. **Wait for initialization:**
```json
{"wait": {"ms": 1000}}
{"screenshot": {"name": "test"}}
```

2. **Check GPU support:**
Screenshots require GPU rendering. Ensure:
- GPU drivers are installed
- Not running in a container without GPU access

3. **For CI environments:**
Use a virtual display:
```bash
# Install xvfb
sudo apt-get install xvfb

# Run with virtual display
xvfb-run cargo run --bin ui_tester
```

### Screenshots not saving

**Problem:** Permission denied or wrong directory.

**Solutions:**

1. **Check directory exists:**
```bash
ls -la /tmp/cody-screenshots/
```

2. **Create directory manually:**
```bash
mkdir -p /tmp/cody-screenshots
chmod 755 /tmp/cody-screenshots
```

3. **Check disk space:**
```bash
df -h /tmp
```

### Wrong screenshot dimensions

**Problem:** Screenshot size doesn't match expectations.

**Solution:** The harness uses a default size. To control this, you'd need to modify the harness builder in the source code:
```rust
let harness = Harness::builder()
    .with_size([800.0, 600.0])
    .build_ui(ui_fn);
```

---

## Integration Issues

### Bun client can't find binary

**Problem:** The `cargo run` command isn't found.

**Solutions:**

1. **Ensure Rust/Cargo is installed:**
```bash
cargo --version
```

2. **Run from project directory:**
```typescript
process.chdir('/path/to/egui-learning');
await tester.start();
```

3. **Pre-build the binary:**
```bash
cargo build --bin ui_tester --release
# Then modify client to use:
["./target/release/ui_tester"]
```

### Commands not being received

**Problem:** The harness isn't reading stdin.

**Solutions:**

1. **Ensure newline is sent:**
```typescript
proc.stdin!.write(JSON.stringify(cmd) + "\n");
```

2. **Check harness is running:**
```typescript
if (proc.exitCode !== null) {
  console.log("Harness crashed!");
}
```

3. **Buffering issues:**
Use line buffering when spawning.

### JSON parse errors

**Problem:** Commands aren't valid JSON.

**Solution:** Validate your JSON:
```typescript
try {
  JSON.parse(jsonCmd);
} catch (e) {
  console.error("Invalid JSON:", e);
}
```

Common mistakes:
- Missing quotes around keys
- Trailing commas
- Using single quotes instead of double
- Unescaped special characters

### Slow response times

**Problem:** Commands take a long time to respond.

**Solutions:**

1. **Build in release mode:**
```bash
cargo build --bin ui_tester --release
./target/release/ui_tester
```

2. **Reduce screenshot resolution:**
Modify harness to use smaller default size (see above).

3. **Use WGPU in performance mode:**
```bash
WGPU_BACKEND=dx12 cargo run --bin ui_tester  # Windows
WGPU_BACKEND=vulkan cargo run --bin ui_tester  # Linux
```

---

## Debug Mode

Enable debug logging to see what's happening:

```bash
RUST_LOG=debug cargo run --bin ui_tester
```

This will show:
- All commands received
- UI rendering steps
- Element queries
- Screenshot operations

---

## Getting Help

If you encounter an issue not listed here:

1. **Check the logs:**
   - stdout: JSON responses
   - stderr: Debug/error messages

2. **Run with backtrace:**
```bash
RUST_BACKTRACE=1 cargo run --bin ui_tester
```

3. **Test individual components:**
```bash
# Test just the screenshot feature
echo '{"id":"1","command":{"screenshot":{"name":"debug"}}}' | \
  cargo run --bin ui_tester 2>&1
```

4. **Check versions:**
```bash
cargo --version
rustc --version
```

5. **Clean build:**
```bash
cargo clean
cargo build --bin ui_tester
```
