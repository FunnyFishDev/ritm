use egui::{
    Align, Align2, Color32, Frame, Label, Layout, Margin, Rect, Response, RichText, ScrollArea,
    Sense, Stroke, StrokeKind, Ui, Vec2,
    epaint::PathShape,
    pos2,
    scroll_area::{ScrollBarVisibility, ScrollSource},
    vec2,
};

use crate::{
    App,
    ui::{
        constant::Constant,
        tutorial::TutorialBox,
        utils::{self, fade::Fade},
    },
};

pub fn show(app: &mut App, ui: &mut Ui) {
    let tape_count = app.turing.tm.graph_ref().get_k() + 1;

    // Apply a scale correction to element for small screen
    let square_size = Constant::scale(ui, Constant::SQUARE_SIZE);
    let horizontal_space = Constant::scale(ui, Constant::HORIZONTAL_SPACING);
    let vertical_space = Constant::scale(ui, Constant::VERTICAL_SPACING);
    let scale = Constant::scale(ui, 1.0);

    // Tapes frame
    let res = Frame::new()
        .inner_margin(Margin::same(3))
        .outer_margin(Margin::same(0))
        .fill(app.theme.primary)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = (0.0, vertical_space).into();

            // Get the absolute center of the ribbons layout
            let center = ui.available_rect_before_wrap().left() + ui.available_width() / 2.0;
            // Compute how many square will be visible
            let mut square_count = ((ui.available_width() + horizontal_space)
                / (horizontal_space + square_size)) as usize;

            // Ensure the count is odd because there is always a square centered
            if square_count.is_multiple_of(2) {
                square_count += 1
            }

            // Compute the final width of the ribbons
            let ribbon_width =
                square_count as f32 * (square_size + horizontal_space) - horizontal_space;

            // ui.set_height(ui.ctx().screen_rect().height());
            let content = ScrollArea::vertical()
                .scroll_bar_visibility(ScrollBarVisibility::VisibleWhenNeeded)
                .auto_shrink(true)
                .min_scrolled_height(ui.ctx().screen_rect().height() / 3.0)
                .max_height(ui.ctx().screen_rect().height() / 3.0)
                .show(ui, |ui| {
                    // Scroll area to center and display the ribbon
                    ScrollArea::horizontal()
                        .scroll_source(ScrollSource::NONE)
                        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                        .horizontal_scroll_offset(
                            3.0 // 3.0 is the margin of the center square
                        + square_size
                        + horizontal_space
                        + (ribbon_width - ui.available_width()) / 2.0,
                        ) // this offset center the symbol
                        .show(ui, |ui| {
                            let width = ui.available_width();
                            // Draw each ribbon
                            let mut write_rect = Rect::ZERO;
                            let mut center_rect = Rect::ZERO;
                            for i in 0..tape_count {
                                // Get the top of the current ribbon to draw the arrow
                                let top = ui.available_rect_before_wrap().top();

                                // Draw the ribbon
                                let res = tape(app, ui, width, i);

                                if i == 0 {
                                    app.tutorial.add_boxe(
                                        "reading_tape",
                                        TutorialBox::new(res.interact_rect)
                                            .with_align(Align2::CENTER_BOTTOM),
                                    );
                                } else if write_rect == Rect::ZERO {
                                    write_rect = Rect::from_min_size(
                                        res.interact_rect.min,
                                        res.interact_rect.size(),
                                    )
                                } else {
                                    write_rect = Rect::from_min_max(
                                        write_rect.min,
                                        write_rect.max + vec2(0.0, res.interact_rect.height()),
                                    )
                                }

                                if center_rect == Rect::ZERO {
                                    center_rect = Rect::from_min_size(
                                        pos2(
                                            res.interact_rect.center().x - 3.0 - square_size / 2.0,
                                            res.interact_rect.min.y,
                                        ),
                                        Vec2::splat(square_size + 6.0),
                                    )
                                } else {
                                    center_rect = Rect::from_min_max(
                                        center_rect.min,
                                        center_rect.max
                                            + vec2(
                                                0.0,
                                                res.interact_rect.height() + vertical_space,
                                            ),
                                    )
                                }

                                // Draw the arrow on top of the ribbon
                                ui.painter().add(PathShape::convex_polygon(
                                    vec![
                                        (center - 9.0 * scale, top).into(),
                                        (center + 9.0 * scale, top).into(),
                                        (center, top + 12.0 * scale).into(),
                                    ],
                                    app.theme.border,
                                    Stroke::NONE,
                                ));
                            }

                            app.tutorial.add_boxe(
                                "writing_tape",
                                TutorialBox::new(write_rect).with_align(Align2::CENTER_BOTTOM),
                            );

                            app.tutorial.add_boxe(
                                "current_character",
                                TutorialBox::new(center_rect).with_align(Align2::CENTER_BOTTOM),
                            );
                        });
                });

            (
                content.content_size.y,
                content.state.offset.y >= content.content_size.y - content.inner_rect.height(),
            )
        });

    if !res.inner.1 && res.inner.0 >= ui.ctx().screen_rect().height() / 3.0 {
        let fade_rect = Rect::from_min_max(
            pos2(res.response.rect.min.x, res.response.rect.max.y - 50.0),
            res.response.rect.max,
        );
        utils::fade::fade(
            ui,
            fade_rect,
            egui::Direction::BottomUp,
            Fade::new()
                .with_color(app.theme.primary, 0.0)
                .with_color(Color32::TRANSPARENT, 1.0)
                .with_step(50),
        );
    }

    app.tutorial.add_boxe(
        "tape_section",
        TutorialBox::new(res.response.rect).with_align(Align2::CENTER_BOTTOM),
    );
}

/// Draw a ribbon with the correct spacing and character
fn tape(app: &mut App, ui: &mut Ui, width: f32, tape_id: usize) -> Response {
    // Apply a scale correction to element for small screen
    let horizontal_space = Constant::scale(ui, Constant::HORIZONTAL_SPACING);
    let square_size = Constant::scale(ui, Constant::SQUARE_SIZE);

    ui.allocate_ui_with_layout(
        vec2(0.0, square_size + 6.0),
        Layout::left_to_right(Align::Center)
            .with_cross_justify(false)
            .with_cross_align(Align::Center),
        |ui| {
            ui.style_mut().spacing.item_spacing = (horizontal_space, 0.0).into();

            let square_count: usize =
                ((width + horizontal_space) / (horizontal_space + square_size)) as usize + 2;

            // Get the chars and pointer from reading or writing ribbon
            let tape = &app.turing.current_step.get_tapes()[tape_id];
            let (chars, pointer): (&Vec<char>, i32) =
                (tape.get_contents(), tape.get_pointer() as i32);

            // Create a vector with the character that are needed
            let tape_center = square_count as i32 / 2;
            let mut tape_vec = vec![' '; (tape_center - pointer).max(0) as usize];
            tape_vec.append(
                &mut chars[(pointer - tape_center).max(0) as usize
                    ..(pointer + tape_center + 1).min(chars.len() as i32) as usize]
                    .to_vec(),
            );

            tape_vec.append(&mut vec![
                ' ';
                (tape_center - (chars.len() as i32 - pointer - 1)).max(0)
                    as usize
            ]);

            for (i, char) in tape_vec.iter().enumerate().take(square_count) {
                square(app, ui, *char, i == tape_center as usize);
            }
        },
    )
    .response
}

/// Draw a single square with a character
fn square(app: &mut App, ui: &mut Ui, character: char, is_current: bool) {
    // Apply a scale correction to element for small screen
    let square_size = Constant::scale(ui, Constant::SQUARE_SIZE);

    Frame::new().show(ui, |ui| {
        let size = square_size + if is_current { 6.0 } else { 0.0 };
        let (rect, _res) = ui.allocate_exact_size(Vec2::splat(size), Sense::empty());

        // Draw the square, with a border if center one
        ui.painter().rect(
            rect,
            Constant::scale(ui, Constant::SQUARE_CORNER),
            app.theme.surface,
            if is_current {
                Stroke::new(Constant::scale(ui, 3.0), app.theme.border)
            } else {
                Stroke::NONE
            },
            StrokeKind::Inside,
        );

        // Add the character into the frame
        ui.put(
            rect,
            Label::new(
                RichText::new(character)
                    .size(square_size / 2.0)
                    .color(app.theme.text_primary),
            ),
        );
    });
}
