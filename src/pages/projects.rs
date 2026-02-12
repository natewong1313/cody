use crate::backend::Project;
use crate::components::button::{ButtonSize, ButtonVariant, StyledButton};
use crate::components::directory_picker::DirectoryPicker;
use crate::components::text_input::StyledTextInput;
use crate::listen;
use crate::theme::{BG_50, BG_500, BG_700, BG_900, BG_950, RADIUS_MD, STROKE_WIDTH};
use egui::{vec2, Align, CentralPanel, Frame, Id, Label, Layout, Modal, RichText, Stroke, Ui};
use egui_flex::{item, Flex, FlexAlign, FlexJustify};
use egui_form::garde::{field_path, GardeReport};
use egui_form::{Form, FormField};
use egui_phosphor::regular;
use garde::Validate;
use uuid::Uuid;

#[derive(Debug, Default, Validate)]
struct ProjectFormFields {
    #[garde(length(min = 1))]
    name: String,
    #[garde(length(min = 1))]
    dir: String,
}

pub struct ProjectsPage {
    projects: Vec<Project>,
    modal_open: bool,
    modal_id: u32,
    form_fields: ProjectFormFields,
}

impl ProjectsPage {
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
            modal_open: false,
            modal_id: 0,
            form_fields: ProjectFormFields::default(),
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, page_ctx: &mut super::PageContext) {
        CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).fill(BG_950))
            .show(ctx, |ui| {
                listen!(
                    self,
                    ui,
                    |ui| page_ctx.sync_engine.listen_projects(ui),
                    projects
                );
                self.render_no_projects_screen(ui);
            });

        if self.modal_open {
            self.render_modal(ctx, page_ctx);
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

    fn render_modal(&mut self, ctx: &egui::Context, page_ctx: &mut super::PageContext) {
        let modal_response = Modal::new(Id::new("create_project_modal").with(self.modal_id))
            .frame(
                Frame::new()
                    .fill(BG_900)
                    .stroke(Stroke::new(STROKE_WIDTH, BG_700))
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
                StyledTextInput::new(&mut self.form_fields.name)
                    .hint_text("Name of your project")
                    .desired_width(368.0),
            );

        ui.add_space(8.0);

        FormField::new(form, field_path!("dir"))
            .label("Project Directory")
            .ui(
                ui,
                DirectoryPicker::new(&mut self.form_fields.dir)
                    .hint_text("Search for a folder...")
                    .id(Id::new("project_dir_picker").with(self.modal_id)),
            );

        ui.add_space(8.0);
    }

    fn render_form_buttons(
        &mut self,
        ui: &mut Ui,
        mut form: Form<GardeReport>,
        page_ctx: &mut super::PageContext,
    ) {
        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let create_response = StyledButton::new("Create")
                    .size(ButtonSize::Sm)
                    .icon(regular::PLUS)
                    .show(ui);

                if let Some(Ok(())) = form.handle_submit(&create_response, ui) {
                    println!(
                        "Creating project: '{}' at '{}'",
                        self.form_fields.name, self.form_fields.dir
                    );
                    let project = Project {
                        id: Uuid::new_v4(),
                        name: self.form_fields.name.clone(),
                        dir: self.form_fields.dir.clone(),
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
    }
}
