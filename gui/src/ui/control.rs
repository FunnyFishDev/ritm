use egui::{
    Align, Align2, Button, Frame, Image, ImageButton, ImageSource, Label, Layout, Response, RichText, Sense, Stroke, TextEdit, Ui, Vec2, include_image, vec2
};
use egui_flex::{Flex, FlexAlign, FlexAlignContent, FlexInstance, item};

use crate::{
    App,
    error::RitmError,
    ui::{component::grid::Grid, constant::Constant, font::Font},
};

#[derive(Default)]
pub struct Control {
    /// User input for the turing machine
    input: String,
    is_running: bool,
    /// power of 2 interval between each iteration
    interval_power: i32,
    last_step_time: f64,
}


impl Control {
    pub fn run(&mut self) {
        self.is_running = true;
    }

    pub fn pause(&mut self) {
        self.is_running = false;
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn update_time(&mut self, time: f64) -> bool {
        let time_elasped = time - self.last_step_time;
        if time_elasped >= self.interval().into() {
            self.last_step_time = time
        }
        time_elasped >= self.interval().into()
    }

    pub fn interval(&self) -> f32 {
        2.0_f32.powi(self.interval_power)
    }

    pub fn speed_up(&mut self) {
        self.interval_power += 1
    }

    pub fn speed_down(&mut self) {
        self.interval_power -= 1
    }

    pub fn input(&self) -> &String {
        &self.input
    }
}

pub fn show(app: &mut App, ui: &mut Ui) {
    Frame::new().show(ui, |ui| {
        ui.set_height(Constant::scale(ui, 70.0));
        let grid = Grid::new(ui, 2, 3);

        grid.place(ui, 1, 1, |ui| input(app, ui));

        grid.place(ui, 1, 2, |ui| control(app, ui));
        grid.place(ui, 2, 2, |ui| speed_control(app, ui));

        grid.place(ui, 1, 3, |ui| step(app, ui));
        grid.place(ui, 2, 3, |ui| state(app, ui));
    });
}

/// Input section of the controls
///
/// User cna enter the input of the turing machine and submit it
fn input(app: &mut App, ui: &mut Ui) {
    ui.allocate_ui_with_layout(
        ui.available_size(),
        Layout::right_to_left(Align::Center),
        |ui| {
            if ui
                .add(Button::new(
                    RichText::new("Submit")
                        .font(Font::default(Constant::scale(ui, Font::MEDIUM_SIZE))),
                ).stroke(Stroke::new(1.0, app.theme.border)))
                .clicked()
            {
                app.turing.set_word(&app.control.input)?;
            }

            if ui
                .add_sized(
                    vec2(
                        ui.available_width(),
                        4.0 + Font::get_heigth(
                            ui,
                            &Font::default(Constant::scale(ui, Font::MEDIUM_SIZE)),
                        ),
                    ), // 4.0 is 2 times the hardcoded default vertical margin of textedit
                    TextEdit::singleline(&mut app.control.input)
                        .font(Font::default(Constant::scale(ui, Font::MEDIUM_SIZE)))
                        .hint_text(
                            RichText::new("Input...")
                                .font(Font::default(Constant::scale(ui, Font::MEDIUM_SIZE)))
                                .color(app.theme.text_secondary),
                        ).background_color(app.theme.surface),
                )
                .has_focus()
            {
                app.event.listen_to_keybind = false;
            }
            Ok::<(), RitmError>(())
        },
    );
}

/// Control the iteration of the application, automatic or manual
fn control(app: &mut App, ui: &mut Ui) {
    let finished = app.turing.accepted.is_some();
    let initial = app.turing.current_step.get_nb_iterations() == 0 && !finished;

    Flex::horizontal()
        .align_items(FlexAlign::Center)
        .align_content(FlexAlignContent::Center)
        .align_items_content(Align2::CENTER_CENTER)
        .gap(vec2(Constant::scale(ui, 10.0), 0.0))
        .h_full()
        .w_full()
        .show(ui, |flex| {
            flex.grow();
            // If playing
            if app.control.is_running() {
                // Display pause button
                if button(
                    flex,
                    app,
                    include_image!("../../assets/icon/pause.svg"),
                    finished,
                )
                .clicked()
                {
                    app.control.pause();
                }
            } else {
                // Else display play button
                if button(
                    flex,
                    app,
                    include_image!("../../assets/icon/play.svg"),
                    finished,
                )
                .clicked()
                {
                    app.control.run();
                }
            }

            // Next button
            if button(
                flex,
                app,
                include_image!("../../assets/icon/next.svg"),
                finished,
            )
            .clicked()
            {
                app.turing.next_step();
            }

            // Reset button
            if button(
                flex,
                app,
                include_image!("../../assets/icon/reset.svg"),
                initial,
            )
            .clicked()
            {
                app.reset();
            }

            flex.grow();
        });
}

/// Control the speed of the automatic iteration
fn speed_control(app: &mut App, ui: &mut Ui) {
    let min = app.control.interval_power >= 3;
    let max = app.control.interval_power <= -5;

    Flex::horizontal()
        .align_content(FlexAlignContent::Center)
        .align_items_content(Align2::CENTER_CENTER)
        .align_items(FlexAlign::Center)
        .w_full()
        .h_full()
        .show(ui, |flex| {
            flex.grow();
            if flex
                .add(
                    item(),
                    ImageButton::new(
                        Image::new(include_image!("../../assets/icon/less.svg"))
                            .fit_to_exact_size(Vec2::splat(Constant::scale(flex.ui(), 25.0)))
                            .tint(if min { app.theme.disabled } else { app.theme.icon }),
                    )
                    .frame(false),
                )
                .clicked()
                && app.control.interval_power < 3
            {
                app.control.speed_up();
            }

            flex.add(
                item(),
                Label::new(
                    RichText::new(format!("{}X", 1.0/app.control.interval()))
                        .font(Font::default(Constant::scale(flex.ui(), Font::MEDIUM_SIZE)))
                        .color(app.theme.icon),
                ),
            );

            if flex
                .add(
                    item(),
                    ImageButton::new(
                        Image::new(include_image!("../../assets/icon/add.svg"))
                            .fit_to_exact_size(Vec2::splat(Constant::scale(flex.ui(), 25.0)))
                            .tint(if max { app.theme.disabled } else { app.theme.icon }),
                    )
                    .frame(false),
                )
                .clicked()
                && app.control.interval_power > -5
            {
                app.control.speed_down();
            }

            flex.grow()
        });
}

fn step(app: &mut App, ui: &mut Ui) {
    Flex::horizontal()
        .align_content(FlexAlignContent::Center)
        .align_items_content(Align2::CENTER_CENTER)
        .align_items(FlexAlign::Center)
        .w_full()
        .h_full()
        .show(ui, |flex| {
            flex.grow();
            flex.add(
                item(),
                Label::new(
                    RichText::new(format!(
                        "Steps : {}",
                        app.turing.current_step.get_nb_iterations()
                    ))
                    .font(Font::default(Constant::scale(flex.ui(), Font::MEDIUM_SIZE))),
                ),
            );
            flex.grow();
        });
}

fn state(app: &mut App, ui: &mut Ui) {
    Flex::horizontal()
        .align_content(FlexAlignContent::Center)
        .align_items_content(Align2::CENTER_CENTER)
        .align_items(FlexAlign::Center)
        .w_full()
        .h_full()
        .show(ui, |flex| {
            flex.grow();
            let (text, color) = if let Some(r) = app.turing.accepted {
                if r {
                    ("Accepted", app.theme.success)
                } else {
                    ("Rejected", app.theme.error)
                }
            } else if app.control.is_running() {
                ("Running", app.theme.text_primary)
            } else {
                ("Idle", app.theme.text_primary)
            };

            flex.add(
                item(),
                Label::new(
                    RichText::new(text)
                        .color(color)
                        .font(Font::default(Constant::scale(flex.ui(), Font::MEDIUM_SIZE))),
                ),
            );
            flex.grow();
        });
}

fn button(flex: &mut FlexInstance, app: &mut App, icon: ImageSource, disabled: bool) -> Response {
    let icon_size = Vec2::splat(Constant::scale(flex.ui(), Constant::CONTROL_ICON_SIZE));
    flex.add(
        item(),
        ImageButton::new(
            Image::new(icon)
                .fit_to_exact_size(icon_size)
                .tint(if disabled {
                    app.theme.disabled
                } else {
                    app.theme.icon
                }),
        )
        .frame(false)
        .sense(if disabled {
            Sense::empty()
        } else {
            Sense::click()
        }),
    )
}

// #[cfg(not(target_arch = "wasm32"))]
// fn interval(is_next: Arc<AtomicBool>, ctx: Context, duration: Duration) {
//     thread::spawn(move || {
//         thread::sleep(duration);
//         is_next.store(true, Ordering::Relaxed);
//         ctx.request_repaint();
//     });
// }

// #[cfg(target_arch = "wasm32")]
// fn interval(is_next: Arc<AtomicBool>, ctx: Context, duration: Duration) {

//     use wasm_thread as thread;

//     thread::spawn(move || {
//         thread::sleep(duration);
//         is_next.store(true, Ordering::Relaxed);
//         ctx.request_repaint();
//     });
// }
