use egui::{Align, Layout, ViewportBuilder};

use crate::opencode::{kill_opencode_server, spawn_opencode_server};
mod opencode;
mod prompt_input;

fn main() -> eframe::Result {
    env_logger::init();

    let mut opencode_proc = spawn_opencode_server().unwrap();

    let opts = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 800.0])
            .with_app_id("opencode-gui"),
        ..Default::default()
    };

    let result = eframe::run_native(
        "opencode gui",
        opts,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    );
    kill_opencode_server(&opencode_proc).unwrap();
    opencode_proc.wait().unwrap();
    result
}

#[derive(Default)]
struct MyApp {
    show_confirmation_dialog: bool,
    allowed_to_close: bool,
    prompt_input: prompt_input::PromptInput,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                self.prompt_input.show(ui);
            });
        });
    }
}
