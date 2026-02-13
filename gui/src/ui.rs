use egui::{
    Align, CentralPanel, CornerRadius, Frame, Layout, Margin, SidePanel, TopBottomPanel, vec2,
};

pub mod code;
pub mod component;
pub mod constant;
pub mod control;
pub mod edit;
pub mod font;
pub mod graph;
pub mod menu;
pub mod popup;
pub mod ribbon;
pub mod theme;
pub mod tutorial;
pub mod utils;

use crate::{App, error::RitmError, ui::font::Font};

pub fn show(app: &mut App, ctx: &egui::Context) -> Result<(), RitmError> {
    CentralPanel::default()
        .frame(Frame {
            outer_margin: Margin::same(0),
            inner_margin: Margin::same(0),
            fill: app.theme.background,
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.spacing_mut().indent = 10.0;
            ui.style_mut().override_font_id = Some(Font::default_medium()); // TODO check if there is not a better way to do that

            if app.code.is_closed() {
                SidePanel::left("settings")
                    .frame(Frame {
                        inner_margin: 5.into(),
                        ..Default::default()
                    })
                    .resizable(false)
                    .exact_width(45.0)
                    .show_inside(ui, |ui| menu::show(app, ui))
                    .inner?;

                // Ribbon and execution control
                TopBottomPanel::top("ribbon")
                    .frame(Frame {
                        outer_margin: Margin {
                            bottom: 10,
                            ..Default::default()
                        },
                        inner_margin: Margin::same(10),
                        corner_radius: CornerRadius {
                            sw: 5,
                            ..Default::default()
                        },
                        fill: app.theme.primary,
                        ..Default::default()
                    })
                    .resizable(false)
                    .show_separator_line(false)
                    .show_inside(ui, |ui| {
                        ribbon::show(app, ui);
                        control::show(app, ui)?;
                        Ok::<(), RitmError>(())
                    })
                    .inner?;

                // Graph visual and edition
                CentralPanel::default()
                    .frame(Frame {
                        outer_margin: Margin::same(0),
                        inner_margin: Margin::same(0),
                        fill: app.theme.secondary,
                        corner_radius: CornerRadius {
                            nw: 5,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .show_inside(ui, |ui| {
                        graph::show(app, ui)?;
                        Ok::<(), RitmError>(())
                    });
                Ok::<(), RitmError>(())
            } else {
                // Code and file loading
                SidePanel::left("code")
                    .frame(Frame {
                        outer_margin: Margin {
                            right: 10,
                            ..Default::default()
                        },
                        fill: app.theme.background,
                        ..Default::default()
                    })
                    .resizable(false)
                    .show_separator_line(false)
                    .max_width(ui.available_width() / 3.0)
                    .min_width(ui.available_width() / 3.0)
                    .show_inside(ui, |ui| {
                        TopBottomPanel::top("settings")
                            .frame(Frame {
                                fill: app.theme.background,
                                inner_margin: 5.into(),
                                ..Default::default()
                            })
                            .resizable(false)
                            .show_separator_line(false)
                            .show_inside(ui, |ui| menu::show(app, ui))
                            .inner?;

                        CentralPanel::default()
                            .frame(Frame {
                                outer_margin: Margin::same(0),
                                inner_margin: Margin::same(0),
                                fill: app.theme.code_background,
                                corner_radius: CornerRadius {
                                    ne: 5,
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .show_inside(ui, |ui| code::show(app, ui))
                            .inner
                    });

                // Ribbon and Graph
                CentralPanel::default()
                    .frame(Frame {
                        outer_margin: Margin::same(0),
                        inner_margin: Margin::same(0),
                        ..Default::default()
                    })
                    .show_inside(ui, |ui| {
                        // Ribbon and execution control
                        TopBottomPanel::top("ribbon")
                            .frame(Frame {
                                outer_margin: Margin {
                                    bottom: 10,
                                    ..Default::default()
                                },
                                inner_margin: Margin::same(10),
                                corner_radius: CornerRadius {
                                    sw: 5,
                                    ..Default::default()
                                },
                                fill: app.theme.primary,
                                ..Default::default()
                            })
                            .resizable(false)
                            .show_separator_line(false)
                            .show_inside(ui, |ui| {
                                ui.allocate_ui_with_layout(
                                    vec2(ui.available_width(), 0.0),
                                    Layout::top_down(Align::Min),
                                    |ui| {
                                        ribbon::show(app, ui);
                                        control::show(app, ui)?;
                                        Ok::<(), RitmError>(())
                                    },
                                )
                                .inner
                            })
                            .inner?;

                        // Graph visual and edition
                        CentralPanel::default()
                            .frame(Frame {
                                outer_margin: Margin::same(0),
                                inner_margin: Margin::same(0),
                                fill: app.theme.secondary,
                                corner_radius: CornerRadius {
                                    nw: 5,
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .show_inside(ui, |ui| {
                                graph::show(app, ui)?;
                                Ok::<(), RitmError>(())
                            });
                        Ok::<(), RitmError>(())
                    });
                Ok(())
            }
        });

    // Display the current popup/modal
    popup::show(ctx, app)?;

    Ok::<(), RitmError>(())
}
