use egui::{
    Align, AtomExt, Button, Color32, Frame, Image, ImageButton, Key, Label, Layout, Margin,
    RichText, ScrollArea, Shadow, Stroke, TextEdit, Ui, Vec2, Vec2b, include_image,
    scroll_area::ScrollBarVisibility, style::WidgetVisuals, vec2,
};
use ritm_core::turing_transition::{
    TransitionMultRibbonInfo, TransitionOneRibbonInfo, TransitionsInfo, TuringDirection,
};

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
                        ui.spacing_mut().scroll.floating = false;
                        ui.spacing_mut().scroll.bar_width = 3.0;
                        ScrollArea::vertical()
                            .auto_shrink(Vec2b::new(true, true))
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
                                    if transition(
                                        app,
                                        ui,
                                        transition_index,
                                        transition_index == count - 1
                                            && app.transient.add_transition,
                                    )? {
                                        marked_to_delete.push(transition_index);
                                    }
                                }

                                app.transient.add_transition = false;

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
                            info: if k == 0 {
                                TransitionsInfo::OneTape(TransitionOneRibbonInfo::default())
                            } else {
                                TransitionsInfo::MultipleTapes(
                                    TransitionMultRibbonInfo::create_default(k),
                                )
                            },
                            inner_transition: Transition::new(),
                        }),
                        None,
                    ));

                    app.transient.add_transition = true;
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
                || columns[0].ctx().input(|r| r.key_pressed(Key::Escape))
            {
                app.popup.close();
                app.turing.cancel_transition_change();
            };

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
                || columns[0].ctx().input(|r| r.key_pressed(Key::Enter))
            {
                if let Err(reason) = app.turing.apply_transition_change() {
                    return Err(RitmError::GuiError(GuiError::InvalidTransition {
                        reason: reason.to_string(),
                    }));
                } else {
                    app.popup.close();
                }
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
fn transition(
    app: &mut App,
    ui: &mut Ui,
    transition_index: usize,
    scroll: bool,
) -> Result<bool, RitmError> {
    let mut marked_to_delete = false;
    let error = &app.turing.get_transitions_edit()?.1[transition_index].1;
    let frame = Frame::new()
        .fill(app.theme.surface)
        .inner_margin(Margin::symmetric(10, 6))
        .corner_radius(5)
        .stroke(if error.is_some() {
            Stroke::new(2.0, app.theme.error)
        } else {
            Stroke::NONE
        })
        .show(ui, |ui| {
            ui.allocate_ui_with_layout(
                vec2(ui.available_width(), 30.0),
                Layout::right_to_left(egui::Align::Center),
                |ui| {
                    // Delete
                    if ui
                        .add(
                            ImageButton::new(
                                Image::new(include_image!("../../../assets/icon/delete.svg"))
                                    .fit_to_exact_size(vec2(30.0, 30.0))
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
                                    .fit_to_exact_size(vec2(30.0, 30.0))
                                    .tint(
                                        if selected_transition[transition_index].0.has_changed() {
                                            app.theme.overlay
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

                    // Layout single character TextEdit
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.spacing_mut().item_spacing = vec2(5.0, 0.0);
                        ui.set_height(
                            Font::get_heigth(ui, &Font::default_medium()) + MARGIN.y * 2.0,
                        );

                        // 2 cases : single tape or more than one tape
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
                                    &mut transition.replace_with,
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
                                    format!("{transition_index}-{}", 0),
                                );
                            }
                            TransitionsInfo::MultipleTapes(transition) => {
                                move_read(
                                    ui,
                                    &mut transition.chars_read[0],
                                    &mut transition.move_read,
                                    format!("{transition_index}-{}", transition.chars_write.len()),
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

                                    move_write(
                                        ui,
                                        &mut transition.chars_write[i],
                                        format!("{transition_index}-{}", i),
                                    );

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
        })
        .response;
    if scroll {
        frame.scroll_to_me(Some(Align::Max));
    }
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
        let before = *chars_read;
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
                *chars_read = text
                    .chars()
                    .nth(if before == text.chars().next().expect("should exist") {
                        1
                    } else {
                        0
                    })
                    .expect("Char should exist");
            } else if text.is_empty() {
                *chars_read = '\0';
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

/// Display the write text edit (wrapper because borrowing)
fn write_edit(ui: &mut Ui, chars_write: &mut (char, TuringDirection), visual: WidgetVisuals) {
    read_edit(ui, &mut chars_write.0, &mut chars_write.1, visual);
}

/// Display combobox
fn move_read(ui: &mut Ui, chars_read: &mut char, move_read: &mut TuringDirection, i: String) {
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
fn move_write(ui: &mut Ui, chars_write: &mut (char, TuringDirection), i: String) {
    move_read(ui, &mut chars_write.0, &mut chars_write.1, i);
}
