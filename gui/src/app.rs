use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::Path,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

use egui::{
    FontData, FontDefinitions, FontFamily, Key, Pos2, Rect, Ui, UserData, ViewportCommand, vec2,
};
use egui_extras::install_image_loaders;
use image::{ExtendedColorType, save_buffer};
use rand::random;
use ritm_core::{
    turing_graph::TuringMachineGraph,
    turing_machine::{Mode, TuringExecutionSteps, TuringMachines},
    turing_parser::{graph_to_string, parse_turing_graph_string},
    turing_state::TuringStateType, turing_transition::{TuringDirection, TuringTransition},
};

use crate::{
    turing::{State, Transition, TransitionEdit},
    ui::{self, popup::RitmPopup, theme::Theme, utils::FileDialog},
};

/// The only structure that is persistent each redraw of the application
pub struct App {
    /// The turing machine itself
    pub turing: TuringMachines,

    /// Current step of the turing machine or None if no step
    pub step: TuringExecutionSteps,

    /// User input for the turing machine
    pub input: String,

    /// Used for graph display, zooming and moving
    pub graph_rect: Rect,

    /// Hold graph information of each states
    pub states: HashMap<usize, State>,

    /// The code used to create the turing machine
    pub code: String,

    /// The event/state of the application
    pub event: Event,

    /// Current theme
    pub theme: Theme,

    /// Selected state
    pub selected_state: Option<usize>,

    /// Selected transition
    pub selected_transition: Option<(usize, usize)>,

    /// Interval between each iteration
    pub interval: i32,

    /// Store the current editing of the transitions between 2 states
    pub rules_edit: Vec<TransitionEdit>,

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
        let graph = TuringMachineGraph::new(1).ok().unwrap();

        let mut turing = TuringMachines::new(graph, "".to_string(), Mode::StopAfter(500)).unwrap();

        let step = turing.into_iter().next().unwrap();

        let mut sf = Self {
            turing: turing,
            step: step,
            input: "".to_string(),
            graph_rect: Rect::ZERO,
            states: HashMap::new(),
            code: "".to_string(), // TODO display a message as comment instead
            event: Event::default(),
            theme: Theme::DEFAULT,
            selected_state: None,
            selected_transition: None,
            interval: 0,
            rules_edit: vec![],
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

        // Update the graph data with the turing data at initialization
        sf.turing_to_graph();

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

    /// Add a state to the turing machine at a certain position
    pub fn add_state(&mut self, position: Pos2, name: String) {
        // Create the state in the core of the application to miror the id into
        // the gui structure
        let id = self.turing.graph_mut().add_state(&name);
        self.states.insert(
            id,
            State {
                id: id,
                name: name,
                position,
                ..Default::default()
            },
        );
    }

    /// Remove the selected state from the core and the gui structure
    pub fn remove_state(&mut self) {
        for (_, state) in self.states.iter_mut() {
            state
                .transitions
                .retain(|t| t.target_id != self.selected_state.unwrap());
        }
        self.states.remove(&self.selected_state.unwrap());
        self.selected_state = None;
    }

    /// Remove the selected state from the core and the gui structure
    pub fn remove_transitions(&mut self) {
        let (from, to) = self.selected_transition.unwrap();
        if let Err(_) = self
            .turing
            .graph_mut()
            .remove_transitions_with_index(from, to)
        {
            println!("Error occured !")
        }

        self.states.get_mut(&from).unwrap().transitions.clear();
        for (i, transition) in self
            .turing
            .graph_ref()
            .get_state(from)
            .unwrap()
            .transitions
            .iter()
            .enumerate()
        {
            self.states
                .get_mut(&from)
                .unwrap()
                .transitions
                .push(Transition {
                    id: i,
                    identifier: (from, i),
                    parent_id: from,
                    target_id: to,
                    text: transition.to_string(),
                });
        }
        self.selected_transition = None;
    }

    /// Add a transition graphically and logically
    pub fn add_transition(&mut self, target: usize) {
        let transition = TuringTransition::new(
            vec!['ç'; self.turing.graph_ref().get_k() + 1],
            TuringDirection::None,
            vec![('ç', TuringDirection::None); self.turing.graph_ref().get_k()],
        );

        let transition_rule = transition.to_string();
        self.turing
            .graph_mut()
            .append_rule_state(self.selected_state.unwrap(), transition, target)
            .ok();

        let source = State::get_mut(self, self.selected_state.unwrap());
        let transition =
            Transition::new(transition_rule, source.transitions.len(), source.id, target);
        source.transitions.push(transition);
        self.event.is_adding_transition = false;

        self.selected_state = None;
        self.selected_transition = None;
    }

    /// Continue the execution of the turing machine by one iteration
    pub fn next(&mut self) {
        match self.turing.into_iter().next() {
            Some(step) => self.step = step,
            None => {
                // Store the result of the computation
                self.event.is_accepted = Some(
                    self.turing
                        .graph_ref()
                        .get_state(self.turing.get_state_pointer())
                        .unwrap()
                        .state_type
                        == TuringStateType::Accepting,
                );

                // Disable auto-play if the user reset the machine
                self.event.is_running = false;
            }
        }
    }

    /// Reset the graph
    pub fn reset_graph(&mut self) {
        self.turing = TuringMachines::new(
            TuringMachineGraph::new(1).ok().unwrap(),
            self.turing.get_word().to_string(),
            Mode::StopAfter(500),
        )
        .unwrap();
        self.step = self.turing.into_iter().next().unwrap();
        self.selected_state = None;
        self.selected_transition = None;
        self.event.need_recenter = true;
        self.turing_to_graph();
        self.reset();
    }

    /// Reset the machine execution
    pub fn reset(&mut self) {
        self.turing.reset();
        self.step = self.turing.into_iter().next().unwrap();
        self.event.is_accepted = None;
    }

    /// Reset the machine execution with the new input
    pub fn set_input(&mut self) {
        let _ = self.turing.reset_word(&self.input);
        self.step = self.turing.into_iter().next().unwrap();
        self.event.is_accepted = None;
    }

    /// Apply the changes made to the transition
    pub fn apply_transition_change(&mut self) {
        // Need to have a transition selected

        if let Some(selected_transition) = self.selected_transition
            && !self
                .rules_edit
                .iter()
                .any(|transition| transition.to().is_err())
        {
            // Reset the machine to avoid problem when removing transition, especially transition_taken
            // MEMO : maybe block the removing of the last transition taken ?
            let _ = self.turing.reset();

            let transitions = &mut self
                .turing
                .graph_mut()
                .get_state_mut(selected_transition.0)
                .unwrap()
                .transitions;

            transitions.clear();
            transitions.append(
                self.rules_edit
                    .iter()
                    .map(|transition| transition.to().unwrap())
                    .collect::<Vec<TuringTransition>>()
                    .as_mut(),
            );

            let transitions_gui: Vec<Transition> = transitions
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    Transition::new(
                        f.to_string(),
                        i,
                        selected_transition.0,
                        f.index_to_state.unwrap(),
                    )
                })
                .collect();

            State::get_mut(self, selected_transition.0).transitions = transitions_gui;
        };
        self.popup = RitmPopup::None;
    }

    /// Cancel the changes made to the transition
    pub fn cancel_transition_change(&mut self) {
        self.rules_edit.clear();
        self.popup = RitmPopup::None;
    }

    pub fn graph_to_code(&mut self) {
        self.code = graph_to_string(self.turing.graph_ref());
    }

    pub fn code_to_graph(&mut self) {
        match parse_turing_graph_string(self.code.to_string()) {
            Ok(graph) => {
                self.turing =
                    TuringMachines::new(graph, self.input.to_string(), Mode::StopFirstReject)
                        .unwrap();
                self.turing_to_graph();
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
        self.event.need_recenter = true;
    }

    /// Create a graphical representation of a turing machine by copying each states and transitions information into GUI-oriented struct
    pub fn turing_to_graph(&mut self) {
        self.states = HashMap::new();

        let mut state_list: BTreeSet<usize> = BTreeSet::new();
        let mut layer_state: Vec<usize> = vec![];

        for (i, state) in self.turing.graph_ref().get_states().iter().enumerate() {
            let index = i;

            let transitions: Vec<Transition> = state
                .transitions
                .iter()
                .enumerate()
                .map(|(i, f)| Transition::new(f.to_string(), i, index, f.index_to_state.unwrap()))
                .collect();

            if state.state_type == TuringStateType::Accepting || state.transitions.is_empty() {
                layer_state.push(index);
            } else {
                state_list.insert(index);
            }

            self.states.insert(
                index,
                State {
                    id: index,
                    name: state.name.to_string(),
                    transitions: transitions,
                    ..Default::default()
                },
            );
        }

        let mut j = 0.0;
        while !(state_list.is_empty() && layer_state.is_empty()) {
            let mut next_layer_state: Vec<usize> = vec![];

            let layer_count = layer_state.len() as f32 - 1.0;
            for (i, state_id) in layer_state.iter().enumerate() {
                State::get_mut(self, *state_id).position = Pos2::new(
                    (j - (layer_count / 2.0 - i as f32)) * -200.0,
                    (j + (layer_count / 2.0 - i as f32)) * -200.0,
                ) + vec2(random(), random());
            }

            for state_id in &state_list {
                let state = self.turing.graph_ref().get_state(*state_id).unwrap();
                if state
                    .transitions
                    .iter()
                    .any(|f| layer_state.contains(&f.index_to_state.unwrap()))
                {
                    next_layer_state.push(*state_id);
                }
            }

            state_list.retain(|k| !next_layer_state.contains(k));

            layer_state = next_layer_state;

            j += 1.0;
        }
    }

    /// Unpin all states
    pub fn unpin(&mut self) {
        for (_, state) in &mut self.states {
            state.is_pinned = false;
        }
    }

    /// Pin all states
    pub fn pin(&mut self) {
        for (_, state) in &mut self.states {
            state.is_pinned = true;
        }
    }
}

/// Update loop
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        install_image_loaders(ctx);

        ui::show(self, ctx);

        if self.event.is_running {
            if ctx.input(|r| r.time) - self.last_step_time >= 2.0_f32.powi(self.interval) as f64 {
                self.next();
                self.last_step_time = ctx.input(|r| r.time);
            }
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
                    self.unpin();
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
                    self.next();
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
