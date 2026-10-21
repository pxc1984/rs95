//! The rs95 component showcase, shared by `examples/demo.rs` and the snapshot test.

use egui::{Align, Layout, RectAlign, Ui, vec2};
use rs95::{
    AppBar, Button, Checkbox, Counter, Frame, GroupBox, MenuButton, MenuItem, NumberInput,
    ProgressBar, Select, Separator, ScrollView, Slider, StatusField, Table, Tabs, TextInput,
    Theme, Toolbar, Window, menu_separator, radio_value, tooltip,
};

pub struct Showcase {
    theme: usize,
    applied_theme: Option<usize>,
    pub progress: f32,
    checked: bool,
    unchecked: bool,
    radio: u8,
    name: String,
    notes: String,
    count: i32,
    volume: f32,
    fruit: u8,
    tab: usize,
    toggle: bool,
    selected_row: Option<usize>,
    files: Vec<Vec<String>>,
    sort_col: usize,
    notepad_open: bool,
    controls_open: bool,
    mines: i64,
    word_wrap: bool,
}

impl Default for Showcase {
    fn default() -> Self {
        Self {
            theme: 0,
            applied_theme: None,
            progress: 0.62,
            checked: true,
            unchecked: false,
            radio: 1,
            name: "Bill".into(),
            notes: "Welcome to rs95!\n\nAn authentic Windows 95 look for egui.".into(),
            count: 42,
            volume: 70.0,
            fruit: 1,
            tab: 0,
            toggle: true,
            selected_row: Some(1),
            files: [
                ("autoexec.bat", "1 KB", "MS-DOS Batch"),
                ("command.com", "92 KB", "MS-DOS App"),
                ("config.sys", "1 KB", "System file"),
                ("io.sys", "218 KB", "System file"),
                ("msdos.sys", "2 KB", "System file"),
                ("readme.txt", "12 KB", "Text Document"),
            ]
            .into_iter()
            .map(|(a, b, c)| vec![a.into(), b.into(), c.into()])
            .collect(),
            sort_col: 0,
            notepad_open: true,
            controls_open: true,
            mines: 10,
            word_wrap: true,
        }
    }
}

impl Showcase {
    pub fn ui(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        if self.applied_theme != Some(self.theme) {
            rs95::install(&ctx, Theme::all()[self.theme]);
            self.applied_theme = Some(self.theme);
        }
        let t = Theme::get(&ctx);

        // taskbar
        egui::Panel::bottom("taskbar")
            .frame(egui::Frame::NONE.fill(t.material))
            .show_separator_line(false)
            .show(ui, |ui| self.taskbar(ui));

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(t.desktop_background))
            .show(ui, |ui| {
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui.add_space(12.0);
                    for (label, open) in [("Controls", &mut self.controls_open), ("Notepad", &mut self.notepad_open)] {
                        let r = ui.add(Button::new(label).thin().active(*open).min_size(vec2(80.0, 0.0)));
                        if r.clicked() {
                            *open = !*open;
                        }
                    }
                });
            });

        self.controls_window(&ctx);
        self.tabs_window(&ctx);
        self.explorer_window(&ctx);
        self.notepad_window(&ctx);
        self.mines_window(&ctx);
    }

    fn taskbar(&mut self, ui: &mut Ui) {
        AppBar::show(ui, |ui| {
            let (_, picked) = MenuButton::from_button(Button::new("Start"))
                .align(RectAlign::TOP_START)
                .show(ui, |ui| {
                    let mut picked = None;
                    for (i, name) in ["Programs", "Documents", "Settings"].into_iter().enumerate() {
                        if ui.add(MenuItem::new(name).submenu(true)).clicked() {
                            picked = Some(i);
                        }
                    }
                    ui.add(MenuItem::new("Find").submenu(true));
                    ui.add(MenuItem::new("Help"));
                    ui.add(MenuItem::new("Run..."));
                    menu_separator(ui);
                    ui.add(MenuItem::new("Shut Down..."));
                    picked
                });
            let _ = picked;
            ui.add(Separator::vertical());
            ui.add(Button::new("Notepad").active(self.notepad_open).min_size(vec2(120.0, 0.0)));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add(StatusField::new("12:00 PM").width(80.0));
            });
        });
    }

    fn controls_window(&mut self, ctx: &egui::Context) {
        Window::new("Controls")
            .open(&mut self.controls_open)
            .default_pos([110.0, 20.0])
            .help_button(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    ui.add(
                        Select::new("theme", &mut self.theme)
                            .options(Theme::all().iter().enumerate().map(|(i, t)| (i, t.name)))
                            .width(140.0),
                    );
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add(Button::new("OK").primary(true).min_size(vec2(75.0, 0.0)));
                    ui.add(Button::new("Cancel").min_size(vec2(75.0, 0.0)));
                    ui.add_enabled(false, Button::new("Disabled"));
                    if ui.add(Button::new("Toggle").active(self.toggle)).clicked() {
                        self.toggle = !self.toggle;
                    }
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.add(Checkbox::new(&mut self.checked, "Show hidden files"));
                        ui.add(Checkbox::new(&mut self.unchecked, "Hide extensions"));
                        ui.add_enabled(false, Checkbox::new(&mut true.clone(), "Disabled"));
                    });
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        radio_value(ui, &mut self.radio, 0, "Small icons");
                        radio_value(ui, &mut self.radio, 1, "Large icons");
                        radio_value(ui, &mut self.radio, 2, "Details");
                    });
                });
                ui.add_space(4.0);
                egui::Grid::new("form").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                    ui.label("Name:");
                    tooltip(ui.add(TextInput::singleline(&mut self.name).width(180.0)), "Your name");
                    ui.end_row();
                    ui.label("Copies:");
                    ui.add(NumberInput::new(&mut self.count).range(0..=99).width(70.0));
                    ui.end_row();
                    ui.label("Fruit:");
                    ui.add(
                        Select::new("fruit", &mut self.fruit)
                            .option(0, "Apple")
                            .option(1, "Banana")
                            .option(2, "Cherry")
                            .width(180.0),
                    );
                    ui.end_row();
                    ui.label("Volume:");
                    ui.add(Slider::new(&mut self.volume, 0.0..=100.0).step(10.0).marks(11).width(180.0));
                    ui.end_row();
                });
                ui.add_space(6.0);
                ui.add(ProgressBar::new(self.progress).width(300.0));
                ui.add_space(4.0);
                ui.add(ProgressBar::new(self.progress).tile().width(300.0));
            });
    }

    fn tabs_window(&mut self, ctx: &egui::Context) {
        Window::new("Display Properties")
            .default_pos([480.0, 20.0])
            .help_button(true)
            .show(ctx, |ui| {
                Tabs::new(&mut self.tab)
                    .tab("Background")
                    .tab("Screen Saver")
                    .tab("Appearance")
                    .show(ui, |ui, tab| {
                        ui.set_min_size(vec2(300.0, 150.0));
                        match tab {
                            0 => {
                                GroupBox::new("Wallpaper").show(ui, |ui| {
                                    ui.set_min_width(260.0);
                                    radio_value(ui, &mut self.radio, 0, "Tile");
                                    radio_value(ui, &mut self.radio, 1, "Center");
                                });
                            }
                            1 => {
                                GroupBox::new("Energy saving").show(ui, |ui| {
                                    ui.set_min_width(260.0);
                                    ui.label("Wait:");
                                    ui.add(NumberInput::new(&mut self.count).range(1..=60));
                                });
                            }
                            _ => {
                                ui.label("Scheme:");
                                ui.add(
                                    Select::new("scheme", &mut self.theme)
                                        .options(Theme::all().iter().enumerate().map(|(i, t)| (i, t.name)))
                                        .width(200.0),
                                );
                            }
                        }
                    });
                ui.add_space(8.0);
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    ui.add(Button::new("Apply").min_size(vec2(75.0, 0.0)));
                    ui.add(Button::new("Cancel").min_size(vec2(75.0, 0.0)));
                    ui.add(Button::new("OK").primary(true).min_size(vec2(75.0, 0.0)));
                });
            });
    }

    fn explorer_window(&mut self, ctx: &egui::Context) {
        Window::new("Exploring - C:\\")
            .default_pos([480.0, 330.0])
            .minimize_maximize(true)
            .default_size([380.0, 230.0])
            .resizable(true)
            .padding(4)
            .show(ctx, |ui| {
                Toolbar::show(ui, |ui| {
                    MenuButton::new("File").show(ui, |ui| {
                        ui.add(MenuItem::new("Open").shortcut("Ctrl+O"));
                        ui.add(MenuItem::new("Save").shortcut("Ctrl+S"));
                        menu_separator(ui);
                        ui.add_enabled(false, MenuItem::new("Print"));
                        ui.add(MenuItem::new("Exit"));
                    });
                    MenuButton::new("View").show(ui, |ui| {
                        if ui.add(MenuItem::new("Word wrap").checked(self.word_wrap)).clicked() {
                            self.word_wrap = !self.word_wrap;
                        }
                    });
                    MenuButton::new("Help").show(ui, |ui| {
                        ui.add(MenuItem::new("About rs95"));
                    });
                });
                ui.add_space(2.0);
                let headers = ["Name", "Size", "Type"];
                let r = Table::new(&headers)
                    .widths(&[150.0, 70.0])
                    .selected(&mut self.selected_row)
                    .show(ui, &self.files);
                if let Some(c) = r.header_clicked {
                    self.sort_col = c;
                    self.files.sort_by(|a, b| a[c].cmp(&b[c]));
                }
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add(StatusField::new(format!("{} object(s)", self.files.len())).width(150.0));
                    ui.add(StatusField::new("1.44 MB free").width(150.0));
                });
            });
    }

    fn notepad_window(&mut self, ctx: &egui::Context) {
        Window::new("Notepad")
            .open(&mut self.notepad_open)
            .default_pos([110.0, 470.0])
            .minimize_maximize(true)
            .active(false)
            .padding(2)
            .show(ctx, |ui| {
                ScrollView::new("notepad").max_height(120.0).show(ui, |ui| {
                    ui.set_width(300.0);
                    for i in 1..=15 {
                        ui.label(format!("Line {i}: The quick brown fox jumps over the lazy dog."));
                    }
                });
                ui.add_space(4.0);
                ui.add(TextInput::multiline(&mut self.notes).rows(3).width(330.0));
            });
    }

    fn mines_window(&mut self, ctx: &egui::Context) {
        Window::new("Minesweeper")
            .default_pos([890.0, 20.0])
            .minimize_maximize(true)
            .close_button(true)
            .show(ctx, |ui| {
                Frame::field().fill(Theme::get(ui.ctx()).material).inner_margin(4).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(Counter::new(self.mines));
                        ui.add_space(20.0);
                        if ui.add(Button::new(":)").square(true).min_size(vec2(26.0, 26.0))).clicked() {
                            self.mines = 10;
                        }
                        ui.add_space(20.0);
                        ui.add(Counter::new(127));
                    });
                });
                ui.add_space(4.0);
                Frame::field().fill(Theme::get(ui.ctx()).material).inner_margin(0).show(ui, |ui| {
                    egui::Grid::new("mines").spacing([0.0, 0.0]).show(ui, |ui| {
                        for y in 0..8 {
                            for x in 0..8 {
                                let open = (x + y) % 5 == 0 || y > 5;
                                let r = ui.add(Button::new(if open && x == 3 { "1" } else { " " })
                                    .active(open)
                                    .min_size(vec2(18.0, 18.0)));
                                if r.clicked() {
                                    self.mines -= 1;
                                }
                            }
                            ui.end_row();
                        }
                    });
                });
            });
    }
}
