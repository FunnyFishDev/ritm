use egui::{
    Align2, AtomExt, Button, Color32, Frame, Image, ImageButton, Margin, Popup, PopupCloseBehavior,
    RectAlign, RichText, Separator, Stroke, Ui, Vec2, include_image, vec2,
};
use egui_flex::{Flex, FlexAlign, FlexAlignContent, FlexDirection, FlexInstance, item};
use include_directory::{Dir, include_directory};

use crate::{
    App,
    error::{GuiError, RitmError},
    ui::{
        constant::Constant,
        font::Font,
        popup::{RitmPopupEnum, boolean_popup},
        tutorial::{TutorialBox, TutorialEnum},
        utils::FileDialog,
    },
};

static EXAMPLES: Dir = include_directory!("ritm_core/resources");

#[derive(Default)]
pub struct Menu {
    /// File loaded
    pub file: FileDialog,
}

/// Global application control, like settings, compile or load file
pub fn show(app: &mut App, ui: &mut Ui) -> Result<(), RitmError> {
    if let Some(tutorial) = app.tutorial.current_tutorial()
        && tutorial == TutorialEnum::Menu
    {
        app.code.open();
    }

    app.tutorial.add_boxe(
        "menu_section",
        TutorialBox::new(ui.available_rect_before_wrap()).with_align(Align2::CENTER_BOTTOM),
    );

    let mut flex = Flex::new()
        .align_items(FlexAlign::Center)
        .align_content(FlexAlignContent::Start)
        .gap(vec2(10.0, 10.0));

    flex = if app.code.is_closed() {
        flex.direction(FlexDirection::Vertical).h_full()
    } else {
        flex.direction(FlexDirection::Horizontal).w_full()
    };

    Frame::new()
        .inner_margin(Margin::same(5))
        .show(ui, |ui| {
            flex.show(ui, |ui| {
                app.transient.is_small_window = ui.ui().ctx().screen_rect().width()
                    < ((Constant::ICON_SIZE + 10.0) * 6.0) * 3.0;

                if app.transient.is_small_window {
                    app.code.close();
                }

                panel_open(app, ui);

                settings(app, ui);

                save(app, ui)?;

                machine_folder(app, ui)?;

                help(app, ui);

                if !app.code.is_closed() {
                    to_graph(app, ui)?;

                    ui.grow();

                    panel_close(app, ui);
                }
                Ok(())
            })
            .inner
        })
        .inner
}

fn settings(app: &mut App, ui: &mut FlexInstance) {
    let button = ui.add(
        item(),
        ImageButton::new(
            Image::new(include_image!("../../assets/icon/setting.svg"))
                .fit_to_exact_size(Vec2::splat(Constant::ICON_SIZE))
                .tint(app.theme.icon),
        )
        .frame(false),
    );
    if button.clicked() {
        app.popup.switch_to(RitmPopupEnum::Settings);
    }

    app.tutorial.add_boxe(
        "setting",
        TutorialBox::new(button.rect.expand(5.0)).with_align(Align2::RIGHT_BOTTOM),
    );
}

fn save(app: &mut App, ui: &mut FlexInstance) -> Result<(), RitmError> {
    let button = ui.add(
        item(),
        ImageButton::new(
            Image::new(include_image!("../../assets/icon/save.svg"))
                .fit_to_exact_size(Vec2::splat(Constant::ICON_SIZE))
                .tint(if app.code.current_code()?.is_empty() {
                    app.theme.disabled
                } else {
                    app.theme.icon
                }),
        )
        .frame(false),
    );
    if button.clicked() && !app.code.current_code()?.is_empty() {
        app.menu
            .file
            .save("new.tm", app.code.current_code()?.as_bytes().to_vec())
    };

    app.tutorial.add_boxe(
        "save",
        TutorialBox::new(button.rect.expand(5.0)).with_align(Align2::RIGHT_BOTTOM),
    );

    Ok(())
}

fn machine_folder(app: &mut App, ui: &mut FlexInstance) -> Result<(), RitmError> {
    let res = ui.add(
        item(),
        ImageButton::new(
            Image::new(include_image!("../../assets/icon/machine_folder.svg"))
                .fit_to_exact_size(Vec2::splat(Constant::ICON_SIZE))
                .tint(app.theme.icon),
        )
        .frame(false),
    );

    app.tutorial.add_boxe(
        "machine_folder",
        TutorialBox::new(res.rect.expand(5.0)).with_align(Align2::RIGHT_BOTTOM),
    );

    Popup::menu(&res)
        .gap(if app.code.is_closed() { 10.0 } else { 5.0 })
        .align(if app.code.is_closed() {
            RectAlign::RIGHT_START
        } else {
            RectAlign::BOTTOM_START
        })
        .close_behavior(PopupCloseBehavior::CloseOnClick)
        .show(|ui| {
            for example in EXAMPLES.files() {
                let filename = example
                    .path()
                    .file_stem()
                    .expect("should exist")
                    .to_str()
                    .expect("should translate");
                let code = example.contents_utf8().expect("should exist").to_string();
                let button = Button::new(
                    RichText::new(filename)
                        .font(Font::default_small())
                        .color(app.theme.text_primary),
                )
                .frame(false)
                .min_size(vec2(0.0, 25.0));
                if ui.add(button).clicked() {
                    app.code.new_tab(filename.to_string(), code);
                    app.code_to_graph()?; // TODO: add a setting to toggle this
                }
            }

            ui.visuals_mut().widgets.noninteractive.bg_stroke = Stroke::new(1.0, app.theme.border);
            ui.add(Separator::default().grow(6.0));

            let img = Image::new(include_image!("../../assets/icon/upload.svg"))
                .fit_to_exact_size(Vec2::splat(25.0))
                .tint(app.theme.overlay)
                .atom_size(Vec2::splat(25.0));

            if ui
                .add(
                    Button::new((
                        RichText::new("Upload")
                            .font(Font::default_small())
                            .color(app.theme.text_primary),
                        img,
                    ))
                    .frame(false),
                )
                .clicked()
            {
                app.menu.file.open();
            }

            Ok::<(), RitmError>(())
        });

    if let Some(file) = app.menu.file.get() {
        app.transient.temp_code = Some(
            std::str::from_utf8(&file)
                .map_err(|e| {
                    RitmError::GuiError(GuiError::FileError(format!("Could not load file {e}",)))
                })?
                .to_string(),
        );
    }

    if let Some(code) = &app.transient.temp_code {
        let code = code.clone();
        ui.add_ui(item(), |ui| {
            if let Some(answer) = boolean_popup(ui, app, "Do you want to create a new tab ?")? {
                if answer {
                    app.code.new_tab(app.code.tab_name(), code);
                } else {
                    *app.code.current_code_mut()? = code;
                }
                app.transient.temp_code = None;
            }
            Ok::<(), RitmError>(())
        });
    }
    Ok(())
}

fn help(app: &mut App, ui: &mut FlexInstance) {
    let button = ui.add(
        item(),
        ImageButton::new(
            Image::new(include_image!("../../assets/icon/help.svg"))
                .fit_to_exact_size(Vec2::splat(Constant::ICON_SIZE))
                .tint(app.theme.icon),
        )
        .frame(false),
    );

    app.tutorial.add_boxe(
        "help",
        TutorialBox::new(button.rect.expand(5.0)).with_align(Align2::CENTER_BOTTOM),
    );

    Popup::menu(&button)
        .gap(if app.code.is_closed() { 10.0 } else { 5.0 })
        .align(if app.code.is_closed() {
            RectAlign::RIGHT_START
        } else {
            RectAlign::BOTTOM_START
        })
        .close_behavior(PopupCloseBehavior::CloseOnClick)
        .show(|ui| {
            fn button(ui: &mut Ui, app: &mut App, tutorial: TutorialEnum, already_played: bool) {
                ui.horizontal(|ui| {
                    ui.add_sized(
                        vec2(25.0, 25.0),
                        Image::new(include_image!("../../assets/icon/correct.svg")).tint(
                            if already_played {
                                app.theme.success
                            } else {
                                Color32::TRANSPARENT
                            },
                        ),
                    );
                    let button = Button::new(
                        RichText::new(tutorial.to_string())
                            .font(Font::default_small())
                            .color(app.theme.text_primary),
                    )
                    .frame(false)
                    .min_size(vec2(0.0, 25.0));
                    if ui.add(button).clicked() {
                        app.transient.temp_tutorial = Some(tutorial);
                    }
                });
            }

            let mut keys: Vec<(TutorialEnum, bool)> = app
                .tutorial
                .already_played
                .keys()
                .copied()
                .zip(app.tutorial.already_played.values().copied())
                .collect();
            keys.sort();
            for (tutorial, already_played) in keys {
                button(ui, app, tutorial, already_played);
            }

            Ok::<(), RitmError>(())
        });
}

fn to_graph(app: &mut App, ui: &mut FlexInstance) -> Result<(), RitmError> {
    let button = ui.add(
        item(),
        ImageButton::new(
            Image::new(include_image!("../../assets/icon/graph.svg"))
                .fit_to_exact_size(Vec2::splat(Constant::ICON_SIZE))
                .tint(app.theme.icon),
        )
        .frame(false),
    );
    if button.clicked() {
        app.code_to_graph()?;
    }

    app.tutorial.add_boxe(
        "to_graph",
        TutorialBox::new(button.rect.expand(5.0)).with_align(Align2::CENTER_BOTTOM),
    );

    Ok(())
}

fn panel_close(app: &mut App, ui: &mut FlexInstance) {
    let button = ui.add(
        item(),
        ImageButton::new(
            Image::new(include_image!("../../assets/icon/panel_close.svg"))
                .fit_to_exact_size(Vec2::splat(Constant::ICON_SIZE))
                .tint(app.theme.icon),
        )
        .frame(false),
    );
    if button.clicked() {
        app.code.close();
    }

    app.tutorial.add_boxe(
        "close",
        TutorialBox::new(button.rect.expand(5.0)).with_align(Align2::CENTER_BOTTOM),
    );
}

fn panel_open(app: &mut App, ui: &mut FlexInstance) {
    if app.code.is_closed()
        && !app.transient.is_small_window
        && ui
            .add(
                item(),
                ImageButton::new(
                    Image::new(include_image!("../../assets/icon/panel_open.svg"))
                        .fit_to_exact_size(Vec2::splat(Constant::ICON_SIZE))
                        .tint(app.theme.icon),
                )
                .frame(false),
            )
            .clicked()
    {
        app.code.open();
    }
}
