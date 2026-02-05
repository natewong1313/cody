use egui::TextEdit;

pub struct SessionPage {
    session_id: String,
    prompt_input: String,
}

impl SessionPage {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            prompt_input: "".to_string(),
        }
    }
}

impl super::Page for SessionPage {
    fn render(&mut self, ui: &mut egui::Ui, ctx: &mut super::PageContext) {
        let session = ctx.current_sessions.get(&self.session_id).unwrap();

        ui.label("Session");
        ui.label(session.title.clone().unwrap_or("Hello".to_string()));
        ui.add(TextEdit::multiline(&mut self.prompt_input));
    }
}
