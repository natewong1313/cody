use crate::{
    app::App,
    backend::{BackendServer, Contract},
    opencode::{OpencodeApiClient, OpencodeProcess},
    sync_engine::SyncEngineClient,
};
use egui::{FontData, FontDefinitions, FontFamily, ViewportBuilder};
use futures::StreamExt;
use std::sync::Arc;
use tarpc::server::{self, Channel};

mod actions;
mod app;
mod backend;
mod components;
mod opencode;
mod pages;
mod sync_engine;
mod theme;
// mod ui_tests;

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
                let sync_engine = SyncEngineClient::new();

                // Load JetBrains Mono Nerd Font
                let mut fonts = FontDefinitions::default();

                fonts.font_data.insert(
                    "JetBrainsMono-Regular".to_owned(),
                    FontData::from_static(include_bytes!(
                        "../assets/JetBrainsMono/JetBrainsMonoNerdFont-Regular.ttf"
                    ))
                    .into(),
                );
                fonts.font_data.insert(
                    "JetBrainsMono-Bold".to_owned(),
                    FontData::from_static(include_bytes!(
                        "../assets/JetBrainsMono/JetBrainsMonoNerdFont-Bold.ttf"
                    ))
                    .into(),
                );
                fonts.font_data.insert(
                    "JetBrainsMono-Italic".to_owned(),
                    FontData::from_static(include_bytes!(
                        "../assets/JetBrainsMono/JetBrainsMonoNerdFont-Italic.ttf"
                    ))
                    .into(),
                );
                fonts.font_data.insert(
                    "JetBrainsMono-BoldItalic".to_owned(),
                    FontData::from_static(include_bytes!(
                        "../assets/JetBrainsMono/JetBrainsMonoNerdFont-BoldItalic.ttf"
                    ))
                    .into(),
                );
                fonts.font_data.insert(
                    "JetBrainsMono-Medium".to_owned(),
                    FontData::from_static(include_bytes!(
                        "../assets/JetBrainsMono/JetBrainsMonoNerdFont-Medium.ttf"
                    ))
                    .into(),
                );
                fonts.font_data.insert(
                    "JetBrainsMono-MediumItalic".to_owned(),
                    FontData::from_static(include_bytes!(
                        "../assets/JetBrainsMono/JetBrainsMonoNerdFont-MediumItalic.ttf"
                    ))
                    .into(),
                );

                // Set JetBrains Mono as the primary text font
                fonts
                    .families
                    .entry(FontFamily::Proportional)
                    .or_default()
                    .insert(0, "JetBrainsMono-Regular".to_owned());
                fonts
                    .families
                    .entry(FontFamily::Monospace)
                    .or_default()
                    .insert(0, "JetBrainsMono-Regular".to_owned());

                // Register Phosphor icons as a dedicated font family so icons
                // aren't shadowed by Nerd Font PUA glyphs in the same range
                fonts.font_data.insert(
                    "phosphor".into(),
                    egui_phosphor::Variant::Regular.font_data().into(),
                );
                fonts
                    .families
                    .entry(FontFamily::Name("phosphor".into()))
                    .or_default()
                    .push("phosphor".into());

                cc.egui_ctx.set_fonts(fonts);

                // Register handler to repaint UI when patches arrive
                let ctx = cc.egui_ctx.clone();
                subsecond::register_handler(Arc::new(move || ctx.request_repaint()));
                Ok(Box::new(App::new(api_client.clone(), sync_engine)))
            }),
        )
    });

    process.stop();
    result
}
