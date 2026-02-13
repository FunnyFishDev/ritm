use egui::{
    Align2, Area, Color32, Context, CornerRadius, Frame, Id, Image, ImageButton, Label, Layout,
    Margin, Mesh, Pos2, Rect, Ui, UiBuilder, Vec2, include_image, pos2, text::LayoutJob, vec2,
};
use i_overlay::{
    core::{fill_rule::FillRule, overlay_rule::OverlayRule},
    float::single::SingleFloatOverlay,
    i_float::float::compatible::FloatPointCompatible,
};
use i_triangle::float::triangulatable::Triangulatable;

use crate::{App, ui::font::Font};

#[derive(Debug)]
pub struct Tutorial {
    boxes: Vec<TutorialBox>,
    current_step: Option<usize>,
}

#[derive(Debug)]
pub struct TutorialBox {
    rect: Rect,
    text: String,
    alignment: Align2,
    text_size: Option<Vec2>,
    end_page: bool,
}

impl Default for Tutorial {
    fn default() -> Self {
        let rect = Rect::from_min_size(pos2(400.0, 300.0), vec2(200.0, 200.0));
        Self {
            // rects: vec![],
            boxes: vec![
                TutorialBox {
                    rect,
                    text: "RB".to_string(),
                    alignment: Align2::RIGHT_BOTTOM,
                    text_size: None,
                    end_page: true,
                },
                TutorialBox {
                    rect,
                    text: "CB".to_string(),
                    alignment: Align2::CENTER_BOTTOM,
                    text_size: None,
                    end_page: true,
                },
                TutorialBox {
                    rect,
                    text: "LB".to_string(),
                    alignment: Align2::LEFT_BOTTOM,
                    text_size: None,
                    end_page: true,
                },
                TutorialBox {
                    rect,
                    text: "LC".to_string(),
                    alignment: Align2::LEFT_CENTER,
                    text_size: None,
                    end_page: true,
                },
                TutorialBox {
                    rect,
                    text: "LT".to_string(),
                    alignment: Align2::LEFT_TOP,
                    text_size: None,
                    end_page: true,
                },
                TutorialBox {
                    rect,
                    text: "CT".to_string(),
                    alignment: Align2::CENTER_TOP,
                    text_size: None,
                    end_page: true,
                },
                TutorialBox {
                    rect,
                    text: "RT".to_string(),
                    alignment: Align2::RIGHT_TOP,
                    text_size: None,
                    end_page: true,
                },
                TutorialBox {
                    rect,
                    text: "RC".to_string(),
                    alignment: Align2::RIGHT_CENTER,
                    text_size: None,
                    end_page: true,
                },
                TutorialBox {
                    rect,
                    text: "CC".to_string(),
                    alignment: Align2::CENTER_CENTER,
                    text_size: None,
                    end_page: true,
                },
            ],
            current_step: Some(0),
        }
    }
}

pub fn show(ctx: &Context, app: &mut App) {
    let Some(current_step) = app.tutorial.current_step else {
        return;
    };
    Area::new(Id::new("Tutorial_area"))
        .fixed_pos(Pos2::ZERO)
        .movable(false)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let mut mesh = Mesh::default();
            let mut main = vec![rect_to_contour(&ui.clip_rect()).to_vec()];

            let mut end = false;
            let mut i = current_step;

            while !end {
                let boxe = &app.tutorial.boxes[i];

                let rect = rect_to_contour(&boxe.rect);
                main = main.overlay(&rect, OverlayRule::Xor, FillRule::EvenOdd)[0].clone();

                end = boxe.end_page;
                i += 1
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

            let mut end = false;
            let mut i = current_step;

            while !end {
                let boxe = &app.tutorial.boxes[i];
                end = boxe.end_page;

                let pos = boxe.alignment.pos_in_rect(&boxe.rect);
                let text_max_size = boxe.text_size.unwrap_or(vec2(300.0, 300.0));
                tuto_box(
                    ui,
                    app,
                    boxe.text.to_string(),
                    pos,
                    text_max_size,
                    boxe.alignment,
                );

                i += 1
            }
        });
}

fn tuto_box(ui: &mut Ui, app: &mut App, text: String, pos: Pos2, max_size: Vec2, align: Align2) {
    let margin = Margin::same(10);
    let bottom_height = 20.0;
    let job = LayoutJob {
        halign: egui::Align::Min,
        ..LayoutJob::simple(
            text,
            Font::default_big(),
            app.theme.text_primary,
            max_size.x,
        )
    };
    let galley = ui.fonts(|f| f.layout_job(job));
    let rect = Rect::from_center_size(
        pos + vec2(
            align.x().to_sign() * (galley.size().x / 2.0 + margin.leftf()),
            -bottom_height / 2.0
                + align.y().to_sign() * ((galley.size().y + bottom_height) / 2.0 + margin.topf()),
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
                        if let Some(step) = app.tutorial.current_step {
                            if step >= app.tutorial.boxes.len()-1 {
                                if ui
                                    .add(
                                        ImageButton::new(
                                            Image::new(include_image!(
                                                "../../assets/icon/close.svg"
                                            ))
                                            .shrink_to_fit()
                                            .tint(app.theme.overlay),
                                        )
                                        .frame(false),
                                    )
                                    .clicked()
                                {
                                    app.tutorial.current_step = None
                                }
                            } else if ui
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
                                app.tutorial.current_step = Some(step + 1)
                            }
                        }
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
