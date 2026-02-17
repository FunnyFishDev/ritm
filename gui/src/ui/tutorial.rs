use std::{
    collections::HashMap,
    fmt::{self, Display},
    ops::RangeInclusive,
};

use egui::{
    Align2, Area, Color32, Context, CornerRadius, Frame, Id, Image, ImageButton, Label, Layout,
    Margin, Mesh, Pos2, Rect, Sense, Ui, UiBuilder, Vec2, include_image, pos2, text::LayoutJob,
    vec2,
};
use i_overlay::{
    core::{fill_rule::FillRule, overlay_rule::OverlayRule},
    float::single::SingleFloatOverlay,
    i_float::float::compatible::FloatPointCompatible,
};
use i_triangle::float::triangulatable::Triangulatable;
use phf_macros::phf_map;

use crate::{App, ui::font::Font};

static TUTORIALS: phf::Map<&'static str, (TutorialEnum, RangeInclusive<usize>, &'static str)> = phf_map! {
    "default" => (TutorialEnum::Code, 0..=0, "/!\\ Text not found. Please contact administrator. /!\\"),

    "graph_section" => (TutorialEnum::Graph, 0..=0, "The graph section allow you to \"draw\" the turing machine without writing a single line of code."),
    "initial_state" => (TutorialEnum::Graph, 1..=2, "This is the initial state..."),
    "accept_state" => (TutorialEnum::Graph, 2..=2, "...and this is the accept state"),
    "to_code" => (TutorialEnum::Graph, 3..=3, "Click on this button to convert the current graph into code\nMany operations can delete the graph, save it often to avoid losing it !"),
    "erase" => (TutorialEnum::Graph, 4..=4, "This button reset the graph, keeping only the initial and accept state"),
    "new_element_creation" => (TutorialEnum::Graph, 5..=5, "You can create new state and transition in 2 different ways :"),
    "by_edit" => (TutorialEnum::Graph, 6..=6, "You can use the edit menu..."),
    "by_touch" => (TutorialEnum::Graph, 7..=7, "...Or you can press a long time on the background for a new state and on a state for a new transition."),

    "code_section" => (TutorialEnum::Code, 0..=0, "The code section allow you to define turing machine with code."),
    "tabs" => (TutorialEnum::Code, 1..=1, "Tabs allow you to create new turing machine without erasing the previous ones."),
    "tab_rename" => (TutorialEnum::Code, 2..=2, "To rename a tab simply double-click on it."),
    "tab_add" => (TutorialEnum::Code, 3..=3, "You can add new tab by clicking on the \"+\""),
    "tab_close" => (TutorialEnum::Code, 4..=4, "You can close tab you don't need anymore by clicking on the cross.\n\nBe aware that if only one tab remain you will not be able to remove it unless you create another one."),
    "code_syntax" => (TutorialEnum::Code, 5..=5, "The syntax is the following :\n\nq_start {ç -> R} q_end; for 0..=0 writing tape\n\nq_start {ç, ç -> R, ç, R} q_end; for 1 writing tape\n\nq_start {ç, ç, ç -> R, ç, R, ç, R} q_end; for 2 writing tape\n\netc..."),
    "code_comment" => (TutorialEnum::Code, 6..=6, "You can also add comment to improve the readability by using \"//\" before the comment"),

    "menu_section" => (TutorialEnum::Menu, 0..=0, "The menu hold different button"),
    "setting" => (TutorialEnum::Menu, 1..=1, "This button open the settings of the application"),
    "save" => (TutorialEnum::Menu, 2..=2, "This button save the code of the current tab in a file with the .tm extension"),
    "machine_folder" => (TutorialEnum::Menu, 3..=3, "This button open a sub-menu containing premade turing machine.\nIt also allow you to load a file with the .tm extension, defining a turing machine."),
    "help" => (TutorialEnum::Menu, 4..=4, "This button display the current tutorial"),
    "to_graph" => (TutorialEnum::Menu, 5..=5, "Click on this button to convert the code of the current tab into a graph\n\n/!\\ Be aware that doing so will overwrite any graph present."),
    "close" => (TutorialEnum::Menu, 6..=6, "This button collapse the code section to grant more space to the graph section"),

    "tape_section" => (TutorialEnum::Tape, 0..=0, "These are the tape used by the turing machine."),
    "reading_tape" => (TutorialEnum::Tape, 1..=1, "For 1 tape machine this is a reading and writing tape.\n for 2 or more tapes machine this is only a reading tape."),
    "writing_tape" => (TutorialEnum::Tape, 2..=2, "These tapes are always reading and writing tapes"),
    "current_character" => (TutorialEnum::Tape, 3..=3, "These are the current character being read on the tapes. The next transition followed depend on those."),
    "special_character" => (TutorialEnum::Tape, 4..=4, "Some character are special and cannot be used as input :\nç represent the start of the tapes\n$ represent the end of the reading tape\n_ represent the end of the writing tapes"),

    "control_section" => (TutorialEnum::Control, 0..=0, "This is the controls section.\nIn this section you can interact with the machine execution."),
    "input" => (TutorialEnum::Control, 1..=1, "The input can be submitted here.\n'ç', '$' and '_' are special characters and therefore cannot be used !"),
    "autoplay" => (TutorialEnum::Control, 2..=2, "You can run the machine here by pressing the play button.\n"),
    "play" => (TutorialEnum::Control, 3..=3, "You can let the machine run itself by pressing this button"),
    "next" => (TutorialEnum::Control, 4..=4, "You can use this button to go to the next step"),
    "reset" => (TutorialEnum::Control, 5..=5, "You can reset the machine and the tapes by clicking on this button"),
    "speed" => (TutorialEnum::Control, 6..=6, "You can change the speed here"),
    "step" => (TutorialEnum::Control, 7..=7, "The current step is diplayed here"),
    "result" => (TutorialEnum::Control, 8..=8, "The current state of the machine. It can be 'idle', 'running', 'accepted' or 'rejected'"),

    "edit_section" => (TutorialEnum::Edit, 0..=0, "Here you can interact with the graph and its elements"),
    "tape_counter" => (TutorialEnum::Edit, 1..=1, "You can change the number of tape here\n\n/!\\ ANY CHANGE WILL ERASE THE CURRENT MACHINE /!\\"),
    "unpin" => (TutorialEnum::Edit, 2..=2, "Unpin the state so the natural force apply"),
    "recenter" => (TutorialEnum::Edit, 3..=3, "Recenter the graph"),
    "add_state" => (TutorialEnum::Edit, 4..=4, "Once toggled, the next click on the background of the graph will add a new state"),
    "edit" => (TutorialEnum::Edit, 5..=5, "Edit the selected state or transitions"),
    "delete" => (TutorialEnum::Edit, 6..=6, "Delete the selected state or transitions"),
    "add_transition" => (TutorialEnum::Edit, 7..=7, "Once toggled, the next click on a state will create a transition."),

    // "keybind" => (TutorialEnum::Misc, 0..=0, ""),
};
#[derive(
    serde::Deserialize, serde::Serialize, Eq, Hash, PartialEq, PartialOrd, Ord, Debug, Clone, Copy,
)]
pub enum TutorialEnum {
    Graph,
    Control,
    Code,
    Edit,
    Menu,
    Tape,
    Misc,
}

impl Display for TutorialEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Graph => "Graph tutorial",
            Self::Control => "Control tutorial",
            Self::Code => "Code tutorial",
            Self::Edit => "Edit tutorial",
            Self::Menu => "Menu tutorial",
            Self::Tape => "Tape tutorial",
            Self::Misc => "Misc tutorial",
        })
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Tutorials {
    tutorial_boxs: Vec<TutorialBox>,
    current_tutorial: Option<TutorialEnum>,
    current_step: usize,
    pub already_played: HashMap<TutorialEnum, bool>,
    pub has_finished: Option<TutorialEnum>,
}

impl Default for Tutorials {
    fn default() -> Self {
        Self {
            tutorial_boxs: vec![],
            current_tutorial: None,
            current_step: 0,
            already_played: HashMap::from([
                (TutorialEnum::Graph, false),
                (TutorialEnum::Control, false),
                (TutorialEnum::Code, false),
                (TutorialEnum::Edit, false),
                (TutorialEnum::Menu, false),
                (TutorialEnum::Tape, false),
            ]),
            has_finished: None,
        }
    }
}

impl Tutorials {
    pub fn add_boxe(&mut self, tutorial: &str, mut boxe: TutorialBox) {
        if let Some(current) = &self.current_tutorial
            && let Some(data) = TUTORIALS.get(tutorial)
            && boxe.rect.is_finite()
            && *current == data.0
            && data.1.contains(&self.current_step)
        {
            boxe.text = data.2.to_string();
            self.tutorial_boxs.push(boxe);
        }
    }

    pub fn start(&mut self, tutorial: TutorialEnum) {
        self.current_tutorial = Some(tutorial)
    }

    pub fn in_tutorial(&self) -> bool {
        self.current_tutorial.is_some()
    }

    pub fn close(&mut self) {
        self.has_finished = self.current_tutorial;
        self.current_tutorial = None;
        self.current_step = 0;
    }

    pub fn next(&mut self) {
        self.current_step += 1;
    }

    pub fn current_tutorial(&self) -> Option<TutorialEnum> {
        self.current_tutorial
    }

    pub fn current_step(&self) -> usize {
        self.current_step
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct TutorialBox {
    pub rect: Rect,
    pub text: String,
    pub alignment: Align2,
    pub text_size: Option<Vec2>,
    pub close_tutorial: bool,
}

impl fmt::Debug for TutorialBox {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.rect)
    }
}

impl TutorialBox {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            ..Default::default()
        }
    }

    pub fn with_align(mut self, align: Align2) -> Self {
        self.alignment = align;
        self
    }

    pub fn close(mut self) -> Self {
        self.close_tutorial = true;
        self
    }

    pub fn with_text_size(mut self, size: Vec2) -> Self {
        self.text_size = Some(size);
        self
    }
}

impl Default for TutorialBox {
    fn default() -> Self {
        Self {
            rect: Rect::ZERO,
            text: "Default".to_string(),
            alignment: Align2::CENTER_CENTER,
            text_size: None,
            close_tutorial: false,
        }
    }
}

pub fn show(ctx: &Context, app: &mut App) {
    if app.tutorial.current_tutorial.is_none() {
        return;
    }
    Area::new(Id::new("Tutorial_area"))
        .fixed_pos(Pos2::ZERO)
        .movable(false)
        .sense(Sense::all())
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let mut mesh = Mesh::default();
            let mut main = vec![rect_to_contour(&ui.clip_rect()).to_vec()];

            if app.tutorial.tutorial_boxs.is_empty() {
                app.tutorial.close();
                return;
            }

            for boxe in &app.tutorial.tutorial_boxs {
                let rect = rect_to_contour(&boxe.rect);
                main = main.overlay(&rect, OverlayRule::Xor, FillRule::EvenOdd)[0].clone();
            }

            let triangulation = main.triangulate().to_triangulation::<u32>();

            let color = Color32::from_black_alpha(100);

            triangulation.points.iter().for_each(|i| {
                mesh.colored_vertex((*i).into(), color);
            });

            triangulation.indices.chunks(3).for_each(|c| {
                mesh.add_triangle(c[0], c[1], c[2]);
            });

            ui.painter().add(mesh);

            let len = app.tutorial.tutorial_boxs.len();
            for i in 0..len {
                let boxe = app.tutorial.tutorial_boxs.pop().unwrap();
                let pos = boxe.alignment.pos_in_rect(&boxe.rect);
                let text_max_size = boxe.text_size.unwrap_or(vec2(300.0, 300.0));
                let next = i == len - 1;
                tuto_box(ui, app, boxe, pos, text_max_size, next);
            }
        });
}

fn tuto_box(ui: &mut Ui, app: &mut App, boxe: TutorialBox, pos: Pos2, max_size: Vec2, next: bool) {
    let margin = Margin::same(10);
    let bottom_height = 20.0;
    let job = LayoutJob {
        halign: egui::Align::Min,
        ..LayoutJob::simple(
            boxe.text.to_string(),
            Font::default_medium(),
            app.theme.text_primary,
            max_size.x,
        )
    };
    let galley = ui.fonts(|f| f.layout_job(job));
    let rect = Rect::from_center_size(
        pos + vec2(
            boxe.alignment.x().to_sign() * (galley.size().x / 2.0 + margin.leftf() + 20.0),
            -bottom_height / 2.0
                + boxe.alignment.y().to_sign()
                    * ((galley.size().y + bottom_height) / 2.0 + margin.topf() + 20.0),
        ),
        galley.size(),
    );

    ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        Frame::new()
            .fill(app.theme.surface)
            .corner_radius(CornerRadius::same(10))
            .inner_margin(margin)
            .show(ui, |ui| {
                ui.put(rect, Label::new(galley));

                ui.allocate_ui_with_layout(
                    vec2(ui.available_width(), bottom_height),
                    Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if boxe.close_tutorial {
                            if ui
                                .add(
                                    ImageButton::new(
                                        Image::new(include_image!("../../assets/icon/close.svg"))
                                            .shrink_to_fit()
                                            .tint(app.theme.overlay),
                                    )
                                    .frame(false),
                                )
                                .clicked()
                            {
                                app.tutorial.close();
                            }
                        } else if next
                            && ui
                                .add(
                                    ImageButton::new(
                                        Image::new(include_image!("../../assets/icon/right.svg"))
                                            .shrink_to_fit()
                                            .tint(app.theme.overlay),
                                    )
                                    .frame(false),
                                )
                                .clicked()
                        {
                            app.tutorial.next();
                        };
                    },
                );
            });
    });
}

fn rect_to_contour(rect: &Rect) -> [Pos; 4] {
    [
        rect.left_top().into(),
        rect.right_top().into(),
        rect.right_bottom().into(),
        rect.left_bottom().into(),
    ]
}

#[derive(Clone, Copy, Debug)]
struct Pos {
    x: f32,
    y: f32,
}

impl FloatPointCompatible<f32> for Pos {
    fn from_xy(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }
}

impl From<Pos2> for Pos {
    fn from(value: Pos2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<Pos> for Pos2 {
    fn from(val: Pos) -> Self {
        pos2(val.x, val.y)
    }
}
