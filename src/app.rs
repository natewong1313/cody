use std::sync::mpsc::{Receiver, Sender, channel};

use crate::BACKEND_ADDR;
use crate::mutations::MutationsClient;
use crate::query::QueryClient;
use crate::{
    actions::{ActionContext, handle_action},
    pages::{PageAction, PageContext, PagesRouter},
};

use tonic::transport::{Channel, Endpoint};

pub struct App {
    #[allow(dead_code)]
    backend_channel: Channel,
    query_client: QueryClient,
    mutations_client: MutationsClient,
    pages_router: PagesRouter,

    pub action_sender: Sender<PageAction>,
    action_reciever: Receiver<PageAction>,
}

impl App {
    pub fn new() -> Self {
        let (action_sender, action_reciever) = channel();
        let backend_channel = Endpoint::from_shared(format!("http://{BACKEND_ADDR}"))
            .unwrap()
            .connect_lazy();
        let query_client = QueryClient::new();
        let mutations_client = MutationsClient::new(backend_channel.clone());
        Self {
            backend_channel,
            query_client,
            mutations_client,
            pages_router: PagesRouter::new(),

            action_sender,
            action_reciever,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(action) = self.action_reciever.try_recv() {
            let mut action_ctx = ActionContext {
                pages_router: &mut self.pages_router,
            };
            handle_action(&mut action_ctx, action);
        }

        let mut page_ctx = PageContext {
            action_sender: &self.action_sender,
            query: &mut self.query_client,
            mutations: &self.mutations_client,
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
