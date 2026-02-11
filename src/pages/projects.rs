use crate::backend::Project;
use crate::components::button::{ButtonSize, ButtonVariant, StyledButton};
use crate::components::text_input::StyledTextInput;
use crate::theme::{BG_50, BG_500, BG_700, BG_900, BG_950, RADIUS_MD, STROKE_WIDTH};
use egui::{Align, CentralPanel, Frame, Id, Label, Layout, Modal, RichText, Ui, vec2};
use egui_flex::{Flex, FlexAlign, FlexJustify, item};
use egui_form::garde::{GardeReport, field_path};
use egui_form::{Form, FormField};
use garde::Validate;
use uuid::Uuid;

#[derive(Debug, Default, Validate)]
struct ProjectFormFields {
    #[garde(length(min = 1))]
    name: String,
    // #[garde(length(min = 1))]
    // directory: String,
}

pub struct ProjectsPage {
    projects: Vec<Project>,
    modal_open: bool,
    modal_id: u32,
    form_fields: ProjectFormFields,
    directory_suggestions: Vec<String>,
    show_suggestions: bool,
}

impl ProjectsPage {
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
            modal_open: false,
            modal_id: 0,
            form_fields: ProjectFormFields::default(),
            directory_suggestions: Vec::new(),
            show_suggestions: false,
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, page_ctx: &mut super::PageContext) {
        CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).fill(BG_950))
            .show(ctx, |ui| {
                for updated_projects in page_ctx.sync_engine.listen_projects(ui) {
                    self.projects = updated_projects;
                }
                self.render_no_projects_screen(ui);
            });

        if self.modal_open {
            let modal_response = Modal::new(Id::new("create_project_modal").with(self.modal_id))
                .frame(
                    Frame::new()
                        .fill(BG_900)
                        .stroke(egui::Stroke::new(STROKE_WIDTH, BG_700))
                        .inner_margin(16.0)
                        .corner_radius(RADIUS_MD),
                )
                .show(ctx, |ui| {
                    self.render_modal_content(ui, page_ctx);
                });

            if modal_response.should_close() {
                self.reset_form();
            }
        }
    }

    fn render_no_projects_screen(&mut self, ui: &mut Ui) {
        Flex::vertical()
            .w_full()
            .h_full()
            .justify(FlexJustify::Center)
            .align_items(FlexAlign::Center)
            .gap(vec2(0.0, 16.0))
            .show(ui, |flex| {
                flex.add(
                    item(),
                    Label::new(RichText::new("No projects yet").color(BG_500).size(14.0)),
                );

                let btn = flex.add(item(), StyledButton::new("New project"));

                if btn.clicked() {
                    self.modal_open = true;
                }
            });
    }

    fn render_modal_content(&mut self, ui: &mut Ui, page_ctx: &mut super::PageContext) {
        ui.set_width(400.0);
        ui.spacing_mut().item_spacing.y = 6.0;

        ui.heading(RichText::new("Create New Project").color(BG_50).strong());
        ui.add_space(16.0);

        let mut form = Form::new().add_report(GardeReport::new(self.form_fields.validate()));

        self.render_form_fields(ui, &mut form);

        ui.add_space(12.0);

        self.render_form_buttons(ui, form, page_ctx);
    }

    fn render_form_fields(&mut self, ui: &mut Ui, form: &mut Form<GardeReport>) {
        FormField::new(form, field_path!("name"))
            .label("Project Name")
            .ui(
                ui,
                StyledTextInput::new(&mut self.form_fields.name).hint_text("Name of your project"),
            );

        ui.add_space(8.0);

        // let dir_response = FormField::new(form, field_path!("directory"))
        //     .label("Directory")
        //     .ui(
        //         ui,
        //         StyledTextInput::new(&mut self.form_fields.directory).hint_text("~/dev"),
        //     );
        //
        // if dir_response.changed() {
        //     self.update_directory_suggestions();
        //     self.show_suggestions = true;
        // }
    }

    fn render_form_buttons(
        &mut self,
        ui: &mut Ui,
        mut form: Form<GardeReport>,
        page_ctx: &mut super::PageContext,
    ) {
        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let create_response = StyledButton::new("Create").size(ButtonSize::Sm).show(ui);

                if let Some(Ok(())) = form.handle_submit(&create_response, ui) {
                    println!("Creating project: '{}'", self.form_fields.name);
                    let project = Project {
                        id: Uuid::new_v4(),
                        name: self.form_fields.name.clone(),
                        dir: "balls".to_string(),
                    };
                    page_ctx.sync_engine.create_project(project);

                    self.reset_form();
                }

                if StyledButton::new("Cancel")
                    .size(ButtonSize::Sm)
                    .variant(ButtonVariant::Secondary)
                    .show(ui)
                    .clicked()
                {
                    self.reset_form();
                }
            });
        });
    }

    fn reset_form(&mut self) {
        self.modal_open = false;
        self.modal_id += 1;
        self.form_fields = ProjectFormFields::default();
        self.directory_suggestions.clear();
        self.show_suggestions = false;
    }

    // fn update_directory_suggestions(&mut self) {
    //     self.directory_suggestions.clear();
    //
    //     let input = &self.form_fields.directory;
    //     if input.is_empty() {
    //         return;
    //     }
    //
    //     let path = Path::new(input);
    //
    //     // Determine the parent directory to read and the prefix to filter by
    //     let (dir_to_read, prefix) = if input.ends_with('/') || input.ends_with('\\') {
    //         (path.to_path_buf(), String::new())
    //     } else if path.parent().is_some() && path.parent().unwrap().exists() {
    //         let parent = path.parent().unwrap().to_path_buf();
    //         let file_name = path
    //             .file_name()
    //             .map(|f| f.to_string_lossy().to_string())
    //             .unwrap_or_default();
    //         (parent, file_name)
    //     } else {
    //         return;
    //     };
    //
    //     if let Ok(entries) = std::fs::read_dir(&dir_to_read) {
    //         let mut suggestions: Vec<String> = entries
    //             .filter_map(|entry| {
    //                 let entry = entry.ok()?;
    //                 let metadata = entry.metadata().ok()?;
    //                 if !metadata.is_dir() {
    //                     return None;
    //                 }
    //                 let name = entry.file_name().to_string_lossy().to_string();
    //                 if name.starts_with('.') {
    //                     return None;
    //                 }
    //                 if !prefix.is_empty()
    //                     && !name.to_lowercase().starts_with(&prefix.to_lowercase())
    //                 {
    //                     return None;
    //                 }
    //                 Some(entry.path().to_string_lossy().to_string())
    //             })
    //             .collect();
    //         suggestions.sort();
    //         suggestions.truncate(10);
    //         self.directory_suggestions = suggestions;
    //     }
    // }
}
