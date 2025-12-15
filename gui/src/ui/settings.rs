use egui::{
    AtomExt, Button, Image, ImageButton, Popup, PopupCloseBehavior, RectAlign, RichText, Separator,
    Stroke, Ui, Vec2, include_image, vec2,
};
use egui_flex::{Flex, FlexAlign, FlexAlignContent, FlexDirection, item};
use include_directory::{Dir, include_directory};

use crate::{
    App,
    ui::{constant::Constant, font::Font, popup::RitmPopup},
};

static EXAMPLES: Dir = include_directory!("ritm_core/resources");

/// Global application control, like settings, compile or load file
pub fn show(app: &mut App, ui: &mut Ui) {
    let mut flex = Flex::new()
        .align_items(FlexAlign::Center)
        .align_content(FlexAlignContent::Start)
        .gap(vec2(10.0, 10.0));

    flex = if app.event.is_code_closed {
        flex.direction(FlexDirection::Vertical).h_full()
    } else {
        flex.direction(FlexDirection::Horizontal).w_full()
    };

    flex.show(ui, |ui| {
        app.event.is_small_window =
            ui.ui().ctx().screen_rect().width() < ((Constant::ICON_SIZE + 10.0) * 6.0) * 3.0;

        if app.event.is_small_window {
            app.event.is_code_closed = true;
        }

        if app.event.is_code_closed
            && !app.event.is_small_window
                && ui
                    .add(
                        item(),
                        ImageButton::new(
                            Image::new(include_image!("../../assets/icon/panel_open.svg"))
                                .fit_to_exact_size(Vec2::splat(Constant::ICON_SIZE))
                                .tint(app.theme.white),
                        )
                        .frame(false),
                    )
                    .clicked()
            {
                app.event.is_code_closed = false;
            }

        if ui
            .add(
                item(),
                ImageButton::new(
                    Image::new(include_image!("../../assets/icon/setting.svg"))
                        .fit_to_exact_size(Vec2::splat(Constant::ICON_SIZE))
                        .tint(app.theme.white),
                )
                .frame(false),
            )
            .clicked()
        {
            app.popup = RitmPopup::Settings;
        }

        if ui
            .add(
                item(),
                ImageButton::new(
                    Image::new(include_image!("../../assets/icon/save.svg"))
                        .fit_to_exact_size(Vec2::splat(Constant::ICON_SIZE))
                        .tint(if app.code.is_empty() {
                            app.theme.gray
                        } else {
                            app.theme.white
                        }),
                )
                .frame(false),
            )
            .clicked()
            && !app.code.is_empty() {
                app.file.save("new.tm", app.code.as_bytes().to_vec())
            };

        let res = ui.add(
            item(),
            ImageButton::new(
                Image::new(include_image!("../../assets/icon/machine_folder.svg"))
                    .fit_to_exact_size(Vec2::splat(Constant::ICON_SIZE))
                    .tint(app.theme.white),
            )
            .frame(false),
        );

        Popup::menu(&res)
            .gap(if app.event.is_code_closed { 10.0 } else { 5.0 })
            .align(if app.event.is_code_closed {
                RectAlign::RIGHT_START
            } else {
                RectAlign::BOTTOM_START
            })
            .close_behavior(PopupCloseBehavior::CloseOnClick)
            .show(|ui| {
                for example in EXAMPLES.files() {
                    let button = Button::new(
                        RichText::new(example.path().file_stem().unwrap().to_str().unwrap())
                            .font(Font::default_small())
                            .color(app.theme.gray),
                    )
                    .frame(false)
                    .min_size(vec2(0.0, 25.0));
                    if ui.add(button).clicked() {
                        app.code = example.contents_utf8().unwrap().to_string();
                        app.code_to_graph();
                    }
                }

                ui.visuals_mut().widgets.noninteractive.bg_stroke =
                    Stroke::new(1.0, app.theme.gray);
                ui.add(Separator::default().grow(6.0));

                let img = Image::new(include_image!("../../assets/icon/upload.svg"))
                    .fit_to_exact_size(Vec2::splat(25.0))
                    .tint(app.theme.gray)
                    .atom_size(Vec2::splat(25.0));

                if ui
                    .add(
                        Button::new((RichText::new("Upload").font(Font::default_small()), img))
                            .frame(false),
                    )
                    .clicked()
                {
                    app.file.open();
                }
            });

        if let Some(file) = app.file.get() {
            app.code = std::str::from_utf8(&file).unwrap().to_string()
        }

        if ui
            .add(
                item(),
                ImageButton::new(
                    Image::new(include_image!("../../assets/icon/help.svg"))
                        .fit_to_exact_size(Vec2::splat(Constant::ICON_SIZE))
                        .tint(app.theme.white),
                )
                .frame(false),
            )
            .clicked()
        {
            app.popup = RitmPopup::Help;
        }

        if !app.event.is_code_closed {
            if ui
                .add(
                    item(),
                    ImageButton::new(
                        Image::new(include_image!("../../assets/icon/graph.svg"))
                            .fit_to_exact_size(Vec2::splat(Constant::ICON_SIZE))
                            .tint(app.theme.white),
                    )
                    .frame(false),
                )
                .clicked()
            {
                app.code_to_graph();
            }

            if ui
                .add(
                    item(),
                    ImageButton::new(
                        Image::new(include_image!("../../assets/icon/panel_close.svg"))
                            .fit_to_exact_size(Vec2::splat(Constant::ICON_SIZE))
                            .tint(app.theme.white),
                    )
                    .frame(false),
                )
                .clicked()
            {
                app.event.is_code_closed = true;
            }
        }
    });
}
