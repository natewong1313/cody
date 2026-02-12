use crate::{
    app::App,
    opencode::{OpencodeApiClient, OpencodeProcess},
    sync_engine::SyncEngineClient,
};
use anyhow::Result;
use egui::{FontData, FontDefinitions, FontFamily, ViewportBuilder};
use std::sync::{Arc, Mutex};

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

/// Application environment/state that gets passed to the hot-reload server
#[derive(Clone)]
struct AppEnv {
    port: u32,
    api_client: OpencodeApiClient,
}

/// Global process handle for cleanup (stored separately since it's not Clone)
static PROCESS_HANDLE: Mutex<Option<OpencodeProcess>> = Mutex::new(None);

/// Setup the application environment (runs once before hot-reloading starts)
async fn setup_app_env() -> Result<AppEnv> {
    log::info!("Setting up application environment...");

    let process = OpencodeProcess::start(PORT)
        .map_err(|e| anyhow::anyhow!("Failed to start opencode server: {}", e))?;
    let api_client = OpencodeApiClient::new(PORT);

    // Store process handle globally for cleanup
    *PROCESS_HANDLE.lock().unwrap() = Some(process);

    log::info!("Application environment setup complete");

    Ok(AppEnv {
        port: PORT,
        api_client,
    })
}

/// Cleanup function to stop the opencode process
fn cleanup_process() {
    let mut handle = PROCESS_HANDLE.lock().unwrap();
    if let Some(process) = handle.take() {
        log::info!("Stopping opencode process...");
        process.stop();
    }
}

/// Run the application with hot-reloading support
/// This function gets called by subsecond when a hot-patch is applied
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

            // Register handler to repaint UI when hot-reload patches arrive
            #[cfg(feature = "local")]
            {
                let ctx = cc.egui_ctx.clone();
                subsecond::register_handler(Arc::new(move || {
                    log::debug!("Hot-reload patch received, requesting repaint");
                    ctx.request_repaint();
                }));
                log::info!("Subsecond hot-reload handler registered");
            }

            Ok(Box::new(App::new(env.api_client.clone(), sync_engine)))
        }),
    )
}

/// Production main entry point (no hot-reloading)
#[cfg(not(feature = "local"))]
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Starting opencode gui (production mode)");

    let env = setup_app_env().await?;

    // Run the app directly without hot-reloading
    run_app(env).map_err(|e| anyhow::anyhow!("Application error: {}", e))?;

    // Cleanup
    cleanup_process();

    Ok(())
}

/// Development main entry point with hot-reloading
#[cfg(feature = "local")]
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with debug level for development
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    log::info!("Starting opencode gui (development mode with hot-reload)");
    log::info!("Run with: dx serve --hot-patch");

    let env = setup_app_env().await?;

    // Use subsecond to enable hot-reloading
    // Note: run_app must be wrapped in subsecond::call for hot-patching to work
    dioxus_devtools::serve_subsecond_with_args(env, |app_env| async move {
        // Clone the environment for each hot-reload iteration
        // subsecond::call may be invoked multiple times during hot-reloading
        subsecond::call(move || {
            let env_clone = app_env.clone();
            run_app(env_clone)
        })
    })
    .await;

    // Cleanup when dev server stops
    cleanup_process();

    Ok(())
}
