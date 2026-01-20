use egui::{
    AtomExt, Button, Context, Frame, Id, Image, Label, LayerId, Margin, Modal, Popup, PopupAnchor,
    RichText, Separator, Stroke, Ui, Vec2, include_image, style::WidgetVisuals,
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
    TransitionEdit(String),
    StateEdit(String),
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
    if let Some(popup) = app.popup.current_popup.clone() {
        match popup {
            RitmPopupEnum::TransitionEdit(title) => {
                show_in(ctx, app, false, true, &title, transition_edit::show)?
            }
            RitmPopupEnum::StateEdit(title) => {
                show_in(ctx, app, false, true, &title, state_edit::show)?
            }
            RitmPopupEnum::Help => show_in(ctx, app, true, false, "Help", help::show)?,
            RitmPopupEnum::Settings => show_in(ctx, app, true, false, "Settings", settings::show)?,
        }
    }
    Ok(())
}

fn show_in(
    ctx: &Context,
    app: &mut App,
    can_be_closed: bool,
    can_be_moved: bool,
    title: &str,
    content: impl FnOnce(&mut Ui, &mut App) -> Result<(), RitmError>,
) -> Result<(), RitmError> {
    let frame = Frame {
        fill: app.theme.background,
        stroke: Stroke::new(2.0, app.theme.border),
        inner_margin: Margin::same(10),
        corner_radius: 10.into(),
        ..Default::default()
    };

    if can_be_moved {
        if let Some(res) = Popup::new(
            Id::new("popup"),
            ctx.clone(),
            PopupAnchor::Position(ctx.screen_rect().center()),
            LayerId::new(egui::Order::Foreground, Id::new("popup/modal-layer")),
        )
        .frame(frame)
        .show(|ui| {
            header(ui, app, title, can_be_closed, can_be_moved, content)
        }) {
            res.inner
        } else {
            Ok(())
        }
    } else {
        Modal::new(Id::new("modal"))
            .frame(frame)
            .show(ctx, |ui| {
                header(ui, app, title, can_be_closed, can_be_moved, content)
            })
            .inner
    }
}

fn header(
    ui: &mut Ui,
    app: &mut App,
    title: &str,
    can_be_closed: bool,
    _can_be_moved: bool,
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

    // Popup header
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
                    .tint(app.theme.icon)
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
