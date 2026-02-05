use crate::{
    app::App,
    opencode::{OpencodeApiClient, OpencodeProcess},
};
use egui::ViewportBuilder;
use std::sync::Arc;

mod app;
mod opencode;
mod prompt_input;

const PORT: u32 = 6767;

#[tokio::main]
async fn main() -> eframe::Result {
    env_logger::init();

    dioxus_devtools::connect_subsecond();

    let process = OpencodeProcess::start(PORT).expect("Failed to start opencode server");
    let api_client = OpencodeApiClient::new(PORT);

    let opts = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 800.0])
            .with_app_id("opencode-gui"),
        ..Default::default()
    };

    let result = subsecond::call(|| {
        eframe::run_native(
            "opencode gui",
            opts.clone(),
            Box::new(|cc| {
                // Register handler to repaint UI when patches arrive
                let ctx = cc.egui_ctx.clone();
                subsecond::register_handler(Arc::new(move || ctx.request_repaint()));
                Ok(Box::new(App::new(api_client.clone())))
            }),
        )
    });

    process.stop();
    result
}
