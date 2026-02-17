use egui::{
    Align2, AtomExt, Button, Context, Frame, Id, Image, Label, Margin, Modal, Pos2, RichText,
    Separator, Stroke, Ui, Vec2, include_image, style::WidgetVisuals, vec2,
};
use egui_flex::{Flex, FlexAlignContent, item};

use crate::{
    App,
    error::RitmError,
    ui::{font::Font, theme::Theme},
};

pub mod help;
pub mod settings;
pub mod state_edit;
pub mod transition_edit;

#[derive(PartialEq, Clone, Debug)]
pub enum RitmPopupEnum {
    TransitionEdit((usize, usize)),
    StateEdit(Option<usize>, Option<Pos2>),
    Settings,
    Help,
}

#[derive(Debug, Default)]
pub struct RitmPopup {
    current_popup: Option<RitmPopupEnum>,
}

impl RitmPopup {
    pub fn current(&self) -> Option<&RitmPopupEnum> {
        self.current_popup.as_ref()
    }

    pub fn switch_to(&mut self, popup: RitmPopupEnum) {
        self.current_popup = Some(popup)
    }

    pub fn close(&mut self) {
        self.current_popup = None
    }
}

pub fn show(ctx: &Context, app: &mut App) -> Result<(), RitmError> {
    if let Some(ritmpopup) = app.popup.current_popup.clone() {
        match ritmpopup {
            RitmPopupEnum::TransitionEdit((source_id, target_id)) => {
                let source = app.turing.get_state(source_id)?.get_name();
                let target = app.turing.get_state(target_id)?.get_name();
                let max_size = vec2(
                    200.0 + app.turing.tm.graph_ref().get_k() as f32 * 125.0,
                    ctx.available_rect().height(),
                );
                modal(
                    ctx,
                    app,
                    format!("{} -> {}", source, target),
                    false,
                    |ui, app| {
                        ui.set_max_size(max_size);
                        // ui.set_min_size(vec2(max_size.x, 0.0));
                        transition_edit::show(ui, app)
                    },
                )?
            }
            RitmPopupEnum::StateEdit(state_id, _) => {
                let title = if let Some(state_id) = state_id {
                    app.turing.get_state(state_id)?.get_name().to_string()
                } else {
                    "New State".to_string()
                };
                let max_size = vec2(300.0, ctx.available_rect().height());
                modal(ctx, app, title, false, |ui, app| {
                    ui.set_max_size(max_size);
                    ui.set_min_size(vec2(max_size.x, 0.0));
                    state_edit::show(ui, app)
                })?
            }
            RitmPopupEnum::Help => modal(ctx, app, "Help".to_string(), true, |ui, app| {
                ui.set_max_size(ui.ctx().screen_rect().size() * 0.8);
                ui.set_min_size(ui.ctx().screen_rect().size() * 0.8);
                help::show(ui, app)
            })?,
            RitmPopupEnum::Settings => modal(ctx, app, "Settings".to_string(), true, |ui, app| {
                ui.set_max_size(ui.ctx().screen_rect().size() * 0.8);
                ui.set_min_size(ui.ctx().screen_rect().size() * 0.8);
                settings::show(ui, app)
            })?,
        }
    }
    Ok(())
}

fn modal<R>(
    ctx: &Context,
    app: &mut App,
    title: String,
    can_be_closed: bool,
    content: impl FnOnce(&mut Ui, &mut App) -> Result<R, RitmError>,
) -> Result<R, RitmError> {
    let frame = Frame {
        fill: app.theme.surface,
        stroke: Stroke::new(2.0, app.theme.border),
        inner_margin: Margin::same(10),
        corner_radius: 10.into(),
        ..Default::default()
    };

    Modal::new(Id::new(&title))
        .frame(frame)
        .show(ctx, |ui| header(ui, app, title, can_be_closed, content))
        .inner
}

fn header<R>(
    ui: &mut Ui,
    app: &mut App,
    title: String,
    can_be_closed: bool,
    content: impl FnOnce(&mut Ui, &mut App) -> Result<R, RitmError>,
) -> Result<R, RitmError> {
    Theme::set_widget(
        ui,
        WidgetVisuals {
            bg_stroke: Stroke::new(1.0, app.theme.border),
            corner_radius: 5.into(),
            expansion: 3.0,
            ..app.theme.default_widget()
        },
    );

    Flex::horizontal()
        .w_full()
        .align_content(FlexAlignContent::SpaceBetween)
        .show(ui, |flex| {
            flex.add(
                item(),
                Label::new(RichText::new(title).font(Font::default_big())),
            );

            flex.add_ui(
                item().align_self_content(Align2::RIGHT_CENTER).grow(1.0),
                |ui| {
                    if can_be_closed {
                        let img = Image::new(include_image!("../../assets/icon/back.svg"))
                            .fit_to_exact_size(Vec2::splat(25.0))
                            .tint(app.theme.overlay)
                            .atom_size(Vec2::splat(25.0));

                        if ui
                            .add(
                                Button::new((img, RichText::new("Back").font(Font::default_big())))
                                    .frame(false),
                            )
                            .clicked()
                        {
                            app.popup.close();
                        }
                    };
                },
            );
        });

    ui.add(Separator::default().spacing(15.0).horizontal().grow(10.0));

    ui.vertical(|ui| content(ui, app)).inner
}

pub fn boolean_popup(
    ui: &mut Ui,
    app: &mut App,
    question: &str,
) -> Result<Option<bool>, RitmError> {
    modal(ui.ctx(), app, "Info".to_string(), false, |ui, app| {
        ui.label(RichText::new(question).font(Font::default_medium()));
        ui.add_space(10.0);
        ui.spacing_mut().button_padding = vec2(0.0, 8.0);
        ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
        ui.columns(2, |ui| {
            if ui[0]
                .add(Button::new(
                    RichText::new("yes")
                        .font(Font::default_medium())
                        .atom_grow(true),
                ))
                .clicked()
            {
                app.popup.close();
                return Ok(Some(true));
            }
            if ui[1]
                .add(Button::new(
                    RichText::new("no")
                        .font(Font::default_medium())
                        .atom_grow(true),
                ))
                .clicked()
            {
                app.popup.close();
                return Ok(Some(false));
            }
            Ok(None)
        })
    })
}

// pub fn warning(ui: &mut Ui, app: &mut App, question: &str) -> Result<(), RitmError> {
//     modal(ui.ctx(), app, "Warning".to_string(), true, |ui, _app| {
//         ui.label(RichText::new(question).font(Font::default_medium()));
//         Ok(())
//     })
// }
