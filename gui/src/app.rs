use std::{collections::VecDeque, time::Duration};

use egui::{Context, Key, Pos2};
use egui_extras::install_image_loaders;
use ritm_core::{
    turing_graph::TuringGraph,
    turing_parser::{graph_to_string, parse_turing_graph_string},
};

use crate::{
    error::{self, RitmError},
    turing::{StateEdit, Turing},
    ui::{
        self,
        code::Code,
        constant::Constant,
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
        utils::{FileData, FileDialog, FileType},
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
    pub error: VecDeque<RitmError>,

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
    pub is_small_window: bool,

    pub listen_to_keybind: bool,

    pub temp_code: Option<String>,

    pub temp_tutorial: Option<TutorialEnum>,

    pub add_transition: bool,

    pub taking_screenshot: bool,

    pub temp_screenshot: Option<FileData>,
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
            error: VecDeque::new(),
            tutorial: Tutorials::default(),
        }
    }
}

impl Default for Transient {
    fn default() -> Self {
        Self {
            is_small_window: false,
            listen_to_keybind: true,
            temp_code: None,
            temp_tutorial: None,
            add_transition: false,
            taking_screenshot: false,
            temp_screenshot: None,
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // cc.egui_ctx.set_debug_on_hover(true);

        // Load the fonts used in the application
        load_font(cc);

        // Load the saved state from a previous session
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

    /// Convert the graph to code
    pub fn graph_to_code(&mut self) {
        let code = graph_to_string(self.turing.tm.graph_ref());
        self.code.new_tab(self.code.tab_name(), code);
    }

    /// Convert the current tab code into a graph
    /// TODO: handle in case the code is invalid
    pub fn code_to_graph(&mut self) -> Result<(), RitmError> {
        match parse_turing_graph_string(self.code.current_code()?) {
            Ok(graph) => {
                self.turing = Turing::new_graph(graph)?;
                self.turing.layer_graph();
                self.code.set_curr_parsing_error(None);
            }
            Err(e) => {
                self.code.set_curr_parsing_error(Some(e));
            }
        }
        self.graph.recenter();
        Ok(())
    }

    /// Change the number of tapes in the graph
    pub fn update_k(&mut self, k: usize) -> Result<(), RitmError> {
        self.graph.reset();
        let new_turing = Turing::new_graph(
            TuringGraph::new(k, true).map_err(|e| RitmError::CoreError(e.to_string()))?,
        )?;
        self.turing = new_turing;
        self.turing.layer_graph();
        Ok(())
    }

    /// Prepare the edition of a state and open the popup
    pub fn edit_state(&mut self, state_id: usize) -> Result<(), RitmError> {
        self.turing.prepare_state_edit(state_id)?;
        self.popup
            .switch_to(ui::popup::RitmPopupEnum::StateEdit(Some(state_id)));
        Ok(())
    }

    /// Prepare the edition of a state and open the popup
    pub fn edit_transition(&mut self, source_id: usize, target_id: usize) -> Result<(), RitmError> {
        self.turing.prepare_transition_edit(source_id, target_id)?;
        self.popup
            .switch_to(ui::popup::RitmPopupEnum::TransitionEdit((
                source_id, target_id,
            )));
        Ok(())
    }

    pub fn new_state_at_pos(&mut self, pos: Pos2) {
        let mut state_edit = StateEdit::empty(self.turing.tm.graph_ref().get_next_id());

        state_edit.get_edit().state.position = pos;
        state_edit.get_edit().name = format!(
            "q_{}",
            self.turing.tm.graph_ref().get_state_hashmap().len() + 1
        );

        self.turing.state_edit = Some(state_edit);
        self.popup
            .switch_to(ui::popup::RitmPopupEnum::StateEdit(None));
    }
}

/// Update loop
impl eframe::App for App {
    /// Save the state of the application
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "ritm", self);
    }

    /// Draw every frame the application
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        install_image_loaders(ctx);
        Constant::update_scale(ctx);
        #[cfg(feature = "profiling")]
        puffin::profile_scope!("root");

        // Start the tutorial
        if let Some(tutorial) = self.transient.temp_tutorial {
            self.transient.temp_tutorial = None;
            self.tutorial.start(tutorial);
        }

        // Draw the whole application and show any error
        if let Err(error) = ui::show(self, ctx) {
            self.error.push_back(error);
        }

        error::show(ctx, self);

        // Draw the tutorial
        tutorial::show(ctx, self);

        // End the tutorial
        if let Some(tutorial) = self.tutorial.has_finished {
            self.tutorial.has_finished = None;
            self.tutorial
                .already_played
                .entry(tutorial)
                .and_modify(|b| *b = true);
        }

        // Force the Ui to update if the machine is running
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

        keybind(ctx, self);

        if self.settings.theme_changer {
            theme_changer(ctx, self);
        }

        if self.settings.enable_debug {
            debug_show(ctx, self);
        }

        if let Some(screenshot) = &self.transient.temp_screenshot {
            FileDialog::default().save("test.png", screenshot.to_vec(), FileType::Image);
            self.transient.temp_screenshot = None;
        }
    }
}

fn keybind(ctx: &Context, app: &mut App) {
    if app.transient.listen_to_keybind {
        let popup_displayed = app.popup.current().is_some();

        ctx.input(|r| {
            if r.key_pressed(Key::Escape) {
                if app.popup.current().is_some() {
                    // Request graceful exit of popup
                    app.popup.close();
                } else {
                    // Unselect what is selected
                    app.graph.unselect()
                }
            }

            if r.key_pressed(Key::Enter) {
                if !app.error.is_empty() {
                    app.error.pop_front();
                } else if app.popup.current().is_some() {
                    app.popup.confirm();
                } else if !popup_displayed && app.tutorial.in_tutorial() {
                    app.tutorial.next();
                }
            }

            if !popup_displayed && !app.tutorial.in_tutorial() {
                // Press S to create a state
                if r.key_pressed(Key::S) {
                    app.edit.is_adding_state ^= true;
                }

                // Press T to create a transition
                if app.graph.selected_state().is_some() && r.key_pressed(Key::T) {
                    app.edit.is_adding_transition ^= true;
                }

                // Press U to unpin all state
                if r.key_pressed(Key::U) {
                    app.turing.unpin_all();
                }

                // Press C to open and close code section
                if r.key_pressed(Key::C) {
                    app.code.toggle();
                }

                // Press R to recenter
                if r.key_pressed(Key::R) {
                    app.graph.recenter();
                }

                // Press Space to make 1 iteration
                if app.turing.accepted.is_none() && r.key_pressed(Key::Space) {
                    app.turing.next_step();
                }

                // Press P to autoplay the machine
                if r.key_pressed(Key::P) {
                    if app.control.is_running() {
                        app.control.run();
                    } else {
                        app.control.pause();
                    }
                }

                if r.key_pressed(Key::Plus) {
                    app.control.speed_down();
                }

                if r.key_pressed(Key::Minus) {
                    app.control.speed_up();
                }

                // Press Backspace to reset the machine
                if r.key_pressed(Key::Backspace) {
                    app.reset();
                }

                // Press Backspace to reset the machine
                if r.key_pressed(Key::Delete) {
                    if let Some(state_id) = app.graph.selected_state()
                        && state_id > 1
                    {
                        let _ = app.turing.remove_state(state_id);
                    }

                    if let Some(transition_id) = app.graph.selected_transitions() {
                        let _ = app
                            .turing
                            .remove_transitions(transition_id.source_id, transition_id.target_id);
                    }
                }
            }
        });
    } else {
        app.transient.listen_to_keybind = true;
    }
}
