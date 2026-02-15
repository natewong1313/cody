use crate::backend::Project;
use crate::components::button::{ButtonSize, ButtonVariant, StyledButton};
use crate::components::directory_picker::DirectoryPicker;
use crate::components::project_card::ProjectCard;
use crate::components::text_input::StyledTextInput;
use crate::listen;
use crate::theme::{BG_50, BG_500, BG_700, BG_900, BG_950, RADIUS_MD, STROKE_WIDTH};
use egui::{Align, CentralPanel, Frame, Id, Label, Layout, Modal, RichText, Stroke, Ui, vec2};
use egui_flex::{Flex, FlexAlign, FlexJustify, item};
use egui_form::garde::{GardeReport, field_path};
use egui_form::{Form, FormField};
use egui_phosphor::regular;
use egui_taffy::TuiBuilderLogic;
use egui_taffy::taffy;
use egui_taffy::taffy::prelude::*;
use egui_taffy::tui;
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
                self.setup_listeners(ui, page_ctx);

                // if self.projects.len() == 0 {
                //     self.render_no_projects_screen(ui);
                // } else {
                self.render_projects(ui);
                // }
            });

        if self.modal_open {
            self.render_modal(ctx, page_ctx);
        }
    }

    fn setup_listeners(&mut self, ui: &mut Ui, page_ctx: &mut super::PageContext) {
        listen!(
            self,
            ui,
            |ui| page_ctx.sync_engine.listen_projects(ui),
            projects
        );
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

    fn render_projects(&mut self, ui: &mut Ui) {
        let placeholder_projects = [
            ("Cody", "~/dev/cody"),
            ("Website Redesign", "~/projects/website-redesign"),
            ("API Server", "~/dev/api-server"),
            ("Mobile App", "~/projects/mobile-app"),
            ("Data Pipeline", "~/dev/data-pipeline"),
            ("Design System", "~/projects/design-system"),
        ];

        tui(ui, ui.id().with("projects_grid_wrapper"))
            .reserve_available_width()
            .style(taffy::Style {
                display: taffy::Display::Flex,
                justify_content: Some(taffy::JustifyContent::Center),
                size: taffy::Size {
                    width: percent(1.0),
                    height: auto(),
                },
                ..Default::default()
            })
            .show(|tui| {
                tui.style(taffy::Style {
                    display: taffy::Display::Grid,
                    max_size: taffy::Size {
                        width: length(700.0),
                        height: auto(),
                    },
                    size: taffy::Size {
                        width: percent(1.0),
                        height: auto(),
                    },
                    grid_template_columns: vec![fr(1.0), fr(1.0), fr(1.0)],
                    grid_auto_rows: vec![min_content()],
                    gap: taffy::style_helpers::length(16.),
                    padding: taffy::style_helpers::length(16.),
                    ..Default::default()
                })
                .add(|tui| {
                    for (i, (name, dir)) in placeholder_projects.iter().enumerate() {
                        ProjectCard::new(name, dir, i).show(tui);
                    }
                });
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
                ui.set_width(400.0);
                // Need this for input label spacing
                ui.spacing_mut().item_spacing.y = 6.0;

                ui.heading(RichText::new("Create New Project").color(BG_50).strong());
                ui.add_space(16.0);

                let mut form =
                    Form::new().add_report(GardeReport::new(self.form_fields.validate()));

                FormField::new(&mut form, field_path!("name"))
                    .label("Project Name")
                    .ui(
                        ui,
                        StyledTextInput::new(&mut self.form_fields.name)
                            .hint_text("Name of your project"),
                    );

                FormField::new(&mut form, field_path!("dir"))
                    .label("Project Directory")
                    .ui(
                        ui,
                        DirectoryPicker::new(&mut self.form_fields.dir)
                            .hint_text("Search for a folder...")
                            .id(Id::new("project_dir_picker").with(self.modal_id)),
                    );

                self.render_form_buttons(ui, form, page_ctx);
            });

        if modal_response.should_close() {
            self.reset_form();
        }
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
