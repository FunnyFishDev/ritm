use egui::{
    AtomExt, Button, Context, Frame, Id, Image, Label, LayerId, Margin, Modal, Popup, PopupAnchor,
    Pos2, RichText, Separator, Stroke, Ui, Vec2, include_image, style::WidgetVisuals, vec2,
};
use egui_flex::{Flex, FlexAlignContent, item};

use crate::{
    App,
    error::RitmError,
    turing::TransitionId,
    ui::{font::Font, theme::Theme},
};

pub mod help;
pub mod settings;
pub mod state_edit;
pub mod transition_edit;

#[derive(PartialEq, Clone, Debug)]
pub enum RitmPopupEnum {
    TransitionEdit(TransitionId),
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
            RitmPopupEnum::TransitionEdit(transition_id) => {
                let source = app.turing.get_state(transition_id.source_id)?.get_name();
                let target = app.turing.get_state(transition_id.target_id)?.get_name();
                popup(
                    ctx,
                    app,
                    format!("{} -> {}", source, target),
                    false,
                    |ui, app| {
                        ui.set_max_size(vec2(300.0, ui.ctx().available_rect().height()));
                        ui.set_min_size(vec2(300.0, 0.0));
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
                popup(ctx, app, title, false, |ui, app| {
                    ui.set_max_size(vec2(300.0, ui.ctx().available_rect().height()));
                    ui.set_min_size(vec2(300.0, 0.0));
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

fn popup(
    ctx: &Context,
    app: &mut App,
    title: String,
    can_be_closed: bool,
    content: impl FnOnce(&mut Ui, &mut App) -> Result<(), RitmError>,
) -> Result<(), RitmError> {
    let frame = Frame {
        fill: app.theme.surface,
        stroke: Stroke::new(2.0, app.theme.border),
        inner_margin: Margin::same(10),
        corner_radius: 10.into(),
        ..Default::default()
    };

    if let Some(res) = Popup::new(
        Id::new("popup"),
        ctx.clone(),
        PopupAnchor::Position(ctx.screen_rect().center()),
        LayerId::new(egui::Order::Foreground, Id::new("popup/modal-layer")),
    )
    .frame(frame)
    .show(|ui| header(ui, app, title, can_be_closed, content))
    {
        res.inner
    } else {
        Ok(())
    }
}

fn modal(
    ctx: &Context,
    app: &mut App,
    title: String,
    can_be_closed: bool,
    content: impl FnOnce(&mut Ui, &mut App) -> Result<(), RitmError>,
) -> Result<(), RitmError> {
    let frame = Frame {
        fill: app.theme.surface,
        stroke: Stroke::new(2.0, app.theme.border),
        inner_margin: Margin::same(10),
        corner_radius: 10.into(),
        ..Default::default()
    };

    Modal::new(Id::new("modal"))
        .frame(frame)
        .show(ctx, |ui| header(ui, app, title, can_be_closed, content))
        .inner
}


    fn header(
        ui: &mut Ui,
        app: &mut App,
        title: String,
        can_be_closed: bool,
        content: impl FnOnce(&mut Ui, &mut App) -> Result<(), RitmError>,
    ) -> Result<(), RitmError> {
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

                flex.grow();

                if can_be_closed {
                    let img = Image::new(include_image!("../../assets/icon/back.svg"))
                        .fit_to_exact_size(Vec2::splat(25.0))
                        .tint(app.theme.overlay)
                        .atom_size(Vec2::splat(25.0));

                    if flex
                        .add(
                            item(),
                            Button::new((img, RichText::new("Back").font(Font::default_big())))
                                .frame(false),
                        )
                        .clicked()
                    {
                        app.popup.close();
                    }
                }
            });

        ui.add(Separator::default().spacing(15.0).horizontal().grow(10.0));

        ui.vertical(|ui| content(ui, app)).inner
    }
