use egui::{
    Align, AtomExt, Button, Color32, Frame, Image, ImageButton, Label, Layout, Margin, RichText,
    ScrollArea, Shadow, Stroke, TextEdit, Ui, Vec2, Vec2b, include_image,
    scroll_area::ScrollBarVisibility, style::WidgetVisuals, vec2,
};
use ritm_core::turing_transition::{TransitionMultRibbonInfo, TransitionsInfo, TuringDirection};

use crate::{
    App,
    error::{GuiError, RitmError},
    turing::{Transition, TransitionEdit, TransitionWrapper},
    ui::{component::combobox::ComboBox, font::Font, theme::Theme},
};

pub fn show(ui: &mut Ui, app: &mut App) -> Result<(), RitmError> {
    // Main layout
    ui.vertical_centered(|ui| {
        ui.style_mut().spacing.item_spacing = vec2(0.0, 10.0);

        // List of the rule
        let width = ui
            .vertical_centered(|ui| {
                ui.style_mut().spacing.item_spacing = vec2(0.0, 10.0);

                Frame::new()
                    .fill(Color32::LIGHT_GRAY)
                    .shadow(Shadow {
                        blur: 0,
                        color: app.theme.shadow,
                        offset: [0, 2],
                        spread: 0,
                    })
                    .inner_margin(10)
                    .corner_radius(5)
                    .show(ui, |ui| {
                        ScrollArea::vertical()
                            .auto_shrink(Vec2b::new(true, false))
                            .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
                            .max_height(ui.ctx().input(|i| i.screen_rect()).height() / 3.0)
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());

                                let selected_transition =
                                    &mut app.turing.get_transitions_edit_mut()?.1;

                                // Create a row with the rule of each transition
                                let count = selected_transition.len();
                                let mut marked_to_delete: Vec<usize> = vec![];
                                for transition_index in 0..count {
                                    if transition(app, ui, transition_index)? {
                                        marked_to_delete.push(transition_index);
                                    }
                                }

                                // Reborrow because the transition() above borrow app
                                let selected_transition =
                                    &mut app.turing.get_transitions_edit_mut()?.1;

                                // Remove transitions
                                marked_to_delete.sort_by(|a, b| b.cmp(a));
                                for t in marked_to_delete {
                                    selected_transition.remove(t);
                                }
                                Ok::<(), RitmError>(())
                            });
                    });

                if ui
                    .add(
                        ImageButton::new(
                            Image::new(include_image!("../../../assets/icon/plus.svg"))
                                .fit_to_exact_size(vec2(35.0, 35.0))
                                .tint(app.theme.overlay),
                        )
                        .frame(false),
                    )
                    .clicked()
                {
                    let k = app.turing.tm.graph_ref().get_k();
                    let selected_transition = &mut app.turing.get_transitions_edit_mut()?.1;
                    selected_transition.push((
                        TransitionEdit::from(&TransitionWrapper {
                            info: TransitionsInfo::MultipleTapes(
                                TransitionMultRibbonInfo::create_default(k),
                            ),
                            inner_transition: Transition::new(),
                        }),
                        None,
                    ));
                }
                Ok::<(), RitmError>(())
            })
            .response
            .rect
            .width();

        ui.set_width(width);

        ui.spacing_mut().button_padding = vec2(0.0, 8.0);
        ui.spacing_mut().item_spacing = vec2(10.0, 0.0);
        ui.columns(2, |columns| {
            let text = RichText::new("Save")
                .color(Theme::constrast_color(app.theme.success))
                .font(Font::default_medium())
                .atom_grow(true);
            if columns[0]
                .add(
                    Button::new(text)
                        .stroke(Stroke::new(2.0, app.theme.border))
                        .fill(app.theme.success)
                        .corner_radius(10.0),
                )
                .clicked()
            {
                let x = app.turing.apply_transition_change()?;
                if x.iter().any(|t| t.is_err()) {
                    return Err(RitmError::GuiError(GuiError::InvalidTransition {
                        reason: x
                            .iter()
                            .filter_map(|f| match f {
                                Ok(_) => None,
                                Err(err) => Some(err.to_string()),
                            })
                            .collect(),
                    }));
                } else {
                    app.popup.close();
                }
            };

            let text = RichText::new("Cancel")
                .color(Theme::constrast_color(app.theme.error))
                .font(Font::default_medium())
                .atom_grow(true);
            if columns[1]
                .add(
                    Button::new(text)
                        .stroke(Stroke::new(2.0, app.theme.border))
                        .fill(app.theme.error)
                        .corner_radius(10.0),
                )
                .clicked()
            {
                app.popup.close();
                app.turing.cancel_transition_change();
            };

            Ok::<(), RitmError>(())
        })?;

        Ok::<(), RitmError>(())
    })
    .inner?;

    Ok(())
}

const MARGIN: Vec2 = vec2(3.0, 2.0);

// Right to left to allow the text edit to take the remaining space
// To remove later with a system based on the number of ribbons
fn transition(app: &mut App, ui: &mut Ui, transition_index: usize) -> Result<bool, RitmError> {
    let mut marked_to_delete = false;
    Frame::new()
        .fill(app.theme.surface)
        .inner_margin(Margin::symmetric(5, 3))
        .corner_radius(5)
        .show(ui, |ui| {
            ui.allocate_ui_with_layout(
                vec2(ui.available_width(), 10.0),
                Layout::right_to_left(egui::Align::Center),
                |ui| {
                    // Delete
                    if ui
                        .add(
                            ImageButton::new(
                                Image::new(include_image!("../../../assets/icon/delete.svg"))
                                    .fit_to_exact_size(vec2(35.0, 35.0))
                                    .tint(app.theme.error),
                            )
                            .frame(false),
                        )
                        .clicked()
                    {
                        marked_to_delete = true;
                    }

                    let (_, selected_transition) = app.turing.get_transitions_edit_mut()?;

                    // Undo change
                    if ui
                        .add(
                            ImageButton::new(
                                Image::new(include_image!("../../../assets/icon/undo.svg"))
                                    .fit_to_exact_size(vec2(35.0, 35.0))
                                    .tint(
                                        if selected_transition[transition_index].0.has_changed() {
                                            app.theme.icon
                                        } else {
                                            app.theme.disabled
                                        },
                                    ),
                            )
                            .frame(false),
                        )
                        .clicked()
                    {
                        // Undo all changes
                        for transition in selected_transition {
                            transition.0.undo();
                        }
                    }
                    // Combobox use global variable
                    ui.spacing_mut().button_padding = MARGIN;
                    ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::LIGHT_GRAY;
                    ui.visuals_mut().widgets.active.weak_bg_fill =
                        ui.visuals_mut().widgets.inactive.weak_bg_fill;
                    ui.visuals_mut().widgets.hovered.weak_bg_fill =
                        ui.visuals_mut().widgets.inactive.weak_bg_fill;
                    ui.visuals_mut().widgets.open.weak_bg_fill =
                        ui.visuals_mut().widgets.inactive.weak_bg_fill;

                    // Make access easier

                    // Layout single character TextEdit
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.spacing_mut().item_spacing = vec2(5.0, 0.0);
                        ui.set_height(
                            Font::get_heigth(ui, &Font::default_medium()) + MARGIN.y * 2.0,
                        );

                        let transition = app.turing.get_transition_edit_mut(transition_index)?;
                        match &mut transition.info {
                            TransitionsInfo::OneTape(transition) => {
                                read_edit(
                                    ui,
                                    &mut transition.chars_read,
                                    &mut transition.move_pointer,
                                    WidgetVisuals {
                                        bg_stroke: Stroke::new(1.0, app.theme.error),
                                        ..app.theme.default_widget()
                                    },
                                );
                            }
                            TransitionsInfo::MultipleTapes(transition) => {
                                // Textedit for each reading character
                                for i in 0..transition.chars_read.len() {
                                    read_edit(
                                        ui,
                                        &mut transition.chars_read[i],
                                        &mut transition.move_read,
                                        WidgetVisuals {
                                            bg_stroke: Stroke::new(1.0, app.theme.error),
                                            ..app.theme.default_widget()
                                        },
                                    );

                                    // Aesthetic purpose, add a colon between each reading char
                                    if i != transition.chars_read.len() - 1 {
                                        ui.add(Label::new(","));
                                    }
                                }
                            }
                        }

                        // Aesthetic purpose, add an arrow between the "condition" and the "action"
                        ui.add(Label::new("->"));

                        match &mut transition.info {
                            TransitionsInfo::OneTape(transition) => {
                                read_edit(
                                    ui,
                                    &mut transition.chars_read,
                                    &mut transition.move_pointer,
                                    WidgetVisuals {
                                        bg_stroke: Stroke::new(1.0, app.theme.error),
                                        ..app.theme.default_widget()
                                    },
                                );

                                // Again, aesthetic purpose
                                ui.add(Label::new(","));

                                move_read(
                                    ui,
                                    &mut transition.replace_with,
                                    &mut transition.move_pointer,
                                    0,
                                );
                            }
                            TransitionsInfo::MultipleTapes(transition) => {
                                move_read(
                                    ui,
                                    &mut transition.chars_read[0],
                                    &mut transition.move_read,
                                    transition.chars_write.len(),
                                );

                                // Again, aesthetic purpose
                                ui.add(Label::new(","));

                                // Textedit for each reading character
                                for i in 0..transition.chars_write.len() {
                                    // TextEdit don't accept char, so must use a String

                                    write_edit(
                                        ui,
                                        &mut transition.chars_write[i],
                                        WidgetVisuals {
                                            bg_stroke: Stroke::new(1.0, app.theme.error),
                                            ..app.theme.default_widget()
                                        },
                                    );

                                    // Again, aesthetic purpose
                                    ui.add(Label::new(","));

                                    move_write(ui, &mut transition.chars_write[i], i);

                                    // Again, aesthetic purpose
                                    if i != transition.chars_write.len() - 1 {
                                        ui.add(Label::new(","));
                                    }
                                }
                            }
                        };
                        Ok::<(), RitmError>(())
                    })
                    .inner
                },
            );
        });
    Ok(marked_to_delete)
}

/// Display the read text edit
fn read_edit(
    ui: &mut Ui,
    chars_read: &mut char,
    move_read: &mut TuringDirection,
    visual: WidgetVisuals,
) {
    // TextEdit don't accept char, so must use a String
    ui.scope(|ui| {
        ui.visuals_mut().selection.stroke = Stroke::NONE;
        if *chars_read == '\0' {
            Theme::set_widget(ui, visual);
        }
        let mut text = chars_read.to_string();
        if ui
            .add(
                TextEdit::singleline(&mut text)
                    .background_color(Color32::LIGHT_GRAY)
                    .lock_focus(false)
                    .frame(true)
                    .font(Font::default_medium())
                    .margin(MARGIN)
                    .desired_width(Font::get_width(ui, &Font::default_medium()))
                    .char_limit(2),
            )
            .changed()
        {
            if text.char_indices().count() >= 2 {
                *chars_read = text.chars().nth(1).expect("Char should exist");
            } else if text.is_empty() {
                *chars_read = 'ç';
            }

            match chars_read {
                '$' => {
                    if *move_read == TuringDirection::Right {
                        *move_read = TuringDirection::None;
                    }
                }
                'ç' => {
                    if *move_read == TuringDirection::Left {
                        *move_read = TuringDirection::None;
                    }
                }
                _ => {}
            }
        }
    });
}

/// Display the read text edit
fn write_edit(ui: &mut Ui, chars_write: &mut (char, TuringDirection), visual: WidgetVisuals) {
    // TextEdit don't accept char, so must use a String
    ui.scope(|ui| {
        ui.visuals_mut().selection.stroke = Stroke::NONE;
        if chars_write.0 == '\0' {
            Theme::set_widget(ui, visual);
        }
        let mut text = chars_write.0.to_string();
        if ui
            .add(
                TextEdit::singleline(&mut text)
                    .background_color(Color32::LIGHT_GRAY)
                    .lock_focus(false)
                    .frame(true)
                    .font(Font::default_medium())
                    .margin(MARGIN)
                    .desired_width(Font::get_width(ui, &Font::default_medium()))
                    .char_limit(2),
            )
            .changed()
        {
            if text.char_indices().count() >= 2 {
                chars_write.0 = text.chars().nth(1).expect("Char should exist");
            } else if text.is_empty() {
                chars_write.0 = 'ç';
            }

            match chars_write.0 {
                '$' => {
                    if chars_write.1 == TuringDirection::Right {
                        chars_write.1 = TuringDirection::None;
                    }
                }
                'ç' => {
                    if chars_write.1 == TuringDirection::Left {
                        chars_write.1 = TuringDirection::None;
                    }
                }
                _ => {}
            }
        }
    });
}

/// Display combobox
fn move_read(ui: &mut Ui, chars_read: &mut char, move_read: &mut TuringDirection, i: usize) {
    // Reading ribbon moving direction
    ComboBox::from_id_salt(format!("moveread-{}", i))
        .selected_text(
            RichText::new(match move_read {
                TuringDirection::Left => "L".to_string(),
                TuringDirection::Right => "R".to_string(),
                TuringDirection::None => "N".to_string(),
            })
            .font(Font::default_medium()),
        )
        .width(20.0) // TODO change and think about this value, I hardcoded it
        .show_ui(ui, |ui| {
            if *chars_read != '$' {
                ui.selectable_value(move_read, TuringDirection::Right, "Right");
            }
            if *chars_read != 'ç' {
                ui.selectable_value(move_read, TuringDirection::Left, "Left");
            }
            ui.selectable_value(move_read, TuringDirection::None, "None");
        });
}

/// Display combobox
fn move_write(ui: &mut Ui, chars_write: &mut (char, TuringDirection), i: usize) {
    // Reading ribbon moving direction
    ComboBox::from_id_salt(format!("moveread-{}", i))
        .selected_text(
            RichText::new(match chars_write.1 {
                TuringDirection::Left => "L".to_string(),
                TuringDirection::Right => "R".to_string(),
                TuringDirection::None => "N".to_string(),
            })
            .font(Font::default_medium()),
        )
        .width(20.0) // TODO change and think about this value, I hardcoded it
        .show_ui(ui, |ui| {
            if chars_write.0 != '$' {
                ui.selectable_value(&mut chars_write.1, TuringDirection::Right, "Right");
            }
            if chars_write.0 != 'ç' {
                ui.selectable_value(&mut chars_write.1, TuringDirection::Left, "Left");
            }
            ui.selectable_value(&mut chars_write.1, TuringDirection::None, "None");
        });
}
