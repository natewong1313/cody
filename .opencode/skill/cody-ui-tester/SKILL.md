---
name: cody-ui-tester
description: UI testing skill for the Cody egui application using a headless test harness with screenshot capture. Enables automated UI interactions (click, type, navigate) and visual testing for Rust egui apps.
references:
  - commands
  - integration
  - troubleshooting
---

# Cody UI Test Harness Skill

Automated UI testing for the Cody egui application. Use this skill to test UI changes visually, perform automated interactions, and verify UI state.

## When to Use This Skill

- Test UI changes visually using screenshots
- Perform automated UI interactions (click, type, navigate)
- Verify UI state after code changes
- Debug UI layout issues
- Validate that UI elements render correctly
- Test user workflows end-to-end

## How to Use This Skill

### Reference File Structure

Each topic in `./references/<topic>/` contains a `README.md`:

| Topic | Purpose | When to Read |
|-------|---------|--------------|
| `commands/` | All available commands and parameters | Writing test scripts |
| `integration/` | Bun/TypeScript integration patterns | Connecting to your agent |
| `troubleshooting/` | Common issues and solutions | Debugging test failures |

## Quick Start

### Starting the Test Harness

```bash
cargo run --bin ui_tester
```

The harness:
1. Runs in headless mode (no visible window)
2. Accepts JSON commands on stdin
3. Returns JSON responses on stdout
4. Saves screenshots to `/tmp/cody-screenshots/`

### Basic Usage

```typescript
import { CodyUITester } from "./ui_tester_client";

const tester = new CodyUITester();
await tester.start();

// Click a button
await tester.click("New Session");

// Take screenshot for analysis
const path = await tester.screenshot("result");

await tester.stop();
```

## Available Commands

### click
Click element by label:
```json
{"id":"1","command":{"click":{"target":"New Session"}}}
```

### type
Type text into focused input:
```json
{"id":"2","command":{"type":{"text":"Hello"}}}
```

### screenshot
Capture screenshot:
```json
{"id":"3","command":{"screenshot":{"name":"main"}}}
```

### get_state
Query UI elements:
```json
{"id":"4","command":"get_state"}
```

### wait
Pause execution:
```json
{"id":"5","command":{"wait":{"ms":1000}}}
```

### key_press
Press a key:
```json
{"id":"6","command":{"key_press":{"key":"Enter"}}}
```

## Response Format

```json
{
  "id": "1",
  "status": "ok",
  "screenshot": "/tmp/cody-screenshots/test.png",
  "error": null,
  "elements": [...]
}
```

## Decision Trees

### "I need to test a UI change"

```
UI change to test?
├─ Visual regression check → Use screenshot comparison
├─ New feature verification → Workflow testing pattern
├─ Bug fix validation → Element discovery + assertions
└─ Layout issue debug → Get state + manual inspection
```

### "I need to interact with the UI"

```
Interaction needed?
├─ Click button/link → click command
├─ Fill form input → click then type
├─ Navigate/submit → key_press Enter
├─ Wait for load → wait command
└─ Check what's visible → get_state command
```

## Architecture

```
AI Agent / Test Script
   │
   │ JSON Commands (stdin)
   ↓
┌─────────────────┐
│   UI Tester     │  ← Rust binary with egui_kittest
│   (Headless)    │
└─────────────────┘
   │
   │ JSON + Screenshot Paths (stdout)
   ↓
/tmp/cody-screenshots/
```

## Reference Topics

- **commands** - Command reference with parameters and examples
- **integration** - Bun/TypeScript client API and patterns
- **troubleshooting** - Common issues and solutions

## Files in This Skill

- `SKILL.md` - This quick reference
- `references/commands/README.md` - Command documentation
- `references/integration/README.md` - Integration guide
- `references/troubleshooting/README.md` - Troubleshooting
