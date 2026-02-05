use crate::pages::PageAction;

pub struct SessionsPage {}

impl SessionsPage {
    pub fn new() -> Self {
        Self {}
    }
}

impl super::Page for SessionsPage {
    fn render(&mut self, ui: &mut egui::Ui, ctx: &mut super::PageContext) {
        ui.label("Sessions");
        let btn = ui.button("New session");
        if btn.clicked() {
            ctx.action_sender.send(PageAction::CreateSession).ok();
            println!("klsdj");
        }
    }
}
