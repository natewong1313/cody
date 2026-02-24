use std::sync::mpsc::{Receiver, Sender, channel};

use crate::query::QueryClient;
use crate::{
    actions::{ActionContext, handle_action},
    pages::{PageAction, PageContext, PagesRouter},
};

#[cfg(feature = "local")]
use subsecond;
use tonic::transport::{Channel, Endpoint};

const BACKEND_ENDPOINT: &str = "http://127.0.0.1:50051";

pub struct App {
    backend_channel: Channel,
    query_client: QueryClient,
    pages_router: PagesRouter,

    pub action_sender: Sender<PageAction>,
    action_reciever: Receiver<PageAction>,
}

impl App {
    pub fn new() -> Self {
        let (action_sender, action_reciever) = channel();
        let query_client = QueryClient::new();
        let backend_channel = Endpoint::from_static(BACKEND_ENDPOINT).connect_lazy();
        Self {
            backend_channel,
            query_client,
            pages_router: PagesRouter::new(),

            action_sender,
            action_reciever,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // self.query_client.load_projects_if_needed();
        // self.query_client.poll(ctx);

        while let Ok(action) = self.action_reciever.try_recv() {
            let mut action_ctx = ActionContext {
                pages_router: &mut self.pages_router,
            };
            handle_action(&mut action_ctx, action);
        }

        let mut page_ctx = PageContext {
            action_sender: &self.action_sender,
        };

        // Wrap rendering in subsecond::call() for hot-reloading support
        // This allows changes in pages/ to be hot-patched without restart
        #[cfg(feature = "local")]
        subsecond::call(|| {
            self.pages_router.mount(ctx, &mut page_ctx);
        });

        #[cfg(not(feature = "local"))]
        self.pages_router.mount(ctx, &mut page_ctx);
    }
}
