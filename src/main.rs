use eframe::egui::{Align2, Color32, Context, FontFamily, Layout, Vec2};
use eframe::epi::{App, Frame, Storage};
use eframe::NativeOptions;

mod api;
mod api_schema;
mod context_menu;
mod menu;
mod project;
mod tool_bar;

trace::init_depth_var!();

struct VoiceVoxRust {
    current_project: VoiceVoxProject,
    opening_file: Option<String>,
    tool_bar_config: Vec<ToolBarOp>,
    current_view: CurrentView,
    tool_bar_config_editing: Vec<ToolBarOp>,
    cursoring: usize,
    block_menu_control: bool,
    dialog_opening: bool,
}

enum CurrentView {
    Main,
    ToolBarCustomize,
}

impl VoiceVoxRust {
    fn new() -> Self {
        Self {
            current_project: VoiceVoxProject {},
            opening_file: None,
            tool_bar_config: vec![
                ToolBarOp::PlayAll,
                ToolBarOp::Stop,
                ToolBarOp::ExportSelected,
                ToolBarOp::Blank,
                ToolBarOp::Undo,
                ToolBarOp::Redo,
            ],
            current_view: CurrentView::Main,
            tool_bar_config_editing: vec![],
            cursoring: 0,
            block_menu_control: false,
            dialog_opening: false,
        }
    }
}
use crate::menu::TopMenuOp;
use crate::project::VoiceVoxProject;
use crate::tool_bar::ToolBarOp;
use eframe::egui;

impl App for VoiceVoxRust {
    fn setup(&mut self, ctx: &Context, _frame: &Frame, _storage: Option<&dyn Storage>) {
        let mut fonts = egui::FontDefinitions::default();
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "Noto".to_owned());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "Noto".to_owned());

        fonts.font_data.insert(
            "Noto".to_owned(),
            egui::FontData::from_static(include_bytes!("../resources/NotoSansJP-Regular.otf")),
        );
        ctx.set_fonts(fonts);
    }
    fn update(&mut self, ctx: &Context, frame: &Frame) {
        frame.set_window_title(&format!(
            "{} VoiceVox",
            self.opening_file.as_ref().unwrap_or(&"*".to_owned())
        ));

        let menu_bar_op = egui::containers::TopBottomPanel::top("TopMenu")
            .show(ctx, |ui| {
                ui.add_enabled_ui(!self.block_menu_control, crate::menu::create_menu_bar)
                    .inner
            })
            .inner;

        if let Some(op) = menu_bar_op {
            match op {
                TopMenuOp::NewProject => {
                    self.current_project = VoiceVoxProject {};
                }
                TopMenuOp::AudioOutput => {}
                TopMenuOp::OutputOne => {}
                TopMenuOp::OutputConnected => {}
                TopMenuOp::LoadText => {}
                TopMenuOp::OverwriteProject => {}
                TopMenuOp::SaveProjectAs => {}
                TopMenuOp::LoadProject => {}
                TopMenuOp::RebootEngine => {}
                TopMenuOp::KeyConfig => {}
                TopMenuOp::ToolBarCustomize => {
                    self.tool_bar_config_editing = self.tool_bar_config.clone();
                    self.current_view = CurrentView::ToolBarCustomize;
                    self.cursoring = 0;
                    self.block_menu_control = true;
                }
                TopMenuOp::SampleVoice => {}
                TopMenuOp::DefaultStyle => {}
                TopMenuOp::Dictionary => {}
                TopMenuOp::Option => {}
                TopMenuOp::Help => {}
            }
        }
        match self.current_view {
            CurrentView::Main => {
                egui::containers::TopBottomPanel::bottom("voice_control").show(ctx, |ui| {});
                egui::containers::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical(|ui| {
                        crate::tool_bar::tool_bar(ui, &self.tool_bar_config, 28.0, false);
                        egui::containers::SidePanel::left("chara_view").show_inside(ui, |ui| {});
                        egui::containers::CentralPanel::default().show_inside(ui, |ui| {});
                        egui::containers::SidePanel::right("parameter_control")
                            .show_inside(ui, |ui| {});
                    });
                });
            }
            CurrentView::ToolBarCustomize => {
                egui::containers::CentralPanel::default().show(ctx, |ui| {
                    ui.add_enabled_ui(!self.dialog_opening, |ui| {
                        ui.vertical(|ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("ツールバーのカスタマイズ").size(28.0),
                                    );
                                    let restore_default = egui::Button::new(
                                        egui::RichText::new("デフォルトに戻す").size(28.0),
                                    );
                                    let is_default = vec![
                                        ToolBarOp::PlayAll,
                                        ToolBarOp::Stop,
                                        ToolBarOp::ExportSelected,
                                        ToolBarOp::Blank,
                                        ToolBarOp::Undo,
                                        ToolBarOp::Redo,
                                    ] != self.tool_bar_config_editing;
                                    let changed =
                                        self.tool_bar_config_editing != self.tool_bar_config;
                                    let save_config =
                                        egui::Button::new(egui::RichText::new("保存").size(28.0));
                                    let exit =
                                        egui::Button::new(egui::RichText::new("X").size(28.0))
                                            .fill(Color32::TRANSPARENT);
                                    ui.with_layout(Layout::right_to_left(), |ui| {
                                        if ui.add(exit).clicked() {
                                            if self.tool_bar_config != self.tool_bar_config_editing
                                            {
                                                self.dialog_opening = true;
                                            } else {
                                                self.block_menu_control = false;
                                                self.current_view = CurrentView::Main;
                                            }
                                        }

                                        if ui.add_enabled(changed, save_config).clicked() {
                                            self.tool_bar_config =
                                                self.tool_bar_config_editing.clone();
                                        }
                                        if ui.add_enabled(is_default, restore_default).clicked() {
                                            self.tool_bar_config_editing = vec![
                                                ToolBarOp::PlayAll,
                                                ToolBarOp::Stop,
                                                ToolBarOp::ExportSelected,
                                                ToolBarOp::Blank,
                                                ToolBarOp::Undo,
                                                ToolBarOp::Redo,
                                            ];
                                        }
                                    });
                                });

                                let op = crate::tool_bar::tool_bar(
                                    ui,
                                    &self.tool_bar_config_editing,
                                    28.0,
                                    true,
                                );

                                if let Some(op) = op {
                                    self.cursoring = self
                                        .tool_bar_config_editing
                                        .iter()
                                        .enumerate()
                                        .find(|(_, x)| **x == op)
                                        .map(|x| x.0)
                                        .unwrap()
                                }
                                let index = if self.cursoring >= self.tool_bar_config_editing.len()
                                {
                                    0
                                } else {
                                    self.cursoring
                                };

                                let text = self
                                    .tool_bar_config_editing
                                    .get(index)
                                    .map(|op| &crate::tool_bar::TOOL_BAR_OPS[&op]);
                                ui.horizontal(|ui| {
                                    if let Some(text) = text {
                                        ui.label(
                                            egui::RichText::new(format!("「{}」を選択中", text))
                                                .size(28.0)
                                                .monospace(),
                                        );
                                    }
                                    let move_left = egui::Button::new(
                                        egui::RichText::new("左に動かす").size(28.0),
                                    );
                                    let move_right = egui::Button::new(
                                        egui::RichText::new("右に動かす").size(28.0),
                                    );

                                    ui.with_layout(Layout::right_to_left(), |ui| {
                                        if ui
                                            .button(egui::RichText::new("削除する").size(28.0))
                                            .clicked()
                                        {
                                            self.tool_bar_config_editing.remove(index);
                                        };
                                        if ui
                                            .add_enabled(
                                                index + 1 != self.tool_bar_config_editing.len(),
                                                move_right,
                                            )
                                            .clicked()
                                        {
                                            self.cursoring += 1;
                                            self.tool_bar_config_editing.swap(index, index + 1);
                                        }
                                        if ui.add_enabled(index != 0, move_left).clicked() {
                                            self.cursoring -= 1;
                                            self.tool_bar_config_editing.swap(index, index - 1);
                                        }
                                    });
                                });
                            });
                        });
                    });
                });
                if self.dialog_opening {
                    egui::containers::Window::new("exit_customize")
                        .title_bar(false)
                        .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
                        .resizable(false)
                        .show(ctx, |ui| {

                            let h = ui
                                .label(
                                    egui::RichText::new("カスタマイズを放棄しますか")
                                        .size(28.0)
                                        .strong(),
                                )
                                .rect
                                .height();
                            ui.set_height(h * 3.0);
                            ui.add_space(h);
                            ui.label(
                                "このまま終了すると,カスタマイズは放棄されてリセットされます.",
                            );
                            ui.with_layout(Layout::right_to_left(), |ui| {
                                if ui.button(egui::RichText::new("終了").size(28.0)).clicked() {
                                    self.dialog_opening = false;
                                    self.block_menu_control = false;
                                    self.current_view = CurrentView::Main;
                                }
                                if ui
                                    .button(egui::RichText::new("キャンセル").size(28.0))
                                    .clicked()
                                {
                                    self.dialog_opening = false;
                                }
                            });
                        });
                }
            }
        }
    }

    fn name(&self) -> &str {
        "VoiceVox"
    }
}
fn main() {
    simple_log::console("debug").unwrap();

    eframe::run_native(
        Box::new(VoiceVoxRust::new()),
        NativeOptions {
            always_on_top: false,
            maximized: false,
            decorated: true,
            drag_and_drop_support: true,
            icon_data: None,
            initial_window_pos: None,
            initial_window_size: None,
            min_window_size: None,
            max_window_size: None,
            resizable: true,
            transparent: false,
        },
    );
}
