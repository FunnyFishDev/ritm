use egui::{
    AtomExt, Button, Image, ImageButton, Popup, PopupCloseBehavior, RectAlign, RichText, Separator,
    Stroke, Ui, Vec2, include_image, vec2,
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
    let mut flex = Flex::new()
        .align_items(FlexAlign::Center)
        .align_content(FlexAlignContent::Start)
        .gap(vec2(10.0, 10.0));

    flex = if app.code.is_closed() {
        flex.direction(FlexDirection::Vertical).h_full()
    } else {
        flex.direction(FlexDirection::Horizontal).w_full()
    };

    flex.show(ui, |ui| {
        app.transient.is_small_window =
            ui.ui().ctx().screen_rect().width() < ((Constant::ICON_SIZE + 10.0) * 6.0) * 3.0;

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
}

fn settings(app: &mut App, ui: &mut FlexInstance) {
    if ui
        .add(
            item(),
            ImageButton::new(
                Image::new(include_image!("../../assets/icon/setting.svg"))
                    .fit_to_exact_size(Vec2::splat(Constant::ICON_SIZE))
                    .tint(app.theme.icon),
            )
            .frame(false),
        )
        .clicked()
    {
        app.popup.switch_to(RitmPopupEnum::Settings);
    }
}

fn save(app: &mut App, ui: &mut FlexInstance) -> Result<(), RitmError> {
    if ui
        .add(
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
        )
        .clicked()
        && !app.code.current_code()?.is_empty()
    {
        app.menu
            .file
            .save("new.tm", app.code.current_code()?.as_bytes().to_vec())
    };
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
        app.transient.code = Some(
            std::str::from_utf8(&file)
                .map_err(|e| {
                    RitmError::GuiError(GuiError::FileError(format!("Could not load file {e}",)))
                })?
                .to_string(),
        );
    }

    if let Some(code) = &app.transient.code {
        let code = code.clone();
        ui.add_ui(item(), |ui| {
            if let Some(answer) = boolean_popup(ui, app, "Do you want to create a new tab ?")? {
                if answer {
                    app.code.new_tab(app.code.tab_name(), code);
                } else {
                    *app.code.current_code_mut()? = code;
                }
                app.transient.code = None;
            }
            Ok::<(), RitmError>(())
        });
    }
    Ok(())
}

fn help(app: &mut App, ui: &mut FlexInstance) {
    if ui
        .add(
            item(),
            ImageButton::new(
                Image::new(include_image!("../../assets/icon/help.svg"))
                    .fit_to_exact_size(Vec2::splat(Constant::ICON_SIZE))
                    .tint(app.theme.icon),
            )
            .frame(false),
        )
        .clicked()
    {
        app.popup.switch_to(RitmPopupEnum::Help);
    }
}

fn to_graph(app: &mut App, ui: &mut FlexInstance) -> Result<(), RitmError> {
    if ui
        .add(
            item(),
            ImageButton::new(
                Image::new(include_image!("../../assets/icon/graph.svg"))
                    .fit_to_exact_size(Vec2::splat(Constant::ICON_SIZE))
                    .tint(app.theme.icon),
            )
            .frame(false),
        )
        .clicked()
    {
        app.code_to_graph()?;
    }

    Ok(())
}

fn panel_close(app: &mut App, ui: &mut FlexInstance) {
    if ui
        .add(
            item(),
            ImageButton::new(
                Image::new(include_image!("../../assets/icon/panel_close.svg"))
                    .fit_to_exact_size(Vec2::splat(Constant::ICON_SIZE))
                    .tint(app.theme.icon),
            )
            .frame(false),
        )
        .clicked()
    {
        app.code.close();
    }
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
