use crate::app::App;
use egui::{FontData, FontDefinitions, FontFamily};

#[cfg(not(all(feature = "browser", target_arch = "wasm32")))]
use egui::ViewportBuilder;

#[cfg(not(all(feature = "browser", target_arch = "wasm32")))]
use anyhow::Result;
#[cfg(all(feature = "browser", target_arch = "wasm32"))]
use eframe::web_sys::{self, HtmlCanvasElement};
#[cfg(all(feature = "browser", target_arch = "wasm32"))]
use wasm_bindgen::JsCast;
#[cfg(all(feature = "browser", target_arch = "wasm32"))]
use wasm_bindgen::prelude::*;

mod actions;
mod app;
// #[cfg(all(feature = "browser", target_arch = "wasm32"))]
// #[path = "backend_web.rs"]
// mod backend;
#[cfg(not(all(feature = "browser", target_arch = "wasm32")))]
mod backend;
mod components;
mod opencode;
mod pages;
#[cfg(not(all(feature = "browser", target_arch = "wasm32")))]
mod query;
mod theme;

#[derive(Clone)]
struct AppEnv {
    backend_client: backend::rpc::BackendRpcClient,
}

impl AppEnv {
    pub fn new(backend_client: backend::rpc::BackendRpcClient) -> Self {
        Self { backend_client }
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

#[cfg(not(all(feature = "browser", target_arch = "wasm32")))]
fn run_app(env: AppEnv) -> eframe::Result {
    let backend_client = env.backend_client;
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
            Ok(Box::new(App::new(backend_client.clone())))
        }),
    )
}

#[cfg(not(feature = "local"))]
#[cfg(not(all(feature = "browser", target_arch = "wasm32")))]
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .filter(Some("winit"), log::LevelFilter::Warn)
        .filter(Some("tracing::span"), log::LevelFilter::Warn)
        .init();

    log::info!("Starting opencode gui (production mode)");

    let backend_client = backend::rpc::start_local_backend_rpc()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start backend RPC: {e}"))?;
    let env = AppEnv::new(backend_client);

    run_app(env).map_err(|e| anyhow::anyhow!("Application error: {}", e))?;

    Ok(())
}

#[cfg(feature = "local")]
#[cfg(not(all(feature = "browser", target_arch = "wasm32")))]
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .filter(Some("winit"), log::LevelFilter::Warn)
        .filter(Some("tracing::span"), log::LevelFilter::Warn)
        .init();

    log::info!("Starting opencode gui (development mode with hot-reload)");
    log::info!("Run with: dx serve --hot-patch");

    let backend_client = backend::rpc::start_local_backend_rpc()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start backend RPC: {e}"))?;
    let env = AppEnv::new(backend_client);

    dioxus_devtools::serve_subsecond_with_args(env, |app_env| async move {
        subsecond::call(move || {
            let env_clone = app_env.clone();
            run_app(env_clone)
        })
    })
    .await;

    Ok(())
}

#[cfg(all(feature = "browser", target_arch = "wasm32"))]
#[wasm_bindgen(start)]
pub fn wasm_start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    wasm_bindgen_futures::spawn_local(async {
        let web_options = eframe::WebOptions::default();
        let runner = eframe::WebRunner::new();
        let window = web_sys::window().expect("window not available");
        let document = window.document().expect("document not available");
        let canvas: HtmlCanvasElement = document
            .get_element_by_id("the_canvas_id")
            .expect("canvas element with id 'the_canvas_id' not found")
            .dyn_into()
            .expect("element with id 'the_canvas_id' is not a canvas");
        let result = runner
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    configure_egui(cc);
                    Ok(Box::new(App::new()))
                }),
            )
            .await;

        if let Err(err) = result {
            log::error!("Failed to start web app: {err:?}");
        }
    });

    Ok(())
}

#[cfg(all(feature = "browser", target_arch = "wasm32"))]
fn main() {}
