use egui::{
    AtomExt, Button, Context, Frame, Id, Image, Label, LayerId,
    Margin, Modal, Popup, PopupAnchor, RichText, Separator, Stroke, Ui, Vec2, include_image,
    style::WidgetVisuals,
};
use egui_flex::{Flex, FlexAlignContent, item};

use crate::{
    App,
    ui::{font::Font, theme::Theme},
};

pub mod help;
pub mod settings;
pub mod state_edit;
pub mod transition_edit;

#[derive(PartialEq, Clone)]
pub enum RitmPopup {
    None,
    TransitionEdit(String),
    StateEdit(String),
    Settings,
    Help,
}

pub fn show(ctx: &Context, app: &mut App) {
    match app.popup.clone() {
        RitmPopup::None => {}
        RitmPopup::TransitionEdit(title) => {
            show_in(ctx, app, false, true, &title, transition_edit::show)
        }
        RitmPopup::StateEdit(title) => show_in(ctx, app, false, true, &title, state_edit::show),
        RitmPopup::Help => show_in(ctx, app, true, false, "Help", help::show),
        RitmPopup::Settings => show_in(ctx, app, true, false, "Settings", settings::show),
    }
}

fn show_in<R>(
    ctx: &Context,
    app: &mut App,
    can_be_closed: bool,
    can_be_moved: bool,
    title: &str,
    content: impl FnOnce(&mut Ui, &mut App) -> R,
) {
    let frame = Frame {
        fill: app.theme.white,
        stroke: Stroke::new(2.0, app.theme.gray),
        inner_margin: Margin::same(10),
        corner_radius: 10.into(),
        ..Default::default()
    };

    if can_be_moved {
        Popup::new(
            Id::new("popup"),
            ctx.clone(),
            PopupAnchor::PointerFixed,
            LayerId::new(egui::Order::Foreground, Id::new("popup/modal-layer")),
        )
        .frame(frame)
        .show(|ui| {
            Theme::set_widget(
                ui,
                WidgetVisuals {
                    bg_stroke: Stroke::new(1.0, app.theme.gray),
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
                            .tint(app.theme.gray)
                            .atom_size(Vec2::splat(25.0));

                        if flex
                            .add(
                                item(),
                                Button::new((img, RichText::new("Back").font(Font::default_big())))
                                    .frame(false),
                            )
                            .clicked()
                        {
                            app.popup = RitmPopup::None;
                        }
                    }
                });

            ui.add(Separator::default().spacing(15.0).horizontal().grow(10.0));

            ui.vertical(|ui| content(ui, app))
        });
    } else {
        Modal::new(Id::new("modal")).frame(frame).show(ctx, |ui| {
            // ui.set_max_size(ui.ctx().screen_rect().size() * 0.8);

            Theme::set_widget(
                ui,
                WidgetVisuals {
                    bg_stroke: Stroke::new(1.0, app.theme.gray),
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
                            .tint(app.theme.gray)
                            .atom_size(Vec2::splat(25.0));

                        if flex
                            .add(
                                item(),
                                Button::new((img, RichText::new("Back").font(Font::default_big())))
                                    .frame(false),
                            )
                            .clicked()
                        {
                            app.popup = RitmPopup::None;
                        }
                    }
                });

            ui.add(Separator::default().spacing(15.0).horizontal().grow(10.0));

            ui.vertical(|ui| content(ui, app))
        });
    }
}
