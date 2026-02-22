use egui::Ui;
use egui_inbox::UiInbox;

use crate::backend::Backend;

pub enum QueryUIMessage {}

pub struct QueryClient<B>
where
    B: Backend,
{
    backend: B,
    updates_inbox: UiInbox<QueryUIMessage>,
}

impl<B> QueryClient<B>
where
    B: Backend,
{
    pub fn new(backend: B) -> Self {
        let updates = UiInbox::new();
        Self {
            backend,
            updates_inbox: updates,
        }
    }

    // pub fn subscribe_projects(&self, ui: Ui) {
    //     let projects = self.backend.list_projects();
    // }
}
