use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use egui::{
    Align2, Area, Color32, Context, CornerRadius, Frame, Id, Image, ImageButton, Label, Layout,
    Margin, Mesh, Pos2, Rect, Sense, TextBuffer, Ui, UiBuilder, Vec2, include_image, pos2,
    text::LayoutJob, vec2,
};
use i_overlay::{
    core::{fill_rule::FillRule, overlay_rule::OverlayRule},
    float::single::SingleFloatOverlay,
    i_float::float::compatible::FloatPointCompatible,
};
use i_triangle::float::triangulatable::Triangulatable;

use crate::{App, ui::font::Font};

#[derive(
    serde::Deserialize, serde::Serialize, Eq, Hash, PartialEq, PartialOrd, Ord, Debug, Clone, Copy,
)]
pub enum TutorialEnum {
    Menu,
    Graph,
    Edit,
    Tape,
    Control,
    Code,
    Misc,
}

impl Display for TutorialEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            match self {
                Self::Graph => t!("graph_tutorial"),
                Self::Control => t!("controls_tutorial"),
                Self::Code => t!("code_tutorial"),
                Self::Edit => t!("edit_tutorial"),
                Self::Menu => t!("menu_tutorial"),
                Self::Tape => t!("tape_tutorial"),
                Self::Misc => t!("misc_tutorial"),
            }
            .as_str(),
        )
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
        let data = self.tutorial(tutorial);
        if let Some(current) = &self.current_tutorial
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

    fn tutorial(
        &self,
        name: &str,
    ) -> (
        TutorialEnum,
        std::ops::RangeInclusive<usize>,
        std::borrow::Cow<'_, str>,
    ) {
        match name {
            "graph_section" => (TutorialEnum::Graph, 0..=0, t!("tutorial.graph_section")),
            "initial_state" => (TutorialEnum::Graph, 1..=2, t!("tutorial.initial_state")),
            "accept_state" => (TutorialEnum::Graph, 2..=2, t!("tutorial.accept_state")),
            "to_code" => (TutorialEnum::Graph, 3..=3, t!("tutorial.to_code")),
            "erase" => (TutorialEnum::Graph, 4..=4, t!("tutorial.erase")),
            "new_element_creation" => (
                TutorialEnum::Graph,
                5..=5,
                t!("tutorial.new_element_creation"),
            ),
            "by_edit" => (TutorialEnum::Graph, 6..=6, t!("tutorial.by_edit")),
            "by_touch" => (TutorialEnum::Graph, 7..=7, t!("tutorial.by_touch")),

            "code_section" => (TutorialEnum::Code, 0..=0, t!("tutorial.code_section")),
            "tabs" => (TutorialEnum::Code, 1..=1, t!("tutorial.tabs")),
            "tab_rename" => (TutorialEnum::Code, 2..=2, t!("tutorial.tab_rename")),
            "tab_add" => (TutorialEnum::Code, 3..=3, t!("tutorial.tab_add")),
            "tab_close" => (TutorialEnum::Code, 4..=4, t!("tutorial.tab_close")),
            "code_syntax" => (TutorialEnum::Code, 5..=5, t!("tutorial.code_syntax")),
            "code_comment" => (TutorialEnum::Code, 6..=6, t!("tutorial.code_comment")),

            "menu_section" => (TutorialEnum::Menu, 0..=0, t!("tutorial.menu_section")),
            "setting" => (TutorialEnum::Menu, 1..=1, t!("tutorial.setting")),
            "save" => (TutorialEnum::Menu, 2..=2, t!("tutorial.save")),
            "machine_folder" => (TutorialEnum::Menu, 3..=3, t!("tutorial.machine_folder")),
            "help" => (TutorialEnum::Menu, 4..=4, t!("tutorial.help")),
            "to_graph" => (TutorialEnum::Menu, 5..=5, t!("tutorial.to_graph")),
            "close" => (TutorialEnum::Menu, 6..=6, t!("tutorial.close")),

            "tape_section" => (TutorialEnum::Tape, 0..=0, t!("tutorial.tape_section")),
            "reading_tape" => (TutorialEnum::Tape, 1..=1, t!("tutorial.reading_tape")),
            "writing_tape" => (TutorialEnum::Tape, 2..=2, t!("tutorial.writing_tape")),
            "current_character" => (TutorialEnum::Tape, 3..=3, t!("tutorial.current_character")),
            "special_character" => (TutorialEnum::Tape, 4..=4, t!("tutorial.special_character")),

            "control_section" => (TutorialEnum::Control, 0..=0, t!("tutorial.control_section")),
            "input" => (TutorialEnum::Control, 1..=1, t!("tutorial.input")),
            "autoplay" => (TutorialEnum::Control, 2..=2, t!("tutorial.autoplay")),
            "play" => (TutorialEnum::Control, 3..=3, t!("tutorial.play")),
            "next" => (TutorialEnum::Control, 4..=4, t!("tutorial.next")),
            "reset" => (TutorialEnum::Control, 5..=5, t!("tutorial.reset")),
            "speed" => (TutorialEnum::Control, 6..=6, t!("tutorial.speed")),
            "step" => (TutorialEnum::Control, 7..=7, t!("tutorial.step")),
            "result" => (TutorialEnum::Control, 8..=8, t!("tutorial.result")),

            "edit_section" => (TutorialEnum::Edit, 0..=0, t!("tutorial.edit_section")),
            "tape_counter" => (TutorialEnum::Edit, 0..=0, t!("tutorial.tape_counter")),
            "unpin" => (TutorialEnum::Edit, 1..=1, t!("tutorial.unpin")),
            "recenter" => (TutorialEnum::Edit, 2..=2, t!("tutorial.recenter")),
            "add_state" => (TutorialEnum::Edit, 3..=3, t!("tutorial.add_state")),
            "edit" => (TutorialEnum::Edit, 4..=4, t!("tutorial.edit")),
            "delete" => (TutorialEnum::Edit, 5..=5, t!("tutorial.delete")),
            "add_transition" => (TutorialEnum::Edit, 6..=6, t!("tutorial.add_transition")),
            // "keybind" => (TutorialEnum::Misc, 0..=0, ""),
            _ => (TutorialEnum::Code, 0..=0, t!("tutorial.default")),
        }
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
