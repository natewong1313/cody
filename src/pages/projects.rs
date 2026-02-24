use crate::backend::Project;
use crate::components::button::{ButtonSize, ButtonVariant, StyledButton};
use crate::components::dir_button::DirButton;
use crate::components::project_card::ProjectCard;
use crate::components::text_input::StyledTextInput;
use crate::pages::{PageAction, Route};
use crate::theme::{BG_50, BG_500, BG_700, BG_900, BG_950, RADIUS_MD, STROKE_WIDTH};
use chrono::Utc;
use egui::{
    Align, CentralPanel, Frame, Grid, Id, Label, Layout, Margin, Modal, RichText, Stroke, Ui, vec2,
};
use egui_flex::{Flex, FlexAlign, FlexJustify, item};
use egui_form::garde::{GardeReport, field_path};
use egui_form::{Form, FormField};
use egui_inbox::UiInbox;
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
    modal_open: bool,
    modal_id: u32,
    form_fields: ProjectFormFields,
    dir_inbox: UiInbox<String>,
}

impl ProjectsPage {
    pub fn new() -> Self {
        Self {
            modal_open: false,
            modal_id: 0,
            form_fields: ProjectFormFields::default(),
            dir_inbox: UiInbox::new(),
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, page_ctx: &mut super::PageContext) {
        CentralPanel::default()
            .frame(
                Frame::central_panel(&ctx.style())
                    .fill(BG_950)
                    .inner_margin(0.0),
            )
            .show(ctx, |ui| {
                // let loading = page_ctx.query.projects_loading();
                // let projects = page_ctx.query.projects().to_vec();
                // let error = page_ctx.query.projects_error().map(str::to_owned);
                //
                // if projects.is_empty() && loading {
                //     ui.centered_and_justified(|ui| {
                //         ui.label(
                //             RichText::new("Loading projects...")
                //                 .color(BG_500)
                //                 .size(16.0),
                //         );
                //     });
                //     return;
                // }
                //
                // if projects.is_empty() {
                //     if let Some(error) = error {
                //         ui.centered_and_justified(|ui| {
                //             ui.label(RichText::new(error).color(egui::Color32::RED).size(14.0));
                //         });
                //         ui.add_space(8.0);
                //         ui.centered_and_justified(|ui| {
                //             if StyledButton::new("Retry").show(ui).clicked() {
                //                 page_ctx.query.refresh_projects();
                //             }
                //         });
                //         return;
                //     }
                //
                //     self.render_no_projects_screen(ui);
                //     return;
                // }
                //
                // self.render_projects_screen(ui, page_ctx, &projects);
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
                    egui::Image::from_bytes(
                        "bytes://shape1.svg",
                        include_bytes!("../../assets/shape1.svg"),
                    )
                    .fit_to_exact_size(vec2(96.0, 96.0)),
                );

                flex.add(
                    item(),
                    Label::new(RichText::new("No projects yet").color(BG_500).size(16.0)),
                );

                let btn = flex.add(item(), StyledButton::new("New project"));

                if btn.clicked() {
                    self.modal_open = true;
                }
            });
    }

    fn render_projects_screen(
        &mut self,
        ui: &mut Ui,
        page_ctx: &mut super::PageContext,
        projects: &[Project],
    ) {
        const GRID_MAX_WIDTH: f32 = 700.0;
        const GRID_PADDING: f32 = 16.0;

        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            let grid_width = ui.available_width().min(GRID_MAX_WIDTH);
            ui.set_max_width(grid_width);

            Frame::new().inner_margin(GRID_PADDING).show(ui, |ui| {
                Frame::new()
                    .outer_margin(Margin {
                        bottom: 16,
                        ..Default::default()
                    })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if StyledButton::new("Create Project")
                                .variant(ButtonVariant::Primary)
                                .show(ui)
                                .clicked()
                            {
                                self.modal_open = true;
                            }
                        });
                    });
                self.render_projects_grid(ui, page_ctx, projects);
            });
        });
    }

    fn render_projects_grid(
        &mut self,
        ui: &mut Ui,
        page_ctx: &mut super::PageContext,
        projects: &[Project],
    ) {
        const GRID_COLUMNS: usize = 3;
        const GRID_GAP: f32 = 16.0;
        let total_gap = GRID_GAP * (GRID_COLUMNS as f32 - 1.0);
        let card_width = ((ui.available_width() - total_gap) / GRID_COLUMNS as f32).max(0.0);

        Grid::new("projects_grid")
            .num_columns(GRID_COLUMNS)
            .spacing(vec2(GRID_GAP, GRID_GAP))
            .min_col_width(card_width)
            .max_col_width(card_width)
            .show(ui, |ui| {
                for (i, proj) in projects.iter().enumerate() {
                    let response = ProjectCard::new(&proj.name, &proj.dir, i).show(ui);
                    if response.clicked() {
                        println!("Sending click event");
                        page_ctx
                            .action_sender
                            .send(PageAction::Navigate(Route::Project { id: proj.id }))
                            .ok();
                    }

                    if (i + 1) % GRID_COLUMNS == 0 {
                        ui.end_row();
                    }
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
                ui.set_width(400.0);

                ui.heading(RichText::new("Create New Project").color(BG_50).strong());
                ui.add_space(16.0);

                let mut form =
                    Form::new().add_report(GardeReport::new(self.form_fields.validate()));

                let dir_display = self.form_fields.dir.clone();
                FormField::new(&mut form, field_path!("dir"))
                    .label("Directory")
                    .ui(
                        ui,
                        DirButton::new(&dir_display, &self.dir_inbox).on_dir_change(|dir| {
                            if self.form_fields.name.is_empty() {
                                if let Some(name) = std::path::Path::new(&dir)
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                {
                                    self.form_fields.name = name.to_string();
                                }
                            }
                            self.form_fields.dir = dir;
                        }),
                    );

                FormField::new(&mut form, field_path!("name"))
                    .label("Project name")
                    .ui(
                        ui,
                        StyledTextInput::new(&mut self.form_fields.name)
                            .hint_text("Enter a name for your project"),
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
                    self.on_create_project_click(page_ctx);
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

    fn on_create_project_click(&mut self, page_ctx: &mut super::PageContext) {
        let now = Utc::now().naive_utc();
        let project_id = Uuid::new_v4();
        let project = Project {
            id: project_id,
            name: self.form_fields.name.clone(),
            dir: self.form_fields.dir.clone(),
            created_at: now,
            updated_at: now,
        };

        // page_ctx.query.create_project(project);

        self.reset_form();
    }

    fn reset_form(&mut self) {
        self.modal_open = false;
        self.modal_id += 1;
        self.form_fields = ProjectFormFields::default();
    }
}
