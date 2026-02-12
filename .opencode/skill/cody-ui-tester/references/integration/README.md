# Integration Guide

Patterns and examples for integrating the Cody UI Test Harness with Bun/TypeScript.

## Table of Contents

1. [Quick Start](#quick-start)
2. [TypeScript Client](#typescript-client)
3. [Testing Patterns](#testing-patterns)
4. [Vision Model Integration](#vision-model-integration)
5. [Best Practices](#best-practices)

---

## Quick Start

### Prerequisites

```bash
# Install Bun if not already installed
curl -fsSL https://bun.sh/install | bash

# Verify installation
bun --version
```

### Basic Setup

1. **Create a test script:**

```typescript
// tests/test.ts
import { CodyUITester } from "./ui_tester_client";

const tester = new CodyUITester();

await tester.start();

// Your test code here
await tester.click("New Session");
const screenshot = await tester.screenshot("result");
console.log("Screenshot:", screenshot);

await tester.stop();
```

2. **Run the test:**

```bash
bun run test.ts
```

---

## TypeScript Client

### API Reference

```typescript
export class CodyUITester {
  /**
   * Start the test harness binary.
   * Waits for harness to be ready (up to 10 seconds).
   */
  async start(): Promise<void>;

  /**
   * Stop the test harness and cleanup resources.
   */
  stop(): void;

  /**
   * Click on a UI element by its label.
   * @param target - The label/text of the element to click
   */
  async click(target: string): Promise<CommandResponse>;

  /**
   * Type text into the currently focused input.
   * @param text - Text to type (alphanumeric only)
   */
  async typeText(text: string): Promise<CommandResponse>;

  /**
   * Capture a screenshot.
   * @param name - Filename (without extension)
   * @returns Full path to the saved screenshot
   */
  async screenshot(name: string): Promise<string | null>;

  /**
   * Get all visible UI elements with their metadata.
   * @returns Array of UI elements
   */
  async getState(): Promise<UiElement[]>;

  /**
   * Wait for a specified duration.
   * @param milliseconds - Duration in milliseconds
   */
  async wait(milliseconds: number): Promise<CommandResponse>;

  /**
   * Press a specific key.
   * @param key - Key name (Enter, Escape, Tab, Backspace, Space, a-z, 0-9)
   */
  async keyPress(key: string): Promise<CommandResponse>;
}
```

### Response Types

```typescript
interface CommandResponse {
  id: string;
  status: "ok" | "error" | "timeout";
  screenshot?: string;  // Path to screenshot if captured
  error?: string;       // Error message if status is "error"
  elements?: UiElement[]; // UI elements if get_state was called
}

interface UiElement {
  label?: string;                    // Element text/label
  element_type: string;              // Button, Label, TextInput, etc.
  rect: [number, number, number, number]; // [x, y, width, height]
}
```

---

## Testing Patterns

### Pattern 1: Simple Screenshot Test

```typescript
import { CodyUITester } from "./ui_tester_client";

async function testButtonVisible() {
  const tester = new CodyUITester();
  
  try {
    await tester.start();
    
    // Get current UI state
    const elements = await tester.getState();
    
    // Check if button exists
    const hasButton = elements.some(e => 
      e.label === "New Session" && e.element_type === "Button"
    );
    
    if (!hasButton) {
      throw new Error("New Session button not found");
    }
    
    // Take screenshot for visual confirmation
    const screenshot = await tester.screenshot("button_test");
    console.log("✓ Test passed, screenshot:", screenshot);
    
  } finally {
    tester.stop();
  }
}

testButtonVisible();
```

### Pattern 2: Workflow Testing

```typescript
async function testCreateSessionWorkflow() {
  const tester = new CodyUITester();
  
  try {
    await tester.start();
    
    // Step 1: Click "New Session"
    console.log("Step 1: Clicking New Session...");
    await tester.click("New Session");
    await tester.wait(500);
    
    // Step 2: Type session name
    console.log("Step 2: Typing session name...");
    await tester.typeText("My Test Session");
    
    // Step 3: Submit
    console.log("Step 3: Submitting...");
    await tester.keyPress("Enter");
    await tester.wait(1000);
    
    // Step 4: Verify result
    console.log("Step 4: Taking screenshot...");
    const screenshot = await tester.screenshot("session_created");
    
    // Validate with vision model (pseudo-code)
    // const analysis = await analyzeScreenshot(screenshot);
    // assert(analysis.includes("Session created"));
    
    console.log("✓ Workflow test passed");
    
  } finally {
    tester.stop();
  }
}
```

### Pattern 3: Visual Regression

```typescript
import { readFileSync } from "fs";
import crypto from "crypto";

async function visualRegressionTest() {
  const tester = new CodyUITester();
  
  try {
    await tester.start();
    
    // Navigate to the page
    await tester.click("Settings");
    await tester.wait(500);
    
    // Capture screenshot
    const screenshotPath = await tester.screenshot("settings_page");
    
    if (!screenshotPath) {
      throw new Error("Failed to capture screenshot");
    }
    
    // Calculate hash of screenshot
    const screenshotData = readFileSync(screenshotPath);
    const hash = crypto.createHash("sha256").update(screenshotData).digest("hex");
    
    // Compare with baseline (stored from previous run)
    const baselineHash = "..."; // Load from storage
    
    if (hash !== baselineHash) {
      console.log("⚠️ Visual regression detected!");
      console.log("  New hash:", hash);
      console.log("  Baseline:", baselineHash);
      // Fail test or update baseline
    } else {
      console.log("✓ No visual changes detected");
    }
    
  } finally {
    tester.stop();
  }
}
```

### Pattern 4: Element Discovery

```typescript
async function discoverAndClick() {
  const tester = new CodyUITester();
  
  try {
    await tester.start();
    
    // Get all elements
    const elements = await tester.getState();
    
    // Find all buttons
    const buttons = elements.filter(e => 
      e.element_type === "Button"
    );
    
    console.log("Found buttons:");
    for (const btn of buttons) {
      console.log(`  - ${btn.label} at [${btn.rect.join(", ")}]`);
    }
    
    // Click the first button with "New" in the label
    const newButton = buttons.find(b => 
      b.label?.includes("New")
    );
    
    if (newButton?.label) {
      await tester.click(newButton.label);
      console.log(`✓ Clicked "${newButton.label}"`);
    }
    
  } finally {
    tester.stop();
  }
}
```

---

## Vision Model Integration

### Using with OpenAI GPT-4 Vision

```typescript
import OpenAI from "openai";
import { readFileSync } from "fs";

const openai = new OpenAI({ apiKey: process.env.OPENAI_API_KEY });

async function analyzeWithVision(screenshotPath: string): Promise<string> {
  const imageBuffer = readFileSync(screenshotPath);
  const base64Image = imageBuffer.toString("base64");
  
  const response = await openai.chat.completions.create({
    model: "gpt-4-vision-preview",
    messages: [
      {
        role: "user",
        content: [
          { type: "text", text: "Describe what you see in this UI screenshot." },
          {
            type: "image_url",
            image_url: {
              url: `data:image/png;base64,${base64Image}`,
            },
          },
        ],
      },
    ],
  });
  
  return response.choices[0].message.content ?? "";
}

// Usage in test
async function testWithVision() {
  const tester = new CodyUITester();
  
  try {
    await tester.start();
    await tester.click("New Session");
    
    const screenshot = await tester.screenshot("new_session");
    if (screenshot) {
      const description = await analyzeWithVision(screenshot);
      
      // Validate description contains expected text
      if (description.includes("new session form")) {
        console.log("✓ UI appears correct");
      } else {
        console.log("✗ UI validation failed");
        console.log("Description:", description);
      }
    }
  } finally {
    tester.stop();
  }
}
```

### Using with Anthropic Claude

```typescript
import Anthropic from "@anthropic-ai/sdk";

const anthropic = new Anthropic({ apiKey: process.env.ANTHROPIC_API_KEY });

async function analyzeWithClaude(screenshotPath: string): Promise<string> {
  const imageBuffer = readFileSync(screenshotPath);
  const base64Image = imageBuffer.toString("base64");
  
  const response = await anthropic.messages.create({
    model: "claude-3-opus-20240229",
    max_tokens: 1024,
    messages: [
      {
        role: "user",
        content: [
          {
            type: "image",
            source: {
              type: "base64",
              media_type: "image/png",
              data: base64Image,
            },
          },
          {
            type: "text",
            text: "What UI elements do you see? Is there a form visible?",
          },
        ],
      },
    ],
  });
  
  // Extract text content from response
  const content = response.content[0];
  return content.type === "text" ? content.text : "";
}
```

### Vision-Based Assertions

```typescript
async function assertUIState(
  tester: CodyUITester,
  expectedDescription: string
): Promise<void> {
  const screenshot = await tester.screenshot("assertion");
  if (!screenshot) throw new Error("Failed to capture screenshot");
  
  const actualDescription = await analyzeWithVision(screenshot);
  
  // Use LLM to compare
  const comparison = await openai.chat.completions.create({
    model: "gpt-4",
    messages: [
      {
        role: "system",
        content: "Compare the expected UI description with the actual. Return only 'PASS' or 'FAIL'."
      },
      {
        role: "user",
        content: `Expected: ${expectedDescription}\n\nActual: ${actualDescription}`
      }
    ]
  });
  
  const result = comparison.choices[0].message.content;
  if (result !== "PASS") {
    throw new Error(`UI assertion failed. Expected: ${expectedDescription}`);
  }
}
```

---

## Best Practices

### 1. Resource Management

Always use try/finally to ensure cleanup:

```typescript
async function safeTest() {
  const tester = new CodyUITester();
  
  try {
    await tester.start();
    // ... test code ...
  } finally {
    // Always cleanup, even if test fails
    tester.stop();
  }
}
```

### 2. Retry Logic

```typescript
async function retryCommand<T>(
  fn: () => Promise<T>,
  maxRetries = 3
): Promise<T> {
  let lastError;
  
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await fn();
    } catch (err) {
      lastError = err;
      console.log(`Attempt ${i + 1} failed, retrying...`);
      await new Promise(r => setTimeout(r, 1000 * (i + 1))); // Exponential backoff
    }
  }
  
  throw lastError;
}

// Usage
await retryCommand(() => tester.click("Submit"));
```

### 3. Parallel Testing

```typescript
async function runParallelTests() {
  const tests = [
    () => testFeatureA(),
    () => testFeatureB(),
    () => testFeatureC(),
  ];
  
  // Each test creates its own harness instance
  const results = await Promise.allSettled(tests.map(t => t()));
  
  for (const result of results) {
    if (result.status === "rejected") {
      console.error("Test failed:", result.reason);
    }
  }
}
```

### 4. Screenshot Naming Convention

```typescript
function generateScreenshotName(
  testName: string,
  step: number,
  action: string
): string {
  const timestamp = Date.now();
  return `${testName}_step${step}_${action}_${timestamp}`;
}

// Usage
await tester.screenshot(generateScreenshotName("login", 1, "initial"));
await tester.screenshot(generateScreenshotName("login", 2, "after_submit"));
```

### 5. Type-Safe Test Suites

```typescript
interface TestCase {
  name: string;
  run: (tester: CodyUITester) => Promise<void>;
}

const tests: TestCase[] = [
  {
    name: "Button visibility",
    run: async (t) => {
      const elements = await t.getState();
      const hasButton = elements.some(e => e.label === "Submit");
      if (!hasButton) throw new Error("Submit button not found");
    }
  },
  {
    name: "Screenshot capture",
    run: async (t) => {
      const path = await t.screenshot("test");
      if (!path) throw new Error("Screenshot failed");
    }
  }
];

async function runTests() {
  const tester = new CodyUITester();
  
  try {
    await tester.start();
    
    for (const test of tests) {
      try {
        console.log(`Running: ${test.name}`);
        await test.run(tester);
        console.log(`✓ ${test.name} passed`);
      } catch (err) {
        console.error(`✗ ${test.name} failed:`, err);
      }
    }
  } finally {
    tester.stop();
  }
}
```

---

## Error Handling

### Common Errors

```typescript
try {
  await tester.click("Non-existent Button");
} catch (err) {
  // Element not found error
  console.error("Element not found:", err);
}

try {
  await tester.start();
  // If cargo is not installed or not in project directory
} catch (err) {
  console.error("Failed to start harness:", err);
  console.log("Ensure you:");
  console.log("  1. Have Rust/Cargo installed");
  console.log("  2. Are in the egui-learning project directory");
}
```

### Timeout Handling

```typescript
async function withTimeout<T>(
  promise: Promise<T>,
  ms: number
): Promise<T> {
  const timeout = new Promise<never>((_, reject) => {
    setTimeout(() => reject(new Error(`Timeout after ${ms}ms`)), ms);
  });
  
  return Promise.race([promise, timeout]);
}

// Usage
try {
  await withTimeout(tester.click("Slow Button"), 5000);
} catch (err) {
  if (err.message.includes("Timeout")) {
    console.log("Command timed out - UI might be stuck");
  }
}
```

---

## Testing in CI/CD

### GitHub Actions Example

```yaml
name: UI Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Bun
        uses: oven-sh/setup-bun@v1
        with:
          bun-version: latest
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Install dependencies
        run: bun install
      
      - name: Run UI tests
        run: bun run test
        
      - name: Upload screenshots
        uses: actions/upload-artifact@v3
        if: failure()
        with:
          name: failed-screenshots
          path: /tmp/cody-screenshots/*.png
```

### Headless GPU Support

For CI environments without GPU:

```bash
# Install Mesa software renderer
sudo apt-get install mesa-utils libegl1-mesa libgl1-mesa-dri

# Run with software rendering
LIBGL_ALWAYS_SOFTWARE=1 bun run test.ts
```
