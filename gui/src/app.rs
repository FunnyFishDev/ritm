use std::time::Duration;

use egui::Key;
use egui_extras::install_image_loaders;
use ritm_core::turing_parser::{graph_to_string, parse_turing_graph_string};

use crate::{
    error::{self, RitmError},
    turing::Turing,
    ui::{
        self,
        code::Code,
        control::Control,
        edit::Edit,
        font::load_font,
        graph::Graph,
        menu::Menu,
        popup::{
            RitmPopup,
            settings::{Settings, debug_show},
        },
        theme::{Theme, theme_changer},
        tutorial::{self, TutorialEnum, Tutorials},
    },
};

/// The only structure that is persistent each redraw of the application
#[derive(serde::Deserialize, serde::Serialize)]
pub struct App {
    /// The turing machine itself
    #[serde(skip)]
    pub turing: Turing,

    #[serde(skip)]
    pub edit: Edit,

    #[serde(skip)]
    pub graph: Graph,

    #[serde(skip)]
    pub control: Control,

    pub settings: Settings,

    #[serde(skip)]
    pub menu: Menu,

    /// Which popup to display
    #[serde(skip)]
    pub popup: RitmPopup,

    /// The code used to create the turing machine
    pub code: Code,

    #[serde(skip)]
    pub error: Option<RitmError>,

    /// The event/state of the application
    #[serde(skip)]
    pub transient: Transient,

    /// Current theme
    pub theme: Theme,

    #[serde(skip)]
    pub help_slide_index: usize,

    pub tutorial: Tutorials,
}

/// Keep the state of the application
///
/// Used to check what the user see and/or can do
pub struct Transient {
    /// Is the user moving as state around ?

    /// Do we need to display the settings interface ?
    pub are_settings_visible: bool,

    pub is_small_window: bool,

    pub listen_to_keybind: bool,

    pub take_screenshot: bool,

    pub temp_code: Option<String>,

    pub temp_tutorial: Option<TutorialEnum>,

    pub add_transition: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            menu: Menu::default(),
            turing: Turing::default(),
            edit: Edit::default(),
            graph: Graph::default(),
            transient: Transient::default(),
            theme: Theme::retro(),
            popup: RitmPopup::default(),
            code: Code::default(),
            help_slide_index: 0,
            control: Control::default(),
            settings: Settings::default(),
            error: None,
            tutorial: Tutorials::default(),
        }
    }
}

impl Default for Transient {
    fn default() -> Self {
        Self {
            are_settings_visible: false,
            is_small_window: false,
            listen_to_keybind: true,
            take_screenshot: false,
            temp_code: None,
            temp_tutorial: None,
            add_transition: false,
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // cc.egui_ctx.set_debug_on_hover(true);

        // Load the fonts used in the application
        load_font(cc);

        let app: App = if let Some(storage) = cc.storage {
            eframe::get_value(storage, "ritm").unwrap_or_default()
        } else {
            Default::default()
        };

        app.theme.as_global_theme(&cc.egui_ctx);
        app
    }

    /// Reset the machine execution
    pub fn reset(&mut self) {
        self.turing.reset();
    }

    /// Reset the machine execution with the new input
    pub fn set_input(&mut self) -> Result<(), RitmError> {
        self.turing.set_word(self.control.input())
    }

    pub fn graph_to_code(&mut self) {
        let code = graph_to_string(self.turing.tm.graph_ref());
        self.code.new_tab(self.code.tab_name(), code);
    }

    /// TODO: handle in case the code is invalid
    pub fn code_to_graph(&mut self) -> Result<(), RitmError> {
        match parse_turing_graph_string(self.code.current_code()?) {
            Ok(graph) => {
                self.turing = Turing::new_graph(graph);
                self.turing.layer_graph();
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
        self.graph.recenter();
        Ok(())
    }
}

/// Update loop
impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "ritm", self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        install_image_loaders(ctx);

        if let Some(tutorial) = self.transient.temp_tutorial {
            self.transient.temp_tutorial = None;
            self.tutorial.start(tutorial);
        }
        if let Err(error) = ui::show(self, ctx)
            && self.error.is_none()
        {
            self.error = Some(error)
        }

        tutorial::show(ctx, self);

        if let Some(tutorial) = self.tutorial.has_finished {
            self.tutorial.has_finished = None;
            self.tutorial
                .already_played
                .entry(tutorial)
                .and_modify(|b| *b = true);
        }

        error::error(ctx.clone(), self);

        if self.control.is_running() && self.control.update_time(ctx.input(|r| r.time)) {
            self.turing.next_step();
            if self.turing.accepted.is_some() {
                self.control.pause();
                ctx.request_repaint(); // To update the ui one last time
            }
        }

        // While the machine is running we update the application 100 times per step
        if self.control.is_running() {
            ctx.request_repaint_after(Duration::from_millis(
                (self.control.interval() * 10.0) as u64,
            ));
        }

        ctx.input(|r| {
            if r.key_pressed(Key::Escape) {
                if self.popup.current().is_some() {
                    // Request graceful exit of popup
                    self.popup.close();
                } else {
                    // Unselect what is selected
                    self.graph.unselect()
                }
            }
        });

        if self.transient.listen_to_keybind && self.popup.current().is_none() {
            ctx.input(|r| {
                if self.tutorial.in_tutorial() {
                    if r.key_pressed(Key::Enter) {
                        self.tutorial.next();
                    }
                    return;
                }

                // Press A to create a state
                if r.key_pressed(Key::A) {
                    self.edit.is_adding_state ^= true;
                }

                // Press T to create a transition
                if self.graph.selected_state().is_some() && r.key_pressed(Key::T) {
                    self.edit.is_adding_transition ^= true;
                }

                // Press U to unpin all state
                if r.key_pressed(Key::U) {
                    self.turing.unpin_all();
                }

                // Press C to open and close code section
                if r.key_pressed(Key::C) {
                    self.code.toggle();
                }

                // Press R to recenter
                if r.key_pressed(Key::R) {
                    self.graph.recenter();
                }

                // Press Space to make 1 iteration
                if self.turing.accepted.is_none() && r.key_pressed(Key::Space) {
                    self.turing.next_step();
                }

                // Press P to autoplay the machine
                if r.key_pressed(Key::P) {
                    self.control.run();
                }

                // Press Backspace to reset the machine
                if r.key_pressed(Key::Backspace) {
                    self.reset();
                }

                if r.key_pressed(Key::S) {
                    self.transient.take_screenshot = true;
                }
            });
        } else {
            self.transient.listen_to_keybind = true;
        }

        if self.settings.theme_changer {
            theme_changer(ctx, self);
        }

        if self.settings.enable_debug {
            debug_show(ctx, self);
        }
    }
}
