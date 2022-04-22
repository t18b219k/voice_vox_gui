use eframe::egui::{Color32, Context, FontFamily, Layout, Stroke};
use eframe::epi::Frame;
use eframe::{CreationContext, NativeOptions};

mod api;
mod api_schema;
mod chara_change_button;
mod context_menu;
mod dialogue;
mod left_pane;
mod menu;
mod project;
mod tool_bar;

enum DialogueKind {
    ExitCustomize,
    RestoreDefault,
}

struct VoiceVoxRust {
    current_project: VoiceVoxProject,
    opening_file: Option<String>,
    tool_bar_config: Vec<ToolBarOp>,
    current_view: CurrentView,
    tool_bar_config_editing: Vec<ToolBarOp>,
    cursoring: usize,
    block_menu_control: bool,
    opening_dialogues: Option<DialogueKind>,
    current_selected_tts_line: usize,
    tts_lines: Vec<TTS>,
}

enum CurrentView {
    Main,
    ToolBarCustomize,
}

impl VoiceVoxRust {
    async fn new() -> Self {
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
            opening_dialogues: None,
            current_selected_tts_line: 0,
            tts_lines: vec![TTS {
                character_and_style: ("四国めたん".to_string(), "ノーマル".to_string()),
                speaker_in_audio_query: 2,
                text: "".to_string(),
            }],
        }
    }
    fn setup(&mut self, cc: &CreationContext) {
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

        cc.egui_ctx.set_fonts(fonts);
    }
}
use crate::api::Api;
use crate::dialogue::ExitControl;
use crate::menu::TopMenuOp;
use crate::project::VoiceVoxProject;
use crate::tool_bar::ToolBarOp;
use eframe::egui;

struct TTS {
    character_and_style: (String, String),
    speaker_in_audio_query: i64,
    text: String,
}

impl eframe::App for VoiceVoxRust {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let mut should_delete = None;
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
                egui::containers::TopBottomPanel::bottom("voice_control").show(ctx, |_ui| {});
                egui::containers::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical(|ui| {
                        crate::tool_bar::tool_bar(ui, &self.tool_bar_config, 28.0, false);
                        egui::containers::SidePanel::left("chara_view").show_inside(ui, |ui| {
                            if let Some(portrait_line) =
                                self.tts_lines.get(self.current_selected_tts_line)
                            {
                                let left_pane = crate::left_pane::LeftPane {
                                    current_character_and_style: (
                                        portrait_line.character_and_style.0.as_str(),
                                        portrait_line.character_and_style.1.as_str(),
                                    ),
                                };
                                ui.add(left_pane);
                            }
                        });
                        egui::containers::SidePanel::right("parameter_control")
                            .show_inside(ui, |_ui| {});
                        egui::containers::CentralPanel::default().show_inside(ui, |ui| {
                            let bottom_right = ui.max_rect().max;
                            let available_with = ui.available_width() - 64.0;

                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.set_min_width(available_with);
                                let len = self.tts_lines.len();
                                for (line, tts_line) in self.tts_lines.iter_mut().enumerate() {
                                    ui.horizontal(|ui| {
                                        let mut ccb = chara_change_button::CharaChangeButton::new(
                                            &tts_line.character_and_style.0,
                                            &tts_line.character_and_style.1,
                                        );
                                        let chara_change_notify = ccb.ui(ui, ctx);
                                        //フォーカスを失ったら合成リクエストを送る.
                                        if ui.text_edit_singleline(&mut tts_line.text).lost_focus()
                                            && !tts_line.text.is_empty()
                                        {
                                            let text = tts_line.text.clone();
                                            let speaker = tts_line.speaker_in_audio_query;
                                            let _join_handle = tokio::spawn(async move {
                                                api::AudioQuery {
                                                    text,
                                                    speaker,
                                                    core_version: None,
                                                }
                                                .call()
                                                .await
                                            });
                                        }
                                        if len > 1 {
                                            if ui.button("X").clicked() {
                                                should_delete = Some(line);
                                            }
                                        }
                                        if let Some(ccn) = chara_change_notify {
                                            log::debug!(
                                                "set character and style {} {}",
                                                ccn.0,
                                                ccn.1
                                            );
                                            tts_line.speaker_in_audio_query = ccn.2;
                                            tts_line.character_and_style =
                                                (ccn.0.to_owned(), ccn.1.to_owned());
                                        }
                                    });
                                }
                            });

                            let top_left = bottom_right - egui::vec2(64.0, 64.0);
                            let center = bottom_right - egui::vec2(32.0, 32.0);
                            let response = ui.allocate_rect(
                                egui::Rect::from_min_max(top_left, bottom_right),
                                egui::Sense::click(),
                            );
                            let rect = response.rect;
                            let painter = ui.painter_at(rect);
                            painter.circle_filled(center, 32.0, Color32::LIGHT_GREEN);
                            painter.hline(
                                center.x - 8.0..=center.x + 8.0,
                                center.y,
                                Stroke::new(4.0, Color32::BLACK),
                            );
                            painter.vline(
                                center.x,
                                center.y - 8.0..=center.y + 8.0,
                                Stroke::new(4.0, Color32::BLACK),
                            );

                            if response.clicked() {
                                self.tts_lines.push(TTS {
                                    character_and_style: (
                                        "四国めたん".to_string(),
                                        "ノーマル".to_string(),
                                    ),
                                    speaker_in_audio_query: 2,
                                    text: "".to_string(),
                                })
                            }
                        });
                    });
                });
                if let Some(delete_line) = should_delete {
                    self.tts_lines.remove(delete_line);
                }
            }
            CurrentView::ToolBarCustomize => {
                egui::containers::CentralPanel::default().show(ctx, |ui| {
                    ui.add_enabled_ui(self.opening_dialogues.is_none(), |ui| {
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
                                                self.opening_dialogues =
                                                    Some(DialogueKind::ExitCustomize);
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
                                            self.opening_dialogues =
                                                Some(DialogueKind::RestoreDefault);
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
                match self.opening_dialogues {
                    None => {}
                    Some(DialogueKind::ExitCustomize) => {
                        let mut cell: Option<bool> = None;
                        let dialogue = dialogue::Dialogue {
                            title: "カスタマイズを放棄しますか",
                            text: "このまま終了すると,カスタマイズは放棄されてリセットされます.",
                            control_constructor: Box::new(ExitControl {}),
                            cell: Some(&mut cell),
                        };
                        dialogue.show(ctx);
                        match cell {
                            Some(true) => {
                                self.opening_dialogues = None;
                                self.block_menu_control = false;
                                self.current_view = CurrentView::Main;
                            }
                            Some(false) => {
                                self.opening_dialogues = None;
                            }
                            _ => {}
                        }
                    }
                    Some(DialogueKind::RestoreDefault) => {
                        let mut cell: Option<bool> = None;
                        let dialogue = dialogue::Dialogue {
                            title: "ツールバーをデフォルトに戻します",
                            text: "ツールバーをデフォルトに戻します.よろしいですか.",
                            control_constructor: Box::new(crate::dialogue::AcceptControl {}),
                            cell: Some(&mut cell),
                        };
                        dialogue.show(ctx);
                        match cell {
                            Some(true) => {
                                self.opening_dialogues = None;
                                self.block_menu_control = false;
                                self.tool_bar_config_editing = vec![
                                    ToolBarOp::PlayAll,
                                    ToolBarOp::Stop,
                                    ToolBarOp::ExportSelected,
                                    ToolBarOp::Blank,
                                    ToolBarOp::Undo,
                                    ToolBarOp::Redo,
                                ];
                            }
                            Some(false) => {
                                self.opening_dialogues = None;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
#[tokio::main]
async fn main() {
    simple_log::console("debug").unwrap();
    api::init();
    chara_change_button::init_icon_store().await;
    let mut app = VoiceVoxRust::new().await;

    eframe::run_native(
        "voice_vox_gui",
        NativeOptions::default(),
        Box::new(|cc| {
            app.setup(cc);
            Box::new(app)
        }),
    );
}
