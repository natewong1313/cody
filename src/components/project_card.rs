use crate::theme::{BG_50, BG_500, BG_700, BG_900, FUCHSIA_500, RADIUS_MD, STROKE_WIDTH};
use egui::RichText;
use egui_taffy::bg::simple::{TuiBackground, TuiBuilderLogicWithBackground};
use egui_taffy::taffy;
use egui_taffy::taffy::prelude::*;
use egui_taffy::{Tui, TuiBuilderLogic, tid};

pub struct ProjectCard<'a> {
    name: &'a str,
    dir: &'a str,
    index: usize,
}

impl<'a> ProjectCard<'a> {
    pub fn new(name: &'a str, dir: &'a str, index: usize) -> Self {
        Self { name, dir, index }
    }

    pub fn show(self, tui: &mut Tui) {
        tui.id(tid(self.index))
            .style(taffy::Style {
                flex_direction: taffy::FlexDirection::Column,
                align_items: Some(taffy::AlignItems::Stretch),
                padding: length(16.0),
                border: length(STROKE_WIDTH),
                ..Default::default()
            })
            .bg_add(
                TuiBackground::new()
                    .with_background_color(BG_900)
                    .with_border_color(BG_700)
                    .with_border_width(STROKE_WIDTH)
                    .with_corner_radius(RADIUS_MD),
                |tui| {
                    let first_letter = self
                        .name
                        .chars()
                        .next()
                        .unwrap_or('?')
                        .to_uppercase()
                        .to_string();

                    tui.id(tid(self.index + 1000))
                        .style(taffy::Style {
                            size: taffy::Size {
                                width: length(36.0),
                                height: length(36.0),
                            },
                            align_self: Some(taffy::AlignSelf::Start),
                            justify_content: Some(taffy::AlignContent::Center),
                            align_items: Some(taffy::AlignItems::Center),
                            margin: taffy::Rect {
                                left: zero(),
                                right: zero(),
                                top: zero(),
                                bottom: length(8.0),
                            },
                            ..Default::default()
                        })
                        .bg_add(
                            TuiBackground::new()
                                .with_background_color(FUCHSIA_500)
                                .with_corner_radius(RADIUS_MD),
                            |tui| {
                                tui.label(
                                    RichText::new(first_letter).color(BG_50).strong().size(16.0),
                                );
                            },
                        );

                    tui.heading(RichText::new(self.name).color(BG_50).strong().size(16.0));
                    tui.label(RichText::new(self.dir).color(BG_500).size(12.0));
                },
            );
    }
}
