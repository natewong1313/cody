#!/usr/bin/env bun
/**
 * Cody UI Test Harness Client (Bun/TypeScript)
 * 
 * Usage:
 *   bun run ui_tester_client.ts
 *   bun run ui_tester_client.ts --example
 *   bun run ui_tester_client.ts --interactive
 */

import { spawn } from "child_process";
import { createInterface } from "readline";

interface CommandRequest {
  id: string;
  command: any;
}

interface CommandResponse {
  id: string;
  status: "ok" | "error" | "timeout";
  screenshot?: string;
  error?: string;
  elements?: UiElement[];
}

interface UiElement {
  label?: string;
  element_type: string;
  rect: [number, number, number, number];
}

export class CodyUITester {
  private proc: ReturnType<typeof spawn> | null = null;
  private commandId = 0;
  private rl: ReturnType<typeof createInterface> | null = null;

  async start(): Promise<void> {
    console.log("Starting Cody UI Test Harness...");
    
    this.proc = spawn("cargo", ["run", "--bin", "ui_tester", "--quiet"], {
      stdio: ["pipe", "pipe", "pipe"],
    });

    // Wait for startup message on stderr
    await new Promise<void>((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error("Timeout waiting for harness to start"));
      }, 10000);

      this.proc!.stderr!.on("data", (data: Buffer) => {
        const text = data.toString();
        if (text.includes("Accepting JSON commands")) {
          clearTimeout(timeout);
          resolve();
        }
      });

      this.proc!.on("error", (err) => {
        clearTimeout(timeout);
        reject(err);
      });
    });

    // Setup readline for stdout
    this.rl = createInterface({
      input: this.proc.stdout!,
      crlfDelay: Infinity,
    });

    console.log("Test harness started!");
  }

  stop(): void {
    if (this.proc) {
      this.proc.stdin?.end();
      this.proc.kill();
      this.proc = null;
    }
    if (this.rl) {
      this.rl.close();
      this.rl = null;
    }
    console.log("Test harness stopped.");
  }

  private async sendCommand(command: any): Promise<CommandResponse> {
    if (!this.proc || !this.rl) {
      throw new Error("Test harness not started");
    }

    this.commandId++;
    const cmd: CommandRequest = {
      id: this.commandId.toString(),
      command,
    };

    const jsonCmd = JSON.stringify(cmd);
    console.log(`\n→ Sending: ${jsonCmd}`);

    // Send command
    this.proc.stdin!.write(jsonCmd + "\n");

    // Wait for response
    return new Promise((resolve) => {
      this.rl!.once("line", (line) => {
        console.log(`← Received: ${line.trim()}`);
        resolve(JSON.parse(line));
      });
    });
  }

  async click(target: string): Promise<CommandResponse> {
    return this.sendCommand({ click: { target } });
  }

  async typeText(text: string): Promise<CommandResponse> {
    return this.sendCommand({ type: { text } });
  }

  async screenshot(name: string): Promise<string | null> {
    const result = await this.sendCommand({ screenshot: { name } });
    if (result.status === "ok" && result.screenshot) {
      return result.screenshot;
    }
    return null;
  }

  async getState(): Promise<UiElement[]> {
    const result = await this.sendCommand("get_state");
    if (result.status === "ok" && result.elements) {
      return result.elements;
    }
    return [];
  }

  async wait(milliseconds: number): Promise<CommandResponse> {
    return this.sendCommand({ wait: { ms: milliseconds } });
  }

  async keyPress(key: string): Promise<CommandResponse> {
    return this.sendCommand({ key_press: { key } });
  }
}

// Example workflow
async function exampleWorkflow(): Promise<void> {
  const tester = new CodyUITester();

  try {
    await tester.start();

    // Step 1: Get initial state
    console.log("\n=== Step 1: Get Initial UI State ===");
    const elements = await tester.getState();
    console.log(`Found ${elements.length} UI elements:`);
    for (const elem of elements.slice(0, 5)) {
      console.log(`  - ${elem.element_type}: ${elem.label ?? "[no label]"}`);
    }

    // Step 2: Take initial screenshot
    console.log("\n=== Step 2: Take Initial Screenshot ===");
    const screenshotPath = await tester.screenshot("initial_state");
    if (screenshotPath) {
      console.log(`Screenshot saved: ${screenshotPath}`);
      console.log("  (AI would analyze this screenshot here)");
    }

    // Step 3: Click on "New Session" button
    console.log("\n=== Step 3: Click 'New Session' Button ===");
    const result = await tester.click("New Session");
    if (result.status === "ok") {
      console.log("✓ Click successful");
    } else {
      console.log(`✗ Click failed: ${result.error}`);
    }

    // Step 4: Wait a bit for UI to update
    console.log("\n=== Step 4: Wait for UI Update ===");
    await tester.wait(500);

    // Step 5: Take screenshot after click
    console.log("\n=== Step 5: Take Post-Click Screenshot ===");
    const afterClickPath = await tester.screenshot("after_new_session");
    if (afterClickPath) {
      console.log(`Screenshot saved: ${afterClickPath}`);
      console.log("  (AI would verify the new session form is visible)");
    }

    // Step 6: Type some text
    console.log("\n=== Step 6: Type Text ===");
    const typeResult = await tester.typeText("Test Session");
    if (typeResult.status === "ok") {
      console.log("✓ Text typed successfully");
    }

    // Step 7: Press Enter
    console.log("\n=== Step 7: Press Enter ===");
    await tester.keyPress("Enter");

    // Step 8: Final screenshot
    console.log("\n=== Step 8: Final Screenshot ===");
    const finalPath = await tester.screenshot("final_state");
    if (finalPath) {
      console.log(`Screenshot saved: ${finalPath}`);
    }

    console.log("\n=== Workflow Complete ===");
  } finally {
    tester.stop();
  }
}

// Interactive mode
async function interactiveMode(): Promise<void> {
  const tester = new CodyUITester();

  try {
    await tester.start();

    console.log("\nInteractive Mode");
    console.log("Commands: click <target>, type <text>, screenshot <name>, state, wait <ms>, key <key>, quit");

    const rl = createInterface({
      input: process.stdin,
      output: process.stdout,
    });

    while (true) {
      const userInput = await new Promise<string>((resolve) => {
        rl.question("\n> ", resolve);
      });

      const trimmed = userInput.trim();
      if (!trimmed) continue;
      if (trimmed === "quit") break;

      const parts = trimmed.split(/\s+(.+)/);
      const cmd = parts[0];
      const arg = parts[1] ?? "";

      try {
        switch (cmd) {
          case "click":
            const clickResult = await tester.click(arg);
            console.log(`Result: ${clickResult.status}`);
            break;

          case "type":
            const typeResult = await tester.typeText(arg);
            console.log(`Result: ${typeResult.status}`);
            break;

          case "screenshot":
            const path = await tester.screenshot(arg || "screenshot");
            console.log(`Saved to: ${path}`);
            break;

          case "state":
            const elements = await tester.getState();
            console.log(`Found ${elements.length} elements:`);
            for (const elem of elements) {
              const label = elem.label ?? "[no label]";
              console.log(`  - ${elem.element_type}: ${label}`);
            }
            break;

          case "wait":
            const ms = parseInt(arg) || 1000;
            await tester.wait(ms);
            console.log(`Waited ${ms}ms`);
            break;

          case "key":
            await tester.keyPress(arg);
            console.log(`Pressed ${arg}`);
            break;

          default:
            console.log(`Unknown command: ${cmd}`);
        }
      } catch (e) {
        console.log(`Error: ${e}`);
      }
    }

    rl.close();
  } finally {
    tester.stop();
  }
}

// Main entry point
const args = process.argv.slice(2);

if (args.includes("--interactive") || args.includes("-i")) {
  interactiveMode();
} else if (args.includes("--example") || args.includes("-e")) {
  exampleWorkflow();
} else {
  console.log("Cody UI Test Harness Client (Bun/TypeScript)");
  console.log("\nUsage:");
  console.log("  bun run ui_tester_client.ts --example    # Run example workflow");
  console.log("  bun run ui_tester_client.ts --interactive # Interactive mode");
  console.log("\nSee UI_TESTING.md for more information.");
}
