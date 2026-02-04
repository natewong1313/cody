use egui::ViewportBuilder;

use crate::{
    app::App,
    opencode::{OpencodeApiClient, OpencodeProcess},
};

mod app;
mod opencode;
mod prompt_input;

const PORT: u32 = 6767;

#[tokio::main]
async fn main() -> eframe::Result {
    env_logger::init();

    let process = OpencodeProcess::start(PORT).expect("Failed to start opencode server");
    let api_client = OpencodeApiClient::new(PORT);

    let opts = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 800.0])
            .with_app_id("opencode-gui"),
        ..Default::default()
    };

    let result = eframe::run_native(
        "opencode gui",
        opts,
        Box::new(|_cc| Ok(Box::new(App::new(api_client)))),
    );

    process.stop();
    result
}
