//! UI Test Harness for Cody
//!
//! This binary runs the Cody app in headless mode and accepts JSON commands via stdin
//! to perform UI actions and capture screenshots.
//!
//! Usage:
//!   cargo run --bin ui_tester
//!
//! Then send JSON commands:
//!   {"id":"1","command":{"click":{"target":"New Session"}}}
//!   {"id":"2","command":{"screenshot":{"name":"main_screen"}}}

use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui::Context;
use egui_kittest::{
    kittest::{NodeT, Queryable},
    Harness,
};
use serde::{Deserialize, Serialize};

const SCREENSHOT_DIR: &str = "/tmp/cody-screenshots";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum UiCommand {
    /// Click on an element by its label/text content
    Click { target: String },

    /// Type text into the currently focused input
    Type { text: String },
    /// Take a screenshot and save it
    Screenshot { name: String },
    /// Get UI state (elements, their positions, etc.)
    GetState,
    /// Wait for a duration (in milliseconds)
    Wait { ms: u64 },
    /// Press a key
    KeyPress { key: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommandRequest {
    id: String,
    command: UiCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommandResponse {
    id: String,
    status: CommandStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    screenshot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    elements: Option<Vec<UiElement>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CommandStatus {
    Ok,
    Error,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UiElement {
    label: Option<String>,
    element_type: String,
    rect: [f32; 4], // [x, y, width, height]
}

impl UiCommand {
    fn timeout(&self) -> Duration {
        match self {
            UiCommand::Screenshot { .. } => Duration::from_secs(10),
            UiCommand::Wait { ms } => Duration::from_millis(*ms).max(Duration::from_secs(1)),
            _ => Duration::from_secs(10),
        }
    }
}

fn main() {
    env_logger::init();

    // Ensure screenshot directory exists
    std::fs::create_dir_all(SCREENSHOT_DIR).expect("Failed to create screenshot directory");

    eprintln!("Cody UI Test Harness");
    eprintln!("====================");
    eprintln!("Accepting JSON commands on stdin...");
    eprintln!();

    run_harness();
}

fn run_harness() {
    use std::cell::RefCell;
    use std::rc::Rc;

    // Modal state - wrapped in Rc/RefCell to share with closure
    let modal_open = Rc::new(RefCell::new(false));

    // Clone for the UI closure
    let modal_open_ui = modal_open.clone();

    // Create a harness that mimics the ProjectsPage UI
    let mut harness = Harness::new_ui(move |ui: &mut egui::Ui| {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.centered_and_justified(|ui| {
                if ui.button("New project").clicked() {
                    *modal_open_ui.borrow_mut() = true;
                }
            });
        });

        if *modal_open.borrow() {
            egui::Window::new("Create New Project")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Project Name");
                    ui.text_edit_singleline(&mut String::new());
                    ui.label("Project Directory");
                    ui.text_edit_singleline(&mut String::new());
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() {
                            *modal_open.borrow_mut() = false;
                        }
                        if ui.button("Cancel").clicked() {
                            *modal_open.borrow_mut() = false;
                        }
                    });
                });
        }
    });

    // Run once to initialize
    harness.run();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                send_error(&mut stdout, "", &format!("Failed to read input: {}", e));
                continue;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        let request: CommandRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                send_error(&mut stdout, "", &format!("Invalid JSON: {}", e));
                continue;
            }
        };

        eprintln!("Received command: {:?}", request.command);

        let timeout = request.command.timeout();
        let start = Instant::now();

        let response = match execute_command(&mut harness, &request.command) {
            Ok((screenshot, elements)) => CommandResponse {
                id: request.id,
                status: CommandStatus::Ok,
                screenshot,
                error: None,
                elements,
            },
            Err(e) => CommandResponse {
                id: request.id,
                status: CommandStatus::Error,
                screenshot: None,
                error: Some(e),
                elements: None,
            },
        };

        let elapsed = start.elapsed();
        if elapsed > timeout {
            eprintln!("Warning: Command exceeded timeout ({:?})", elapsed);
        }

        send_response(&mut stdout, &response);
    }

    eprintln!("Test harness shutting down...");
}

fn execute_command(
    harness: &mut Harness,
    command: &UiCommand,
) -> Result<(Option<String>, Option<Vec<UiElement>>), String> {
    match command {
        UiCommand::Click { target } => {
            eprintln!("Looking for element with label: {}", target);

            let node = harness.get_by_label(target);
            eprintln!("Found element, clicking...");
            node.click();
            harness.run();

            Ok((None, None))
        }

        UiCommand::Type { text } => {
            eprintln!("Typing text: {}", text);
            for c in text.chars() {
                if let Some(key) = char_to_key(c) {
                    harness.key_down(key);
                    harness.key_up(key);
                } else {
                    eprintln!("Warning: Cannot type character: {}", c);
                }
            }
            harness.run();
            Ok((None, None))
        }

        UiCommand::Screenshot { name } => {
            let path = PathBuf::from(SCREENSHOT_DIR).join(format!("{}.png", name));
            eprintln!("Taking screenshot: {}", path.display());

            // Use egui_kittest's snapshot feature
            harness.snapshot(name);

            Ok((Some(path.to_string_lossy().to_string()), None))
        }

        UiCommand::GetState => {
            eprintln!("Collecting UI state...");
            let elements = collect_ui_elements(harness);
            Ok((None, Some(elements)))
        }

        UiCommand::Wait { ms } => {
            eprintln!("Waiting for {} ms", ms);
            std::thread::sleep(Duration::from_millis(*ms));
            harness.run();
            Ok((None, None))
        }

        UiCommand::KeyPress { key } => {
            eprintln!("Pressing key: {}", key);
            handle_key_press(harness, key)?;
            harness.run();
            Ok((None, None))
        }
    }
}

fn handle_key_press(harness: &mut Harness, key: &str) -> Result<(), String> {
    match key {
        "Enter" | "Return" => {
            harness.key_down(egui::Key::Enter);
            harness.key_up(egui::Key::Enter);
        }
        "Escape" | "Esc" => {
            harness.key_down(egui::Key::Escape);
            harness.key_up(egui::Key::Escape);
        }
        "Tab" => {
            harness.key_down(egui::Key::Tab);
            harness.key_up(egui::Key::Tab);
        }
        "Backspace" => {
            harness.key_down(egui::Key::Backspace);
            harness.key_up(egui::Key::Backspace);
        }
        "Space" => {
            harness.key_down(egui::Key::Space);
            harness.key_up(egui::Key::Space);
        }
        _ => {
            if key.len() == 1 {
                let c = key.chars().next().unwrap();
                if let Some(egui_key) = char_to_key(c) {
                    harness.key_down(egui_key);
                    harness.key_up(egui_key);
                } else {
                    return Err(format!("Cannot convert character to key: {}", c));
                }
            } else {
                return Err(format!("Unknown key: {}", key));
            }
        }
    }
    Ok(())
}

fn char_to_key(c: char) -> Option<egui::Key> {
    match c {
        'a' | 'A' => Some(egui::Key::A),
        'b' | 'B' => Some(egui::Key::B),
        'c' | 'C' => Some(egui::Key::C),
        'd' | 'D' => Some(egui::Key::D),
        'e' | 'E' => Some(egui::Key::E),
        'f' | 'F' => Some(egui::Key::F),
        'g' | 'G' => Some(egui::Key::G),
        'h' | 'H' => Some(egui::Key::H),
        'i' | 'I' => Some(egui::Key::I),
        'j' | 'J' => Some(egui::Key::J),
        'k' | 'K' => Some(egui::Key::K),
        'l' | 'L' => Some(egui::Key::L),
        'm' | 'M' => Some(egui::Key::M),
        'n' | 'N' => Some(egui::Key::N),
        'o' | 'O' => Some(egui::Key::O),
        'p' | 'P' => Some(egui::Key::P),
        'q' | 'Q' => Some(egui::Key::Q),
        'r' | 'R' => Some(egui::Key::R),
        's' | 'S' => Some(egui::Key::S),
        't' | 'T' => Some(egui::Key::T),
        'u' | 'U' => Some(egui::Key::U),
        'v' | 'V' => Some(egui::Key::V),
        'w' | 'W' => Some(egui::Key::W),
        'x' | 'X' => Some(egui::Key::X),
        'y' | 'Y' => Some(egui::Key::Y),
        'z' | 'Z' => Some(egui::Key::Z),
        '0' => Some(egui::Key::Num0),
        '1' => Some(egui::Key::Num1),
        '2' => Some(egui::Key::Num2),
        '3' => Some(egui::Key::Num3),
        '4' => Some(egui::Key::Num4),
        '5' => Some(egui::Key::Num5),
        '6' => Some(egui::Key::Num6),
        '7' => Some(egui::Key::Num7),
        '8' => Some(egui::Key::Num8),
        '9' => Some(egui::Key::Num9),
        _ => None,
    }
}

fn collect_ui_elements(harness: &Harness) -> Vec<UiElement> {
    let mut elements = vec![];

    // Access the root node and traverse the tree
    let root = harness.root();
    collect_elements_recursive(&root, &mut elements);

    elements
}

fn collect_elements_recursive(node: &egui_kittest::Node<'_>, elements: &mut Vec<UiElement>) {
    // Get the AccessKit node
    let ak_node = node.accesskit_node();

    // AccessKit nodes use label() instead of name()
    let label = ak_node.label().map(|s| s.to_string());

    // Handle nodes without rects - use default rect if not available
    let rect = match ak_node.bounding_box() {
        Some(bounds) => [
            bounds.x0 as f32,
            bounds.y0 as f32,
            (bounds.x1 - bounds.x0) as f32,
            (bounds.y1 - bounds.y0) as f32,
        ],
        None => [0.0, 0.0, 0.0, 0.0],
    };

    let element = UiElement {
        label,
        element_type: format!("{:?}", ak_node.role()),
        rect,
    };

    elements.push(element);

    // Recursively collect from children
    for child in node.children() {
        collect_elements_recursive(&child, elements);
    }
}

fn send_response(stdout: &mut io::Stdout, response: &CommandResponse) {
    let json = match serde_json::to_string(response) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Failed to serialize response: {}", e);
            return;
        }
    };

    if let Err(e) = writeln!(stdout, "{}", json) {
        eprintln!("Failed to write response: {}", e);
        return;
    }

    if let Err(e) = stdout.flush() {
        eprintln!("Failed to flush stdout: {}", e);
    }
}

fn send_error(stdout: &mut io::Stdout, id: &str, error: &str) {
    eprintln!("Error: {}", error);
    let response = CommandResponse {
        id: id.to_string(),
        status: CommandStatus::Error,
        screenshot: None,
        error: Some(error.to_string()),
        elements: None,
    };
    send_response(stdout, &response);
}
