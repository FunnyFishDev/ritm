use std::{collections::BTreeMap, time::Duration};

use egui::{
    CentralPanel, Color32, Context, FontData, FontDefinitions, FontFamily, Frame, Id, Key, Margin,
    ScrollArea, ViewportBuilder, ViewportId,
    color_picker::{Alpha, color_picker_color32},
    vec2,
};
use egui_extras::install_image_loaders;
use egui_flex::{Flex, FlexInstance, item};
use ritm_core::turing_parser::{graph_to_string, parse_turing_graph_string};

use crate::{
    error::{self, RitmError},
    turing::Turing,
    ui::{
        self,
        code::Code,
        control::Control,
        edit::Edit,
        graph::Graph,
        menu::Menu,
        popup::{RitmPopup, settings::Settings},
        theme::Theme,
    },
};

/// The only structure that is persistent each redraw of the application
pub struct App {
    /// The turing machine itself
    pub turing: Turing,

    pub edit: Edit,

    pub graph: Graph,

    pub control: Control,

    pub settings: Settings,

    pub menu: Menu,

    /// Which popup to display
    pub popup: RitmPopup,

    /// The code used to create the turing machine
    pub code: Code,

    pub error: Option<RitmError>,

    /// The event/state of the application
    pub transient: Transient,

    /// Current theme
    pub theme: Theme,

    pub help_slide_index: usize,
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

    pub code: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        let mut sf = Self {
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
        };
        
        sf
    }
}

impl Default for Transient {
    fn default() -> Self {
        Self {
            are_settings_visible: false,
            is_small_window: false,
            listen_to_keybind: true,
            take_screenshot: false,
            code: None,
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

    /// Reset the machine execution
    pub fn reset(&mut self) {
        self.turing.reset();
    }

    /// Reset the machine execution with the new input
    /// TODO: stop ignoring result to avoid cloudflare global shutdown
    pub fn set_input(&mut self) {
        let _ = self.turing.tm.reset_word(self.control.input());
        self.turing.reset();
    }

    pub fn graph_to_code(&mut self) {
        let code = graph_to_string(self.turing.tm.graph_ref());
        self.code.new_tab(self.code.tab_name(), code);
    }

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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        install_image_loaders(ctx);

        if let Err(error) = ui::show(self, ctx)
            && self.error.is_none()
        {
            self.error = Some(error)
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
                    self.turing.unpin();
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
        // theme_changer(ctx, self);
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

// pub fn take_screenshot(app: &mut App, ui: &mut Ui) {
//     let ctx = ui.ctx();
//     let rect = ui.min_rect();
//     if app.event.take_screenshot {
//         app.event.take_screenshot = false;

//         ctx.send_viewport_cmd(ViewportCommand::Screenshot(UserData::default()));

//         ctx.input(|i| {
//             i.events.iter().for_each(|e| {
//                 if let egui::Event::Screenshot { image, .. } = e {
//                     let image = image.region(&rect, Some(i.pixels_per_point));
//                     save_buffer(
//                         Path::new("assets/help/screenshot.png"),
//                         image.as_raw(),
//                         image.source_size.x as u32,
//                         image.source_size.y as u32,
//                         ExtendedColorType::Rgba8,
//                     )
//                     .unwrap();
//                 }
//             })
//         });
//     }
// }
