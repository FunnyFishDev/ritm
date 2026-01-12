use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

use egui::{FontData, FontDefinitions, FontFamily, Key, Rect, Ui, UserData, ViewportCommand};
use egui_extras::install_image_loaders;
use image::{ExtendedColorType, save_buffer};
use ritm_core::{
    turing_machine::Mode,
    turing_parser::{graph_to_string, parse_turing_graph_string},
};

use crate::{
    error::RitmError, turing::{State, StateWrapper, TransitionId, TransitionWrapper, Turing}, ui::{self, popup::RitmPopup, theme::Theme, utils::FileDialog}
};

/// The only structure that is persistent each redraw of the application
pub struct App {
    /// The turing machine itself
    pub turing: Turing,

    /// User input for the turing machine
    pub input: String,

    /// Used for graph display, zooming and moving
    pub graph_rect: Rect,

    /// The code used to create the turing machine
    pub code: String,

    /// The event/state of the application
    pub event: Event,

    /// Current theme
    pub theme: Theme,

    /// Selected state
    pub selected_state: Option<usize>,

    /// Selected transition
    pub selected_transition: Option<TransitionId>,

    /// Interval between each iteration
    pub interval: i32,

    /// File loaded
    pub file: FileDialog,

    /// Which popup to display
    pub popup: RitmPopup,

    pub last_step_time: f64,

    pub settings: Settings,

    pub help_slide_index: usize,

    pub temp_state: Option<State>,
}

/// Keep the state of the application
///
/// Used to check what the user see and/or can do
pub struct Event {
    /// Is the user adding a transition ?
    pub is_adding_transition: bool,

    /// Is the user adding a state ?
    pub is_adding_state: bool,

    /// Is the machine running ?
    pub is_running: bool,

    /// Is the input accepted ? None if result is not computed
    pub is_accepted: Option<bool>,

    /// Is the graph stable ?
    pub is_stable: bool,

    /// Is the user moving as state around ?
    pub is_dragging: bool,

    /// Has the Graph changed ?
    pub has_changed: bool,

    /// Do we need to go to the next iteration ?
    pub is_next: Arc<AtomicBool>,

    /// Do we need to recenter the graph ?
    pub need_recenter: bool,

    /// Do we need to display the settings interface ?
    pub are_settings_visible: bool,

    /// Is the code section closed ?
    pub is_code_closed: bool,

    pub is_small_window: bool,

    pub close_popup: bool,

    pub listen_to_keybind: bool,

    pub take_screenshot: bool,
}

pub struct Settings {
    pub turing_machine_mode: Mode,

    pub toggle_after_action: bool,
}

impl Default for App {
    fn default() -> Self {
        let mut sf = Self {
            turing: Turing::default(),
            input: "".to_string(),
            graph_rect: Rect::ZERO,
            code: "".to_string(), // TODO display a message as comment instead
            event: Event::default(),
            theme: Theme::DEFAULT,
            selected_state: None,
            selected_transition: None,
            interval: 0,
            file: FileDialog::default(),
            popup: RitmPopup::None,
            last_step_time: 0.0,
            settings: Settings {
                toggle_after_action: true,
                turing_machine_mode: Mode::StopAfter(500),
            },
            help_slide_index: 0,
            temp_state: None,
        };

        sf.turing.layer_graph();

        sf
    }
}

impl Default for Event {
    fn default() -> Self {
        Self {
            is_accepted: None,
            is_adding_transition: false,
            is_adding_state: false,
            is_running: false,
            is_stable: false,
            is_dragging: false,
            has_changed: false,
            is_next: AtomicBool::new(false).into(),
            need_recenter: false,
            are_settings_visible: false,
            is_code_closed: false,
            is_small_window: false,
            close_popup: false,
            listen_to_keybind: true,
            take_screenshot: false,
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // cc.egui_ctx.set_debug_on_hover(true);

        // Load the fonts used in the application
        load_font(cc);

        let app: App = Default::default();

        Theme::set_global_theme(&app.theme, &cc.egui_ctx);
        app
    }

    /// The currently selected state
    pub fn selected_state(&self) -> Result<usize, RitmError> {
        self.selected_state.ok_or(RitmError::GuiError("No state selected".to_string()))
    }

    /// The currently selected transitions
    pub fn selected_transitions(&self) -> Result<TransitionId, RitmError> {
        self.selected_transition.ok_or(RitmError::GuiError("No transitions selected".to_string()))
    }

    /// Reset the machine execution
    pub fn reset(&mut self) {
        self.turing.reset();
    }

    /// Reset the machine execution with the new input
    /// TODO: stop ignoring result to avoid cloudflare global shutdown
    pub fn set_input(&mut self) {
        let _ = self.turing.tm.reset_word(&self.input);
        self.turing.reset();
    }

    pub fn graph_to_code(&mut self) {
        self.code = graph_to_string(self.turing.tm.graph_ref());
    }

    pub fn code_to_graph(&mut self) {
        match parse_turing_graph_string(self.code.to_string()) {
            Ok(graph) => {
                self.turing = Turing::new_graph(graph);
                self.turing.layer_graph();
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
        self.event.need_recenter = true;
    }
}

/// Update loop
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        install_image_loaders(ctx);

        if let Err(res) = ui::show(self, ctx) {
            println!("Error ! {}", res)
        }

        if self.event.is_running
            && ctx.input(|r| r.time) - self.last_step_time >= 2.0_f32.powi(self.interval) as f64
        {
            self.turing.next_step();
            self.last_step_time = ctx.input(|r| r.time);
        }

        ctx.input(|r| {
            if r.key_pressed(Key::Escape) {
                if self.popup != RitmPopup::None {
                    // Request graceful exit of popup
                    self.event.close_popup = true;
                } else {
                    // Unselect what is selected
                    self.selected_state = None;
                    self.selected_transition = None;
                }
            }
        });

        if self.event.listen_to_keybind && self.popup == RitmPopup::None {
            ctx.input(|r| {
                // Press A to create a state
                if r.key_pressed(Key::A) {
                    self.event.is_adding_state ^= true;
                }

                // Press T to create a transition
                if self.selected_state.is_some() && r.key_pressed(Key::T) {
                    self.event.is_adding_transition ^= true;
                }

                // Press U to unpin all state
                if r.key_pressed(Key::U) {
                    self.turing.unpin();
                }

                // Press C to open and close code section
                if r.key_pressed(Key::C) {
                    self.event.is_code_closed ^= true;
                }

                // Press R to recenter
                if r.key_pressed(Key::R) {
                    self.event.need_recenter = true;
                }

                // Press Space to make 1 iteration
                if self.event.is_accepted.is_none() && r.key_pressed(Key::Space) {
                    self.turing.next_step();
                }

                // Press P to autoplay the machine
                if r.key_pressed(Key::P) {
                    self.event.is_running ^= true;
                }

                // Press Backspace to reset the machine
                if r.key_pressed(Key::Backspace) {
                    self.reset();
                }

                if r.key_pressed(Key::S) {
                    self.event.take_screenshot = true;
                }
            });
        } else {
            self.event.listen_to_keybind = true;
        }

        ctx.request_repaint_after(Duration::from_secs(2.0_f32.powi(self.interval) as u64));
    }
}

/// Load the necessary font for the application
fn load_font(cc: &eframe::CreationContext<'_>) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "RobotoMono-regular".into(),
        FontData::from_static(include_bytes!("../assets/fonts/RobotoMono-Regular.ttf")).into(),
    );
    fonts.font_data.insert(
        "RobotoMono-Bold".into(),
        FontData::from_static(include_bytes!("../assets/fonts/RobotoMono-Bold.ttf")).into(),
    );

    let mut newfam = BTreeMap::new();

    newfam.insert(
        FontFamily::Name("RobotoMono-Bold".into()),
        vec!["RobotoMono-Bold".to_owned()],
    );
    newfam.insert(
        FontFamily::Name("RobotoMono-regular".into()),
        vec!["RobotoMono-regular".to_owned()],
    );
    fonts.families.append(&mut newfam);

    cc.egui_ctx.set_fonts(fonts);
}

pub fn take_screenshot(app: &mut App, ui: &mut Ui) {
    let ctx = ui.ctx();
    let rect = ui.min_rect();
    if app.event.take_screenshot {
        app.event.take_screenshot = false;

        ctx.send_viewport_cmd(ViewportCommand::Screenshot(UserData::default()));

        ctx.input(|i| {
            i.events.iter().for_each(|e| {
                if let egui::Event::Screenshot { image, .. } = e {
                    let image = image.region(&rect, Some(i.pixels_per_point));
                    save_buffer(
                        Path::new("assets/help/screenshot.png"),
                        image.as_raw(),
                        image.source_size.x as u32,
                        image.source_size.y as u32,
                        ExtendedColorType::Rgba8,
                    )
                    .unwrap();
                }
            })
        });
    }
}
