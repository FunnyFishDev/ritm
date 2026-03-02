use egui::{
    Align, Align2, Atom, AtomExt, Button, Color32, Frame, Id, Image, ImageButton, Label, Layout, Margin, RichText, ScrollArea, Stroke, TextEdit, TextFormat, Ui, Vec2, include_image, scroll_area::ScrollBarVisibility, text::LayoutJob, vec2
};
use ritm_core::turing_parser::TuringParserError;

use crate::{
    App,
    error::{GuiError, RitmError},
    ui::{
        font::Font,
        tutorial::{TutorialBox, TutorialEnum},
    },
};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Code {
    tabs: Vec<Tab>,
    code_closed: bool,
    current_tab: usize,
    #[serde(skip)]
    editing_name: bool,
    auto_scroll: bool,
    // The current parsing error
    #[serde(skip)]
    curr_parsing_error: Option<TuringParserError>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Tab {
    name: String,
    code: String,
}

impl Default for Code {
    fn default() -> Self {
        Self {
            tabs: vec![Tab {
                code: "accepting = q_a;

// Initialisation
q_i {ç, ç -> R, ç, R} q_copy;

// Copy to write ribbon
q_copy {0, _ -> R, 0, R} q_copy;
q_copy {1, _ -> R, 1, R} q_copy;
q_copy {$, _ -> L, _, N} q_return;

// Reset reading ribbon position
q_return {0, _ -> L, _, N} q_return;
q_return {1, _ -> L, _, N} q_return;
q_return {ç, _ -> R, _, L} q_check;

// Compare each side until end
q_check {0, 0 -> R, 0, L} q_check;
q_check {1, 1 -> R, 1, L} q_check;
q_check {$, ç -> N, ç, N} q_a;"
                    .to_string(),
                name: "binary_palindrome".to_string(),
            }],
            code_closed: Default::default(),
            current_tab: 0,
            editing_name: false,
            auto_scroll: false,
            curr_parsing_error: None,
        }
    }
}

impl Code {
    pub fn current_code(&self) -> Result<String, RitmError> {
        Ok(self
            .tabs
            .get(self.current_tab)
            .ok_or(RitmError::GuiError(GuiError::InvalidApplicationState))?
            .code
            .clone())
    }

    pub fn current_code_mut(&mut self) -> Result<&mut String, RitmError> {
        Ok(&mut self
            .tabs
            .get_mut(self.current_tab)
            .ok_or(RitmError::GuiError(GuiError::InvalidApplicationState))?
            .code)
    }

    pub fn tab_name_check(&mut self) {
        // If empty then default name
        if self.tabs[self.current_tab].name.is_empty() {
            self.tabs[self.current_tab].name = self.tab_name();
            return;
        }

        let mut flag = 1;
        while flag > 0 && self.tabs.len() > 1 {
            flag -= 1;
            if self
                .tabs
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != self.current_tab)
                .any(|(_, t)| t.name == self.tabs[self.current_tab].name)
            {
                self.tabs[self.current_tab].name.push('2');
                flag += 1;
            };
        }
    }

    pub fn new_tab(&mut self, tab_name: String, code: String) {
        self.tabs.push(Tab {
            code,
            name: tab_name,
        });

        self.auto_scroll = true;
        self.switch_to(self.tabs.len() - 1);
        self.tab_name_check();
    }

    pub fn is_closed(&self) -> bool {
        self.code_closed
    }

    pub fn close(&mut self) {
        self.code_closed = true;
    }

    pub fn open(&mut self) {
        self.code_closed = false;
    }

    pub(crate) fn tab_name(&self) -> String {
        format!("tab{}", self.tabs.len() + 1)
    }

    pub(crate) fn toggle(&mut self) {
        self.code_closed ^= true;
    }

    pub(crate) fn switch_to(&mut self, id: usize) {
        if self.tabs.len() <= id {
            return;
        }

        self.editing_name = false;
        self.current_tab = id;
        self.curr_parsing_error = None;
    }

    pub(crate) fn set_curr_parsing_error(&mut self, error: Option<TuringParserError>) {
        self.curr_parsing_error = error;
    }
}

pub fn show(app: &mut App, ui: &mut Ui) -> Result<(), RitmError> {
    app.tutorial.add_boxe(
        "code_section",
        TutorialBox::new(ui.available_rect_before_wrap()).with_align(Align2::RIGHT_CENTER),
    );

    let tab_response = Frame::new()
        .fill(Color32::from_gray(128).blend(app.theme.code_background.gamma_multiply_u8(240)))
        .show(ui, |ui| {
            ScrollArea::horizontal()
                .id_salt("tabs")
                .max_height(30.0)
                .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
                        let mut marked_to_delete: Vec<usize> = vec![];

                        // Iterate over the tabs
                        for i in 0..app.code.tabs.len() {
                            let is_current_tab = app.code.current_tab == i;

                            // Tab frame
                            let frame = Frame::new()
                                .fill(if !is_current_tab {
                                    Color32::from_gray(128)
                                        .blend(app.theme.code_background.gamma_multiply_u8(210))
                                } else {
                                    app.theme.code_background
                                })
                                .inner_margin(Margin {
                                    left: 8,
                                    right: 2,
                                    ..Default::default()
                                })
                                .show(ui, |ui| {
                                    // Define the id for the sub button
                                    let text_edit_id = Id::new("text_edit");
                                    let delete_button_id = Id::new("delete_button");

                                    // Get the tab data
                                    let tab = &mut app.code.tabs[i];

                                    // Layout the button
                                    ui.spacing_mut().icon_spacing = 4.0;
                                    ui.spacing_mut().button_padding = vec2(0.0, 0.0);
                                    let button = Button::new((
                                        if is_current_tab && app.code.editing_name {
                                            Atom::custom(
                                                text_edit_id,
                                                vec2(
                                                    Font::get_width_word(
                                                        ui,
                                                        &Font::default_big(),
                                                        &app.code.tabs[app.code.current_tab].name,
                                                    )
                                                    .max(30.0)
                                                        + 5.0,
                                                    Font::BIG_SIZE,
                                                ),
                                            )
                                        } else {
                                            RichText::new(tab.name.clone())
                                                .color(app.theme.code)
                                                .font(Font::default_big())
                                                .into()
                                        },
                                        Atom::custom(delete_button_id, Vec2::splat(Font::BIG_SIZE))
                                            .atom_size(vec2(25.0, 25.0)),
                                    ))
                                    .frame(false)
                                    .fill(app.theme.code_background)
                                    .atom_ui(ui);

                                    // Textedit
                                    // TODO: change text_edit color
                                    if let Some(rect) = button.rect(text_edit_id) {
                                        let text_edit = TextEdit::singleline(
                                            &mut app.code.tabs[app.code.current_tab].name,
                                        )
                                        .margin(Margin::symmetric(
                                            2,
                                            -((Font::get_heigth(ui, &Font::default_big())
                                                - Font::BIG_SIZE)
                                                / 2.0)
                                                as i8,
                                        ))
                                        .font(Font::default(Font::BIG_SIZE))
                                        .background_color(app.theme.code_background)
                                        .frame(false)
                                        .text_color(app.theme.code);

                                        let response = ui.put(rect, text_edit);

                                        if response.lost_focus() {
                                            app.code.tab_name_check();
                                            app.code.editing_name = false;
                                        }

                                        if app.code.editing_name {
                                            response.request_focus();
                                        }

                                        // TODO: maybe reenable this ?
                                        // no (i mean put a setting at least)
                                        // response.request_focus();
                                    }

                                    if !is_current_tab && button.response.clicked() {
                                        app.code.switch_to(i);
                                    }

                                    ui.visuals_mut().widgets.hovered.weak_bg_fill =
                                        if !is_current_tab {
                                            Color32::from_gray(128).blend(
                                                app.theme.code_background.gamma_multiply_u8(180),
                                            )
                                        } else {
                                            Color32::from_gray(128).blend(
                                                app.theme.code_background.gamma_multiply_u8(210),
                                            )
                                        };
                                    ui.visuals_mut().widgets.inactive.weak_bg_fill =
                                        if !is_current_tab {
                                            Color32::from_gray(128).blend(
                                                app.theme.code_background.gamma_multiply_u8(210),
                                            )
                                        } else {
                                            app.theme.code_background
                                        };
                                    ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::NONE;
                                    ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::NONE;
                                    ui.visuals_mut().widgets.active.bg_stroke = Stroke::NONE;
                                    ui.spacing_mut().button_padding.x = 0.0;

                                    if !(app.code.editing_name)
                                        && ui.rect_contains_pointer(button.response.rect)
                                        && let Some(rect) = button.rect(delete_button_id)
                                        && ui
                                            .put(
                                                rect,
                                                ImageButton::new(
                                                    Image::new(include_image!(
                                                        "../../assets/icon/close_small.svg"
                                                    ))
                                                    .shrink_to_fit()
                                                    .tint(app.theme.code),
                                                ),
                                            )
                                            .clicked()
                                    {
                                        marked_to_delete.push(i);
                                    }

                                    if button.response.double_clicked() {
                                        app.code.editing_name = true;
                                    }

                                    Ok::<(), RitmError>(())
                                })
                                .response;

                            // Only add tutorial if this is the first tab
                            if i == 0 {
                                app.tutorial.add_boxe(
                                    "tab_rename",
                                    TutorialBox::new(frame.rect).with_align(Align2::RIGHT_CENTER),
                                );

                                if let Some(tutorial) = app.tutorial.current_tutorial()
                                    && tutorial == TutorialEnum::Code
                                    && app.tutorial.current_step() == 2
                                {
                                    frame.scroll_to_me(Some(Align::Min));
                                    app.code.auto_scroll = false
                                }
                            }
                        }

                        // ui.set_max_width(ui.available_width() + 50.0);
                        let plus = Frame::new()
                            .fill(
                                Color32::from_gray(128)
                                    .blend(app.theme.code_background.gamma_multiply_u8(210)),
                            )
                            .inner_margin(vec2(8.0, 0.0))
                            .show(ui, |ui| {
                                ui.add(
                                    ImageButton::new(
                                        Image::new(include_image!("../../assets/icon/plus.svg"))
                                            .tint(app.theme.code)
                                            .fit_to_exact_size(Vec2::splat(ui.available_height())),
                                    )
                                    .frame(false),
                                )
                            });

                        app.tutorial.add_boxe(
                            "tab_add",
                            TutorialBox::new(plus.response.rect).with_align(Align2::RIGHT_CENTER),
                        );

                        if let Some(tutorial) = app.tutorial.current_tutorial()
                            && tutorial == TutorialEnum::Code
                            && app.tutorial.current_step() == 3
                        {
                            plus.response.scroll_to_me(Some(Align::Max));
                            app.code.auto_scroll = false
                        }

                        // move the scrollbar to the last element
                        if app.code.auto_scroll {
                            plus.response.scroll_to_me(Some(Align::Max));
                            app.code.auto_scroll = false
                        }

                        // Add a tab
                        if plus.inner.clicked() {
                            app.code.new_tab("".to_string(), "".to_string());
                        }

                        // Remove the tabs closed
                        for i in marked_to_delete.iter().rev() {
                            app.code.tabs.remove(*i);
                        }

                        if app.code.tabs.is_empty() {
                            app.code.new_tab(app.code.tab_name(), "".to_string());
                        }

                        if app.code.current_tab > app.code.tabs.len() - 1 {
                            app.code.switch_to(app.code.tabs.len() - 1);
                        }

                        Ok::<(), RitmError>(())
                    });
                });
        })
        .response;

    app.tutorial.add_boxe(
        "tabs",
        TutorialBox::new(tab_response.rect).with_align(Align2::CENTER_BOTTOM),
    );

    code(app, ui)?;
    Ok(())
}

/// Display the code section of the application
pub fn code(app: &mut App, ui: &mut Ui) -> Result<(), RitmError> {
    ScrollArea::vertical()
        .id_salt("code")
        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
        .show(ui, |ui| {
            ui.allocate_ui_with_layout(
                ui.available_size(),
                Layout::left_to_right(egui::Align::Min),
                |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                    let code_width = ui.available_width()
                        - 30.0
                        - Font::get_width(ui, &Font::default_medium()) * 3.0;

                    let job = LayoutJob::simple(
                        app.code.tabs[app.code.current_tab].code.clone(),
                        Font::default_medium(),
                        Color32::PLACEHOLDER,
                        code_width,
                    );

                    let galley = ui.painter().layout_job(job);

                    let mut number: String = "".to_string();

                    for i in 1..=galley.rows.len() {
                        number.push_str(
                            &(" ".repeat(
                                ((galley.rows.len() as f32).log10() as usize
                                    - (i as f32).log10() as usize)
                                    .max(0),
                            ) + format!("{}\n", i).as_str()),
                        );
                    }
                    let mut line = 1;
                    let mut col = 1;

                    let mut layouter = |ui: &Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
                        let mut layout_job = LayoutJob::default();
                        let mut code: &str = buf.as_str();
                        while !code.is_empty() {
                            if code.starts_with("//") {
                                let end = code.find("\n").unwrap_or(code.len());
                                layout_job.append(
                                    &code[..end],
                                    0.0,
                                    TextFormat::simple(
                                        Font::default_medium(),
                                        app.theme.syntax_comment,
                                    ),
                                );
                                code = &code[end..];
                            } else {
                                let mut it = code.char_indices();
                                it.next();
                                let end = it.next().map_or(code.len(), |(idx, _chr)| idx);
                                let mut format =
                                    TextFormat::simple(Font::default_medium(), app.theme.code);
                                if &code[..end] == "\n" {
                                    line += 1;
                                    col = 0;
                                } else {
                                    col += 1;
                                }
                                if let Some(err) = &app.code.curr_parsing_error {
                                    match err {
                                        TuringParserError::FileError {
                                            given_path: _,
                                            error: _,
                                        } => (),
                                        TuringParserError::ParsingError {
                                            line_col_pos,
                                            value: _,
                                            missing_value: _,
                                        }
                                        | TuringParserError::TuringError {
                                            line_col_pos,
                                            turing_error: _,
                                            value: _,
                                        } => {
                                            if let Some(line_col) = line_col_pos
                                                && line_col.0 == line
                                            {
                                                format.underline = if line_col.1 == col {
                                                    Stroke::new(3.5, Color32::LIGHT_RED)
                                                } else {
                                                    Stroke::new(2.5, Color32::DARK_RED)
                                                }
                                            }
                                        }
                                    }
                                }
                                layout_job.append(
                                    &code[..end],
                                    0.0,
                                    format,
                                );
                                code = &code[end..];
                            }
                        }
                        layout_job.wrap.max_width = wrap_width;
                        ui.fonts(|f| f.layout_job(layout_job))
                    };

                    let code = TextEdit::multiline(&mut app.code.tabs[app.code.current_tab].code)
                        .code_editor()
                        .font(Font::default_medium())
                        .frame(false)
                        .margin(Margin::same(0))
                        .background_color(app.theme.code_background)
                        .layouter(&mut layouter);

                    let line_number = Label::new(
                        RichText::new(number)
                            .color(app.theme.text_secondary.gamma_multiply(0.5))
                            .font(Font::default_medium()),
                    )
                    .halign(egui::Align::Min)
                    .selectable(false)
                    .extend();

                    ui.add_space(10.0);
                    ui.add_sized(
                        vec2(
                            Font::get_width(ui, &Font::default_medium()) * 2.0,
                            Font::get_heigth(ui, &Font::default_medium())
                                * galley.rows.len() as f32,
                        ),
                        line_number,
                    );
                    ui.add_space(20.0);

                    let resp = ui.add_sized(ui.available_size() - vec2(5.0, 0.0), code);

                    // When the code changes, we can assume the error is irrelevant :
                    if resp.changed() {
                        app.code.curr_parsing_error = None;
                    }

                    if resp.has_focus() {
                        app.transient.listen_to_keybind = false;
                    }
                    if let Some(error) = &app.code.curr_parsing_error {
                        resp.on_hover_text_at_pointer(match error {
                            TuringParserError::FileError {
                                given_path: _,
                                error,
                            } => error.to_string(),
                            TuringParserError::ParsingError {
                                line_col_pos: _,
                                value: _,
                                missing_value,
                            } => format!(
                                "Parsing failed due to a missing value{}",
                                if let Some(val) = missing_value {
                                    format!(", try adding the following : \"{val}\".")
                                } else {
                                    ".".to_string()
                                }
                            ),
                            TuringParserError::TuringError {
                                line_col_pos: _,
                                turing_error,
                                value: _,
                            } => format!("Ran into an error while parsing a line : {turing_error}"),
                        });
                    }

                    ui.add_space(5.0);
                },
            );
        });
    Ok(())
}
