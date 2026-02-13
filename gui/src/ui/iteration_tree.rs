use std::collections::{HashMap, hash_map::Entry};

use egui::{
    Align, Color32, Label, Layout, Margin, Pos2, Rect, RichText, Scene, ScrollArea, Stroke,
    TextEdit, TextFormat, Ui, pos2, scroll_area::ScrollBarVisibility, text::LayoutJob, vec2,
};
use rand::rand_core::le;
use ritm_core::{
    turing_graph::TuringStateType,
    turing_machine::TuringExecutionSteps,
    turing_tape::{TuringReadingTape, TuringWritingTape},
};

use crate::{
    App,
    error::RitmError,
    ui::{constant::Constant, font::Font, theme::Theme},
};

const SIBLING_DIST: f32 = 200.;
const CHILD_DIST: f32 = 200.;
const DIST_BTW_TREES: f32 = 200.;

#[derive(Debug)]
pub enum NodeType {
    Normal,
    Rejecting,
    Accepting,
}

impl NodeType {
    fn get_node_color(&self) -> Color32 {
        match self {
            NodeType::Normal => Color32::WHITE,
            NodeType::Rejecting => Color32::RED,
            NodeType::Accepting => Color32::GREEN,
        }
    }
}

#[derive(Debug)]
pub struct UiNode {
    x: f32,
    y: f32,
    mod_val: f32,
    shift_val: f32,
    reading_tape: TuringReadingTape,
    writting_tapes: Vec<TuringWritingTape>,
    node_type: NodeType,

    children: Vec<usize>,
    parent_id: Option<usize>,
}

impl UiNode {
    fn show(&self, ui: &mut Ui, id: usize, is_last: bool) {
        ui.painter().circle(
            pos2(self.x, self.y),
            Constant::STATE_RADIUS,
            self.node_type.get_node_color(),
            if is_last {
                Stroke::new(5., Color32::BLUE)
            } else {
                Stroke::new(1., Color32::BLACK)
            },
        );
        let name = RichText::new(id.to_string())
            .font(Font::default_big())
            .color(Theme::constrast_color(self.node_type.get_node_color()));

        let label = Label::new(name).wrap().halign(Align::Center);

        let rect = Rect::from_center_size(
            pos2(self.x, self.y),
            vec2(Constant::STATE_RADIUS, Constant::STATE_RADIUS) * 2.0,
        );

        // Draw the label inside the node, without overflow
        ui.put(rect, label);
    }
}

pub enum Node {
    Future,
    Explored {
        reading_tape: TuringReadingTape,
        writting_tapes: Vec<TuringWritingTape>,
        node_type: NodeType,
    },
}

// #[derive(Debug)]
// pub struct IterationNode {
//     reading_tape: TuringReadingTape,
//     writting_tapes: Vec<TuringWritingTape>,

//     children: Vec<usize>,
// }
impl IterationTree {
    pub fn show(&mut self, ui: &mut Ui) -> Result<(), RitmError> {
        if self.nodes.is_empty() {
            return Ok(());
        }
        self.init_xy_2(0, 0, 0);

        self.final_pass(0, 0.);

        self.draw_node(ui, 0);
        Ok(())
    }

    pub fn draw_node(&mut self, ui: &mut Ui, current_id: usize) {
        for child_id in self.nodes[current_id].children.clone() {
            // draw line btw root and child
            ui.painter().line_segment(
                [
                    pos2(self.nodes[current_id].x, self.nodes[current_id].y),
                    pos2(self.nodes[child_id].x, self.nodes[child_id].y),
                ],
                Stroke::new(5., Color32::BLACK),
            );
            self.draw_node(ui, child_id);
        }

        // Draw root after child and lines to cover them
        self.nodes[current_id].show(ui, current_id, false);
    }

    fn has_children(&self, id: usize) -> bool {
        !self.nodes[id].children.is_empty()
    }

    fn get_parent(&self, id: usize) -> &UiNode {
        &self.nodes[self.nodes[id].parent_id.expect("a parent id")]
    }

    fn get_node_mut(&mut self, id: usize) -> &mut UiNode {
        &mut self.nodes[id]
    }
    fn get_node(&self, id: usize) -> &UiNode {
        &self.nodes[id]
    }

    fn init_xy_2(&mut self, current_id: usize, sibling_id: usize, depth: usize) {
        // Init children first
        for child_id in self.nodes[current_id].children.clone().iter().enumerate() {
            self.init_xy_2(*child_id.1, child_id.0, depth + 1);
        }

        self.get_node_mut(current_id).x = 0.;
        self.get_node_mut(current_id).y = depth as f32 * CHILD_DIST;

        // If the leftmost of the children of its parent
        if sibling_id == 0 {
            if !self.has_children(current_id) {
                self.get_node_mut(current_id).x = 0.;
            } else {
                // Midway point from children
                let first = self
                    .get_node(
                        *self
                            .get_node(current_id)
                            .children
                            .first()
                            .expect("at least one node present"),
                    )
                    .x;
                let last = self
                    .get_node(
                        *self
                            .get_node(current_id)
                            .children
                            .last()
                            .expect("at least one node present"),
                    )
                    .x;

                self.get_node_mut(current_id).x = (last - first) / 2. + first;
            }
        } else {
            println!(
                "node_id: {current_id} - sibling id : {sibling_id} - prev_sibling_id : {:?} : x= {}",
                self.get_parent(current_id).children[sibling_id - 1],
                self.get_node(self.get_parent(current_id).children[sibling_id - 1])
                    .x
            );
            // get previous sibling position :
            let prev_sibling = self.get_parent(current_id).children[sibling_id - 1];
            self.get_node_mut(current_id).x = self.get_node(prev_sibling).x + SIBLING_DIST;

            // If has children, use mod to move them later
            if self.has_children(current_id) {
                // Midway point from children
                let first = self
                    .get_node(*self.get_node(current_id).children.first().expect("present"))
                    .x;
                let last = self
                    .get_node(*self.get_node(current_id).children.last().expect("present"))
                    .x;

                // parent x value - middle point of the first and last child
                self.get_node_mut(current_id).mod_val =
                    self.get_node(current_id).x - ((last - first) / 2. + first);
            }
            // Correct nodes
            self.correct_conflicts(current_id, sibling_id);
        }
    }

    fn correct_conflicts(&mut self, current_id: usize, sibling_id: usize) {
        // Suppose this node is not the leftmost node
        let mut tree_left_contour = HashMap::new();
        self.left_contour(current_id, 0, 0., &mut tree_left_contour);

        // Check for conflicts with siblings
        for sibling in 0..sibling_id {
            // Check for conflicts with this child
            let mut sibling_right_contour = HashMap::new();
            self.right_contour(
                self.get_parent(current_id).children[sibling],
                0,
                0.,
                &mut sibling_right_contour,
            );

            // Compare contours :
            let same_depths = tree_left_contour
                .keys()
                .max()
                .expect("one key present")
                .min(sibling_right_contour.keys().max().expect("one key present"));

            // for each level
            let mut max_shift: f32 = 0.;
            for depths in 0..*same_depths {
                if sibling_right_contour[&depths] < tree_left_contour[&depths] + DIST_BTW_TREES {
                    max_shift = max_shift.max(
                        DIST_BTW_TREES
                            - (sibling_right_contour[&depths] - tree_left_contour[&depths]),
                    );
                }
            }

            // update all siblings except first if needed
            if max_shift != 0. {
                for sibling_to_update in 1..self.get_parent(current_id).children.len() {
                    let sibling_to_update_id =
                        self.get_parent(current_id).children[sibling_to_update];
                    self.get_node_mut(sibling_to_update_id).shift_val +=
                        max_shift * (sibling_to_update as f32 / current_id as f32);
                    println!(
                        "Because of {current_id}, Node {sibling_to_update_id} has to shift {}",
                        self.get_node_mut(sibling_to_update_id).shift_val
                    )
                }
            }
        }
    }

    fn right_contour(
        &mut self,
        root_id: usize,
        curr_level: usize,
        mod_acc: f32,
        right_levels: &mut HashMap<usize, f32>,
    ) {
        let mut level = right_levels
            .entry(curr_level)
            .insert_entry(self.get_node(root_id).x);

        *level.get_mut() = level.get().max(self.get_node(root_id).x + mod_acc);

        let new_acc = mod_acc + self.get_node(root_id).mod_val;

        for child_id in self.get_node(root_id).children.clone() {
            self.right_contour(child_id, curr_level + 1, new_acc, right_levels);
        }
    }

    fn left_contour(
        &mut self,
        root_id: usize,
        curr_level: usize,
        mod_acc: f32,
        right_levels: &mut HashMap<usize, f32>,
    ) {
        let mut level = right_levels
            .entry(curr_level)
            .insert_entry(self.get_node(root_id).x);

        *level.get_mut() = level.get().min(self.get_node(root_id).x + mod_acc);

        let new_acc = mod_acc + self.get_node(root_id).mod_val;

        for child_id in self.get_node(root_id).children.clone() {
            self.left_contour(child_id, curr_level + 1, new_acc, right_levels);
        }
    }

    fn final_pass(&mut self, current_id: usize, acc_mod: f32) {
        for child_id in self.get_node(current_id).children.clone() {
            self.final_pass(
                child_id,
                acc_mod + self.get_node(current_id).mod_val + self.get_node(current_id).shift_val,
            );
        }

        // Mod changes children x, while shift changes this node and its children position
        self.get_node_mut(current_id).x += self.get_node(current_id).shift_val + acc_mod;

        self.get_node_mut(current_id).shift_val = 0.;
        self.get_node_mut(current_id).mod_val = 0.;
    }
}

#[derive(Debug)]
pub struct IterationTree {
    nodes: Vec<UiNode>,
    next_parent: usize,
    scene_rect: Rect,
}

impl IterationTree {
    pub fn add_step(&mut self, step: TuringExecutionSteps) {
        if self.nodes.len() > self.next_parent {
            self.nodes[self.next_parent]
                .children
                .push(step.get_nb_iterations());
        }
        let mut parent_id = Some(self.next_parent);

        let mut node_type = NodeType::Normal;

        let (reading_tape, writting_tapes) = match step {
            TuringExecutionSteps::FirstIteration {
                init_state: _,
                init_reading_tape,
                init_write_tapes,
            } => {
                // Recreate a new iteration tree
                *self = IterationTree::default();
                self.next_parent = 0;
                parent_id = None;
                (init_reading_tape, init_write_tapes)
            }
            TuringExecutionSteps::TransitionTaken {
                previous_state: _,
                transition_index: _,
                reached_state,
                state_pointer: _,
                transition_taken: _,
                reading_tape,
                writing_tapes,
                iteration,
            } => {
                if let TuringStateType::Accepting = reached_state.get_type() {
                    node_type = NodeType::Accepting;
                }
                self.next_parent = iteration;
                (reading_tape, writing_tapes)
            }
            TuringExecutionSteps::Backtracked {
                previous_state: _,
                reached_state: _,
                state_pointer: _,
                reading_tape,
                writing_tapes,
                iteration: _,
                backtracked_iteration,
            } => {
                node_type = NodeType::Rejecting;

                // This iteration is a child to a transition
                self.next_parent = backtracked_iteration;
                (reading_tape, writing_tapes)
            }
        };
        self.nodes.push(UiNode {
            reading_tape,
            writting_tapes,
            children: Vec::new(),
            x: 0.,
            y: 0.,
            mod_val: 0.,
            shift_val: 0.,
            node_type,
            parent_id,
        });
    }
}

impl Default for IterationTree {
    fn default() -> Self {
        Self {
            next_parent: 0,
            scene_rect: Rect::ZERO,
            nodes: Vec::new(),
        }
    }
}

/// Display the code section of the application
pub fn show(app: &mut App, ui: &mut Ui) -> Result<(), RitmError> {
    // current rect of the element inside the scene
    let mut inner_rect = Rect::ZERO;

    let mut scene_rect = app.tree.scene_rect;

    // println!("TREE: {:?}", app.tree);

    let scene_response = Scene::new()
        .zoom_range(0.0..=1.5)
        .show(ui, &mut scene_rect, |ui| {
            // // Draw the transitions of the turing machine
            // transition::show(app, ui)?;
            let root_position = Pos2::new(0., 0.);

            // Iterations are stored as a stack, so we reverse it to show the tree
            for (i, iteration) in app.turing.tm.get_memory().iter().rev().enumerate() {
                let i = i as f32;
            }
            if !app.tree.nodes.is_empty() {
                app.tree.show(ui)?;
            }

            // This Rect can be used to "Reset" the view of the graph
            inner_rect = ui.min_rect();

            Ok::<(), RitmError>(())
        })
        .response;

    app.tree.scene_rect = scene_rect;

    Ok(())
}
