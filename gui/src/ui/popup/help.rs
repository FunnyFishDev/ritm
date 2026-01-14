use egui::{
    Align, AtomExt, AtomLayout, CentralPanel, Color32, Frame, Image, ImageButton, ImageSource,
    IntoAtoms, Label, Layout, Margin, RichText, ScrollArea, SidePanel, Stroke, Ui, Vec2,
    include_image, vec2,
};

use crate::{
    App,
    ui::font::Font,
};

pub fn show(ui: &mut Ui, app: &mut App) {
    ui.set_max_size(ui.ctx().screen_rect().size() * 0.8);
    ui.set_min_size(ui.ctx().screen_rect().size() * 0.8);

    SidePanel::left("previous")
        .exact_width(50.0)
        .frame(Frame::new().fill(Color32::TRANSPARENT))
        .show_separator_line(false)
        .resizable(false)
        .show_inside(ui, |ui| {
            if app.help_slide_index > 0 {
                ui.add_space(ui.available_height() / 2.0 - 50.0);
                Frame::new()
                    .inner_margin(0)
                    .outer_margin(0)
                    .stroke(Stroke::new(1.0, app.theme.gray))
                    .corner_radius(10)
                    .show(ui, |ui| {
                        if ui
                            .add_sized(
                                vec2(ui.available_width(), 100.0),
                                ImageButton::new(
                                    Image::new(include_image!("../../../assets/icon/left.svg"))
                                        .fit_to_exact_size(Vec2::splat(ui.available_width())),
                                )
                                .tint(app.theme.gray)
                                .frame(false),
                            )
                            .clicked()
                            && app.help_slide_index > 0
                        {
                            app.help_slide_index -= 1;
                        }
                    });
            }
        });

    SidePanel::right("next")
        .exact_width(50.0)
        .frame(Frame::new().fill(Color32::TRANSPARENT))
        .show_separator_line(false)
        .resizable(false)
        .show_inside(ui, |ui| {
            if app.help_slide_index < 6 {
                ui.add_space(ui.available_height() / 2.0 - 50.0);
                Frame::new()
                    .inner_margin(0)
                    .outer_margin(0)
                    .stroke(Stroke::new(1.0, app.theme.gray))
                    .corner_radius(10)
                    .show(ui, |ui| {
                        if ui
                            .add_sized(
                                vec2(ui.available_width(), 100.0),
                                ImageButton::new(
                                    Image::new(include_image!("../../../assets/icon/right.svg"))
                                        .fit_to_exact_size(Vec2::splat(ui.available_width())),
                                )
                                .tint(app.theme.gray)
                                .frame(false),
                            )
                            .clicked()
                            && app.help_slide_index < 6
                        {
                            app.help_slide_index += 1;
                        }
                    });
            }
        });

    CentralPanel::default()
        .frame(
            Frame::new()
                .fill(Color32::WHITE)
                .inner_margin(Margin::symmetric(15, 5)),
        )
        .show_inside(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.allocate_ui_with_layout(
                    ui.available_size(),
                    Layout::top_down(Align::Min).with_cross_justify(true),
                    |ui| {
                        ui.spacing_mut().item_spacing = vec2(0.0, 20.0);

                        match app.help_slide_index {
                            0 => welcome(app, ui),
                            1 => code(app, ui),
                            2 => settings(app, ui),
                            3 => tapes(app, ui),
                            4 => controls(app, ui),
                            5 => graph(app, ui),
                            6 => editing(app, ui),
                            _ => welcome(app, ui),
                        }
                    },
                );
            });
        });
}

fn title(text: impl Into<String>) -> Label {
    Label::new(RichText::new(text).font(Font::bold())).halign(Align::Center)
}

fn text(text: impl Into<String>) -> Label {
    Label::new(RichText::new(text).font(Font::default_medium())).halign(Align::Min)
}

fn image(app: &mut App, ui: &mut Ui, source: ImageSource) {
    Frame::new()
        .corner_radius(5)
        .stroke(Stroke::new(1.0, app.theme.gray))
        .show(ui, |ui| {
            let img = Image::new(source)
                .maintain_aspect_ratio(true)
                .corner_radius(5);
            let size = img.load_and_calc_size(ui, ui.available_size());
            if let Some(size) = size {
                ui.set_width(size.x);
            }
            ui.add(img);
        });
}

fn icon(_app: &mut App, ui: &mut Ui, text: impl Into<String>, source: ImageSource) {
    AtomLayout::new(
        (
            Image::new(source)
                .tint(Color32::BLACK)
                .atom_size(Vec2::splat(50.0)),
            RichText::new(text).font(Font::default_medium()),
        )
            .into_atoms(),
    )
    .gap(25.0)
    .show(ui);
}

fn welcome(app: &mut App, ui: &mut Ui) {
    ui.add(title("Welcome to RITM !"));

    ui.add(text("This guide will help you understand the interface, so you can build incredible machine and understand the power of turing machine !"));

    image(app, ui, include_image!("../../../assets/help/section.png"));

    ui.add( text("The interface is divided in 6 sections : The code (1), the settings (2), the tapes (3), the controls (4), the graph (5) and the editing (6)."));
}

fn code(app: &mut App, ui: &mut Ui) {
    ui.add(title("Code"));

    ui.add(text(
        "The easiest way to create and share a turing machine is by using code !",
    ));

    ui.spacing_mut().item_spacing = vec2(20.0, 20.0);
    ui.columns_const(|[ui, pic]| {

        ui.add(text("The syntax is the following : ").halign(Align::Max).wrap());
        ui.add(text("q_[name] designates a state with the name [name]. The one before the brackets is the initial state, and the one after is the target state."));
        ui.add(text("{1, 0 -> R, 2, R} is the \"rule\", where if the reading tape is 1 and the writing is 0, then both move to the right (R) and the 1 is replaced by a 2."));
        ui.add(text("The character ç, _ and $ cannot be used in the input as they are special characters. R, L and N mean Right, Left and Nothing."));

        ui.add(text("You can use comments (//) to help structure the code. Comments are ignored when the machine is build."));

        image(app, pic, include_image!("../../../assets/help/code.png"));
    });
}

fn settings(app: &mut App, ui: &mut Ui) {
    ui.add(title("Settings"));

    ui.add(text("You can find on top of the code (or the side of the screen if the code section is closed) multiples icon, each representing a feature."));

    // ui.horizontal(|ui| {
    //     ui.add((text("Open the settings of the application"), Image::new(include_image!("../../../assets/icon/setting.svg"))
    //         .tint(Color32::BLACK).maintain_aspect_ratio(true).into_atoms()));
    // });

    icon(
        app,
        ui,
        "The differents setttings of the application.",
        include_image!("../../../assets/icon/setting.svg"),
    );
    icon(
        app,
        ui,
        "Save the current code as a file with the .tm extension. You can change the filename in the dialog.",
        include_image!("../../../assets/icon/save.svg"),
    );
    icon(
        app,
        ui,
        "Show different examples of turing machine and allow the user to load its own code. The code must be in a file with the .tm extension.",
        include_image!("../../../assets/icon/machine_folder.svg"),
    );
    icon(
        app,
        ui,
        "Open the guide that you are currently reading.",
        include_image!("../../../assets/icon/help.svg"),
    );
    icon(
        app,
        ui,
        "Convert the current code into a graph, for visualisation",
        include_image!("../../../assets/icon/graph.svg"),
    );
    icon(
        app,
        ui,
        "Close the code section of the application, granting more space for the graph section",
        include_image!("../../../assets/icon/panel_close.svg"),
    );
}

fn tapes(app: &mut App, ui: &mut Ui) {
    ui.add(title("Tapes"));

    ui.add(text("The first tape is always the reading tape, where the input is set and can't be edited by the machine."));

    ui.add(text("The next tapes are the writing tapes, and can be any amount but too much will break the ui by lack of space. The number of writing tape can be changed in the settings."));

    ui.add(text("The central square of each tapes are the current characters read by the machine. At step 0, the tapes will always be on their \"ç\" character."));

    image(app, ui, include_image!("../../../assets/help/ribbons.png"));
}

fn controls(app: &mut App, ui: &mut Ui) {
    ui.add(title("Controls"));

    ui.add(text("This section, as its name indicate, control the turing machine, and is divided in 3 subsections."));
    ui.add(title("Input"));
    ui.add(text("The input is the content that will be injected in the reading tape. 'ç', '$' and '_' are not valid characters due to their special caracteristic."));
    ui.add(title("Player controls"));
    ui.add(text(
        "The user can control how the turing machine run at the center of the controls section :",
    ));

    icon(
        app,
        ui,
        "Start the autoplay of the machine, with a default interval of 1 seconds between steps.",
        include_image!("../../../assets/icon/play.svg"),
    );

    icon(
        app,
        ui,
        "Pause the autoplay of the machine.",
        include_image!("../../../assets/icon/pause.svg"),
    );

    icon(
        app,
        ui,
        "Run one step if possible and stop.",
        include_image!("../../../assets/icon/next.svg"),
    );

    icon(
        app,
        ui,
        "Reset the machine and tapes",
        include_image!("../../../assets/icon/reset.svg"),
    );

    ui.add(text("Below these icons, you can find the speed controller. You can increase and decrease the interval of time between each steps by multiple of 2, from 0.125x to 32x."));

    ui.add(title("Status"));
    ui.add(text("Finally, on the right of the control section, you can read the current steps count and the state of the machine."));
    ui.add(text(
        "[Idle] when tapes and turing machine are at their initial state.",
    ));
    ui.add(text(
        "[Accepted] when accept is the current state of the machine.",
    ));
    ui.add(text("[Refused] when refuse is the current state of the machine or the machine is stuck on any state except accept."));
    ui.add(text("[Running] in every other case."));

    image(app, ui, include_image!("../../../assets/help/controls.png"))
}

fn graph(app: &mut App, ui: &mut Ui) {
    ui.add(title("Graph"));
    ui.add(text("The graph section is the main feature of this application, as you can create a turing machine only with it, without any code !"));

    ui.add(text(
        "It make easy to visualise how a turing machine work and troubleshoot any problem !",
    ));

    ui.add(text("3 states will always be displayed and unremovable :"));
    ui.add(text("(i) : The initial state, where any machine start."));
    ui.add(text(
        "(a) : The accept state, which mean the input is accepted when the machine reach it.",
    ));
    ui.add(text(
        "(r) : The reject state, which is a way to cleanly stop the machine",
    ));

    icon(
        app,
        ui,
        "Convert the graph into code",
        include_image!("../../../assets/icon/code.svg"),
    );
    icon(
        app,
        ui,
        "You can use your mouse/finger to move, zoom and unzoom the graph around (On computer you need to use ctrl and scroll to zoom)qs",
        include_image!("../../../assets/icon/zoom.svg"),
    );

    image(app, ui, include_image!("../../../assets/help/graph.png"));
}

fn editing(app: &mut App, ui: &mut Ui) {
    ui.add(title("Editing"));

    ui.add(text("You can find the editing option on the bottom of the graph section. These options depends on the selection or the absence of one. Some option behavior can be changed in the settings."));

    icon(
        app,
        ui,
        "When toggled, the next click on the graph will create a new state.",
        include_image!("../../../assets/icon/stateplus.svg"),
    );

    icon(
        app,
        ui,
        "Recenter the graph on the screen to see the whole turing machine.",
        include_image!("../../../assets/icon/recenter.svg"),
    );

    icon(
        app,
        ui,
        "Unpin every state, applying Force algorithm on them (Simulating natural positioning for better visualisation).",
        include_image!("../../../assets/icon/unpin.svg"),
    );

    icon(
        app,
        ui,
        "When toggled, Add a transition between the selected state and the state clicked and open the transition editing menu. If a transition already exist, directly open the transition editing menu and add a new default transition.",
        include_image!("../../../assets/icon/transition.svg"),
    );

    icon(
        app,
        ui,
        "Delete the selected state/transitions.",
        include_image!("../../../assets/icon/delete.svg"),
    );

    icon(
        app,
        ui,
        "Open the transition editing menu for the selected transitions.",
        include_image!("../../../assets/icon/edit.svg"),
    );
}

fn _keybind(app: &mut App, ui: &mut Ui) {
    ui.add(
        Label::new(
            RichText::new(
                "You can find multiple buttons above the code section.
        1 - Settings : Change the differents settings of the application by clicking on it
        2 - Save : Save your current code ! (Not implemented)
        3 - Load : Load a default machine or a custom one using a .tm file
        4 - Help : Access this guide by clicking on it
        5 - Graph : Convert the current code to a graph, to visualize and edit
        6 - Close : Close the code section to give more space to the graph",
            )
            .font(Font::default_medium()),
        )
        .halign(Align::Center)
        .wrap(),
    );

    Frame::new()
        .corner_radius(5)
        .stroke(Stroke::new(1.0, app.theme.gray))
        .show(ui, |ui| {
            let img = Image::new(include_image!("../../../assets/help/settings.png"))
                .maintain_aspect_ratio(true)
                .corner_radius(5);
            let size = img.load_and_calc_size(ui, ui.available_size());
            if let Some(size) = size {
                ui.set_width(size.x);
            }
            ui.add(img);
        });
}
