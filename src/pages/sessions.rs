use crate::pages::PageAction;

pub struct SessionsPage {}

impl SessionsPage {
    pub fn new() -> Self {
        Self {}
    }
}

impl SessionsPage {
    pub fn render(&mut self, ctx: &egui::Context, page_ctx: &mut super::PageContext) {
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).fill(egui::Color32::from_rgb(0, 0, 0)))
            .show(ctx, |ui| {
                ui.label("Sessions");
                let btn = ui.button("New session");
                if btn.clicked() {
                    page_ctx.action_sender.send(PageAction::CreateSession).ok();
                    println!("klsdj");
                }
            });
    }
}
