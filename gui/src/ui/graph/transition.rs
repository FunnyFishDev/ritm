use std::{
    collections::{BTreeMap, HashMap, HashSet, btree_map::Entry},
    f32::{self, consts::PI},
};

use crate::{
    App,
    error::RitmError,
    turing::TransitionId,
    ui::{
        constant::Constant,
        font::Font,
        theme::Theme,
        utils::{self},
    },
};
use egui::{
    Align2, Color32, Pos2, Rect, Sense, Stroke, TextFormat, Ui, Vec2,
    emath::Rot2,
    epaint::{CubicBezierShape, PathShape, QuadraticBezierShape, TextShape},
    text::LayoutJob,
    vec2,
};
use ritm_core::turing_machine::TuringExecutionSteps;

/// Draw every transition of the turing machine
pub fn show(app: &mut App, ui: &mut Ui) -> Result<(), RitmError> {
    // We use this binary tree mapping structure to group every transition by source and target state
    let mut transitions_hashmap: BTreeMap<(usize, usize), Vec<(TransitionId, bool)>> =
        BTreeMap::new();

    // Used to compute the center of every state position
    let mut neighbors: HashMap<usize, HashSet<usize>> = HashMap::new();

    let transition_data: Vec<(usize, usize, usize)> = app
        .turing
        .tm
        .graph_ref()
        .get_transitions_hashmap()
        .iter()
        .map(|(k, v)| (k.0, k.1, v.len()))
        .collect();
    for (source_id, target_id, count) in transition_data {
        for i in 0..count {
            if source_id != target_id {
                neighbors
                    .entry(source_id)
                    .or_insert(HashSet::from([target_id]))
                    .insert(target_id);
                neighbors
                    .entry(target_id)
                    .or_insert(HashSet::from([source_id]))
                    .insert(source_id);
            }

            // check if the current transition has been used to get to the current state
            let transition_taken = match &app.turing.current_step {
                TuringExecutionSteps::FirstIteration { .. } => None,
                TuringExecutionSteps::TransitionTaken {
                    previous_state,
                    reached_state,
                    transition_index,
                    ..
                } => Some((
                    previous_state.get_id(),
                    transition_index.1,
                    reached_state.get_id(),
                )),
                TuringExecutionSteps::Backtracked { .. } => None,
            };

            let is_previous = transition_taken.is_some_and(|f| f == (source_id, i, target_id));

            match transitions_hashmap.entry((source_id, target_id)) {
                Entry::Occupied(mut e) => {
                    e.get_mut()
                        .push(((source_id, i, target_id).into(), is_previous));
                }
                Entry::Vacant(e) => {
                    e.insert(vec![((source_id, i, target_id).into(), is_previous)]);
                }
            }
        }
    }

    for ((source, target), transitions) in transitions_hashmap.iter() {
        if source == target {
            let transition_vec = app.turing.best_vector(*source)?;
            let source_position = app.turing.get_state(*source)?.inner_state.position;

            let placement = draw_self_arrow(app, ui, source_position, transition_vec)?;
            draw_labels(app, ui, transitions, placement)?;
        } else {
            let transitions_keys = app.turing.tm.graph_ref().get_transitions_hashmap();
            let reverse = transitions_keys.contains_key(&(*source, *target)) && transitions_keys.contains_key(&(*target, *source));

            let target_position = app.turing.get_state(*target)?.inner_state.position;
            let source_position = app.turing.get_state(*source)?.inner_state.position;
            let placement = draw_arrow(app, ui, source_position, target_position, Some(reverse))?;
            draw_labels(app, ui, transitions, placement)?;
        }
    }
    Ok(())
}

/// Draw transition between 2 different state
pub fn draw_arrow(
    app: &mut App,
    ui: &mut Ui,
    source: Pos2,
    target: Pos2,
    reverse: Option<bool>,
) -> Result<(Pos2, Vec2), RitmError> {
    // compute the center between the 2 states
    let center = Pos2::new((source.x + target.x) / 2.0, (source.y + target.y) / 2.0);

    let graph_center = app.turing.graph_center();

    // compute the direction of the curve
    let mut delta = (source - target).rot90().normalized();
    let need_to_flip = utils::distance(center + delta, graph_center)
        < utils::distance(center - delta, graph_center);

    // If there is 2-way transitions, then we arbitrary choose one to be inversed
    // let reversed = reverse.unwrap_or(false);

    // trust me bro, it's a xor operation
    delta = if reverse.is_some_and(|f| !f) && need_to_flip {
        -delta
    } else {
        delta
    };

    // points of the curve
    let points = [
        source,
        (center + delta * Constant::TRANSITION_CURVATURE * 2.0),
        target,
    ];

    // draw the bezier
    ui.painter().add(QuadraticBezierShape::from_points_stroke(
        points,
        false,
        Color32::TRANSPARENT,
        Stroke::new(Constant::TRANSITION_THICKNESS, Color32::BLACK),
    ));

    // compute the curve lenght to find where to draw the triangle
    let curve_lenght = get_quadratic_len(points, 100);
    let offset_pos = quadraticbeziercurve(
        points,
        map(
            &curve_lenght,
            100,
            1.0 - Constant::STATE_RADIUS / curve_lenght.last().expect("list shouldn't be empty"),
        ),
    );
    let (arrow_position, arrow_direction) = if reverse.is_some() {
        (offset_pos, (offset_pos - target.to_vec2()).normalized())
    } else {
        let previous_point = quadraticbeziercurve(points, map(&curve_lenght, 100, 0.99));
        (
            target.to_vec2(),
            (previous_point - target.to_vec2()).normalized(),
        )
    };
    // let arrow_direction = (arrow_position - target.to_vec2()).normalized();

    // compute vertices of triangle
    let triangles = vec![
        arrow_position.to_pos2(),
        (arrow_position
            + arrow_direction * Constant::ARROW_SIZE
            + arrow_direction.rot90() * Constant::ARROW_SIZE / 2.0)
            .to_pos2(),
        (arrow_position + arrow_direction * Constant::ARROW_SIZE
            - arrow_direction.rot90() * Constant::ARROW_SIZE / 2.0)
            .to_pos2(),
    ];

    // draw triangle
    ui.painter().add(PathShape::convex_polygon(
        triangles,
        Color32::BLACK,
        Stroke::NONE,
    ));

    Ok((center, delta * Constant::TRANSITION_CURVATURE))
}

/// Draw self-transition, aka with the target being the source
pub fn draw_self_arrow(
    _app: &mut App,
    ui: &mut Ui,
    state_position: Pos2,
    transition_vec: Vec2,
) -> Result<(Pos2, Vec2), RitmError> {
    // normalize the delta
    let delta = -transition_vec.normalized();

    let size = Constant::SELF_TRANSITION_SIZE * 1.33;
    let points = [
        state_position,
        state_position + (delta * size) + (delta.rot90() * size * 0.69),
        state_position + (delta * size) - (delta.rot90() * size * 0.69),
        state_position,
    ];

    ui.painter().add(CubicBezierShape::from_points_stroke(
        points,
        false,
        Color32::TRANSPARENT,
        Stroke::new(Constant::TRANSITION_THICKNESS, Color32::BLACK),
    ));

    // we get the arrow position on the curve
    let n = 100;
    let curve_lenght = get_cubic_len(points, n);
    let arrow_position = (
        cubicbeziercurve(
            points,
            map(
                &curve_lenght,
                n,
                1.0 - (Constant::STATE_RADIUS - 5.0) / curve_lenght.last().unwrap(),
            ),
        ),
        cubicbeziercurve(
            points,
            map(
                &curve_lenght,
                n,
                1.0 - (Constant::STATE_RADIUS + Constant::ARROW_SIZE / 2.0 - 5.0)
                    / curve_lenght.last().unwrap(),
            ),
        ),
    );
    let arrow_direction = (arrow_position.1 - arrow_position.0).normalized();

    // points of the triangle
    let triangles = vec![
        arrow_position.0.to_pos2(),
        (arrow_position.1
            + arrow_direction * Constant::ARROW_SIZE
            + arrow_direction.rot90() * Constant::ARROW_SIZE / 2.0)
            .to_pos2(),
        (arrow_position.1 + arrow_direction * Constant::ARROW_SIZE
            - arrow_direction.rot90() * Constant::ARROW_SIZE / 2.0)
            .to_pos2(),
    ];

    // draw the triangle
    ui.painter().add(PathShape::convex_polygon(
        triangles,
        Color32::BLACK,
        Stroke::NONE,
    ));

    Ok((state_position, delta * Constant::SELF_TRANSITION_SIZE))
}

/// draw the transitions rules as superposed label
fn draw_labels(
    app: &mut App,
    ui: &mut Ui,
    transitions: &[(TransitionId, bool)],
    placement: (Pos2, Vec2),
) -> Result<(), RitmError> {
    // compute the position of the apsis of the curved transition
    let position = placement.0 + placement.1;

    // debug
    // ui.painter().circle(position, 2.0, Color32::RED, Stroke::NONE);

    let sample_transition_id = &transitions[0].0;
    let source_state = app.turing.get_state(sample_transition_id.source_id)?;
    let target_state = app.turing.get_state(sample_transition_id.target_id)?;

    let vector = if sample_transition_id.source_id != sample_transition_id.target_id {
        source_state.inner_state.position - target_state.inner_state.position
    } else {
        (source_state.inner_state.position - position).rot90()
    };

    let angle = ((vector.angle() + PI / 2.0 - 0.00001).rem_euclid(PI)) - PI / 2.0;

    let reverse = placement.1.y.is_sign_positive();
    let selected = app
        .graph
        .selected_transitions
        .is_some_and(|transitions| transitions == *sample_transition_id);

    let mut clicked = false;

    // place each rules
    for (i, (transition_id, is_previous)) in transitions.iter().enumerate() {
        let transition = app.turing.get_transition(*transition_id)?;
        let text = transition.info.to_string();

        let job = LayoutJob::single_section(
            text,
            TextFormat {
                font_id: if *is_previous {
                    Font::bold()
                } else {
                    Font::default_medium()
                },
                color: if selected {
                    app.theme.selection
                } else if *is_previous {
                    app.theme.highlight
                } else {
                    Theme::constrast_color(app.theme.secondary)
                },
                ..Default::default()
            },
        );

        let galley = ui.painter().layout_job(job);
        let text_height = galley.size().y;

        let bounding_rect =
            Rect::from_center_size(Pos2::ZERO, galley.size()).rotate_bb(Rot2::from_angle(angle));

        let row_i = if reverse {
            i
        } else {
            transitions.len() - 1 - i
        };

        // From this point, the code has been summoned by pain (trigonometry) and blood (my tear)
        let position_rect = Rect::from_center_size(
            position
                + vec2(
                    (-text_height * row_i as f32 - text_height / 2.0) * angle.sin(),
                    (text_height * row_i as f32 + text_height / 2.0) * angle.cos(),
                ) * if reverse { 1.0 } else { -1.0 },
            bounding_rect.size(),
        );

        let anchor: Pos2 = if angle.sin().is_sign_positive() && angle.cos().is_sign_positive() {
            Pos2::new(angle.sin() * text_height, 0.0)
        } else if angle.sin().is_sign_positive() && angle.cos().is_sign_negative() {
            Pos2::new(position_rect.width(), -angle.cos() * text_height)
        } else if angle.sin().is_sign_negative() && angle.cos().is_sign_negative() {
            Pos2::new(
                position_rect.width() - angle.sin() * text_height,
                position_rect.height(),
            )
        } else {
            Pos2::new(0.0, position_rect.height() - angle.cos() * text_height)
        };

        ui.painter_at(position_rect).add(
            TextShape::new(
                position_rect.left_top() + anchor.to_vec2(),
                galley,
                Color32::BLACK,
            )
            .with_angle_and_anchor(angle, Align2::LEFT_TOP),
        );

        let sub_count = 4;
        let size = (position_rect.size() / sub_count as f32).max(Vec2::splat(text_height));
        let (min, max) = if angle >= 0.0 {
            (
                position_rect.left_top() + size / 2.0,
                position_rect.right_bottom() - size / 2.0,
            )
        } else {
            (
                position_rect.left_bottom() + (size / 2.0) * vec2(1.0, -1.0),
                position_rect.right_top() - (size / 2.0) * vec2(1.0, -1.0),
            )
        };

        for i in 0..sub_count + 1 {
            let vec = (max - min) / sub_count as f32;
            let rect = Rect::from_center_size(min + vec * i as f32, size);
            ui.allocate_rect(rect, Sense::click())
                .clicked()
                .then(|| clicked = true);
        }
    }

    if clicked {
        app.graph.select_transitions(*sample_transition_id);
        app.edit.is_adding_transition = false;
        app.edit.is_adding_state = false;
    }
    Ok(())
}

/// return a point on the curve of a quadratic bezier
fn quadraticbeziercurve(points: [Pos2; 3], t: f32) -> Vec2 {
    let x = (1.0 - t).powi(2) * points[0].x
        + 2.0 * (1.0 - t) * t * points[1].x
        + t.powi(2) * points[2].x;
    let y = (1.0 - t).powi(2) * points[0].y
        + 2.0 * (1.0 - t) * t * points[1].y
        + t.powi(2) * points[2].y;
    Vec2::new(x, y)
}

/// return a point on the curve of a cubic bezier
fn cubicbeziercurve(points: [Pos2; 4], t: f32) -> Vec2 {
    let x = (1.0 - t).powi(3) * points[0].x
        + 3.0 * (1.0 - t).powi(2) * t * points[1].x
        + 3.0 * (1.0 - t) * t.powi(2) * points[2].x
        + t.powi(3) * points[3].x;
    let y = (1.0 - t).powi(3) * points[0].y
        + 3.0 * (1.0 - t).powi(2) * t * points[1].y
        + 3.0 * (1.0 - t) * t.powi(2) * points[2].y
        + t.powi(3) * points[3].y;
    Vec2::new(x, y)
}

fn get_cubic_len(points: [Pos2; 4], n: usize) -> Vec<f32> {
    let mut arc_length: Vec<f32> = vec![0.0; n + 1];

    let mut origin = cubicbeziercurve(points, 0.0);
    let mut clen = 0.0;
    for (i, j) in arc_length.iter_mut().enumerate().skip(1).take(n) {
        let pos = cubicbeziercurve(points, i as f32 * (1.0 / n as f32));
        let delta = origin - pos;
        clen += (delta.x.powi(2) + delta.y.powi(2)).sqrt();
        *j = clen;
        origin = pos;
    }

    arc_length
}

fn get_quadratic_len(points: [Pos2; 3], n: usize) -> Vec<f32> {
    let mut arc_length: Vec<f32> = vec![0.0; n + 1];

    let mut origin = quadraticbeziercurve(points, 0.0);
    let mut clen = 0.0;
    for (i, j) in arc_length.iter_mut().enumerate().skip(1).take(n) {
        let pos = quadraticbeziercurve(points, i as f32 / (n + 1) as f32);
        let delta = origin - pos;
        clen += (delta.x.powi(2) + delta.y.powi(2)).sqrt();
        *j = clen;
        origin = pos;
    }

    arc_length
}

fn map(len: &[f32], n: usize, t: f32) -> f32 {
    let target = t * len[n - 1];
    let mut low = 0;
    let mut high = n;
    let mut i = 0;

    while low < high {
        i = low + ((high - low) / 2);

        if len[i] < target {
            low = i + 1;
        } else {
            high = i;
        }
    }

    if len[i] > target && i != 0 {
        i -= 1;
    }

    let before = len[i];

    if before == target {
        i as f32 / n as f32
    } else {
        (i as f32 + (target - before) / (len[i + 1] - before)) / n as f32
    }
}
