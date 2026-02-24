use crate::app::App;
use egui::ViewportBuilder;
use egui::{FontData, FontDefinitions, FontFamily};

use anyhow::Result;

mod actions;
mod app;
mod backend;
mod components;
mod opencode;
mod pages;
mod query;
mod theme;

pub const BACKEND_ADDR: &str = "[::1]:50051";

#[derive(Clone)]
struct AppEnv {}

impl AppEnv {
    pub fn new() -> Self {
        Self {}
    }
}

fn configure_egui(cc: &eframe::CreationContext<'_>) {
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
    egui_extras::install_image_loaders(&cc.egui_ctx);

    #[cfg(feature = "local")]
    {
        use std::sync::Arc;

        let ctx = cc.egui_ctx.clone();
        subsecond::register_handler(Arc::new(move || {
            log::debug!("Hot-reload patch received, requesting repaint");
            ctx.request_repaint();
        }));
        log::info!("Subsecond hot-reload handler registered");
    }
}

fn run_app(env: AppEnv) -> eframe::Result {
    let opts = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 800.0])
            .with_app_id("opencode-gui"),
        ..Default::default()
    };

    eframe::run_native(
        "opencode gui",
        opts,
        Box::new(move |cc| {
            configure_egui(cc);
            Ok(Box::new(App::new()))
        }),
    )
}

#[cfg(not(feature = "local"))]
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .filter(Some("winit"), log::LevelFilter::Warn)
        .filter(Some("tracing::span"), log::LevelFilter::Warn)
        .init();

    log::info!("Starting opencode gui (production mode)");

    let grpc_addr = BACKEND_ADDR.parse()?;
    let _backend_task = backend::spawn_backend(grpc_addr)
        .map_err(|e| anyhow::anyhow!("Failed to start backend gRPC server: {e}"))?;

    let env = AppEnv::new();

    run_app(env).map_err(|e| anyhow::anyhow!("Application error: {}", e))?;

    Ok(())
}

#[cfg(feature = "local")]
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .filter(Some("winit"), log::LevelFilter::Warn)
        .filter(Some("tracing::span"), log::LevelFilter::Warn)
        .init();

    log::info!("Starting opencode gui (development mode with hot-reload)");
    log::info!("Run with: dx serve --hot-patch");

    let grpc_addr = BACKEND_ADDR.parse()?;
    let _backend_task = backend::spawn_backend(grpc_addr)
        .map_err(|e| anyhow::anyhow!("Failed to start backend gRPC server: {e}"))?;

    // let backend_client = backend::rpc::start_local_backend_rpc()
    //     .await
    //     .map_err(|e| anyhow::anyhow!("Failed to start backend RPC: {e}"))?;
    let env = AppEnv::new();

    dioxus_devtools::serve_subsecond_with_args(env, |app_env| async move {
        subsecond::call(move || {
            let env_clone = app_env.clone();
            run_app(env_clone)
        })
    })
    .await;

    Ok(())
}
