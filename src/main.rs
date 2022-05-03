use eframe::egui::{Color32, Context, FontFamily, Layout, Stroke};
use eframe::{CreationContext, Frame, NativeOptions};

use crate::api::{APIError, Api};

use crate::bottom_pane::Displaying;
use crate::commands::AudioQueryCommands;
use crate::dialogue::ExitControl;
use crate::history::Command;
use crate::menu::TopMenuOp;
use crate::project::VoiceVoxProject;
use crate::tool_bar::ToolBarOp;
use eframe::egui;
use std::collections::{BTreeMap, HashMap};
use std::io::{Cursor, Seek};
use tokio::sync::oneshot::error::TryRecvError;
use tokio::sync::oneshot::Receiver;
use tokio::time::Instant;

mod api;
mod api_schema;
mod bottom_pane;
mod chara_change_button;
mod commands;
mod context_menu;
mod dialogue;
mod history;
mod left_pane;
mod menu;
mod project;
mod right_pane;
mod tool_bar;

enum DialogueKind {
    ExitCustomize,
    RestoreDefault,
}

struct VoiceVoxRust {
    opening_file: Option<String>,
    tool_bar_config: Vec<ToolBarOp>,
    current_view: CurrentView,
    tool_bar_config_editing: Vec<ToolBarOp>,
    cursoring: usize,
    block_menu_control: bool,
    opening_dialogues: Option<DialogueKind>,
    current_selected_tts_line: String,
    back_up_text: String,
    histories: crate::history::HistoryManager,
    audio_query_jobs: HashMap<String, AudioQueryState>,
    current_displaying: crate::bottom_pane::Displaying,
    /// used to reduce Synthesis request.
    /// * key : (uuid,timestamp)
    /// * value : Cursor wrapped wav file.
    ///
    synthesis_cache: HashMap<(String, tokio::time::Instant), SynthesisState>,
}

static BLANK_AUDIO_QUERY: once_cell::race::OnceBox<api_schema::AudioQuery> =
    once_cell::race::OnceBox::new();

async fn init_blank_audio_query() {
    let blank_query = api::AudioQuery {
        text: "".to_string(),
        speaker: 2,
        core_version: None,
    }
    .call()
    .await
    .unwrap();
    BLANK_AUDIO_QUERY.set(Box::new(blank_query)).unwrap();
    log::debug!("initialized blank audio query.")
}

enum CurrentView {
    Main,
    ToolBarCustomize,
}

impl VoiceVoxRust {
    async fn new() -> Self {
        Self {
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
            current_selected_tts_line: String::new(),
            back_up_text: "".to_string(),
            histories: crate::history::HistoryManager::new().await,
            audio_query_jobs: Default::default(),
            current_displaying: Displaying::Accent,
            synthesis_cache: HashMap::new(),
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

enum AudioQueryState {
    WaitingForQuery(String, Receiver<<crate::api::AudioQuery as Api>::Response>),
    NoJob,
    Finished(String, api_schema::AudioQuery),
    Failed,
}

enum SynthesisState {
    WaitingForSynthesis(Receiver<<crate::api::Synthesis as Api>::Response>),
    Finished(Cursor<Vec<u8>>),
    Failed,
}

impl eframe::App for VoiceVoxRust {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
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
                TopMenuOp::NewProject => {}
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
                let mut invocations: Vec<(Box<dyn Command>, String)> = vec![];

                egui::containers::TopBottomPanel::bottom("voice_control").show(ctx, |ui| {
                    if let Some(ai) = self
                        .histories
                        .project
                        .audioItems
                        .get(&self.current_selected_tts_line)
                    {
                        let mut should_play = None;
                        if let Some(query) = &ai.query {
                            if let Some(cmd) = crate::bottom_pane::create_bottom_pane(
                                &mut self.current_displaying,
                                &mut should_play,
                                ui,
                                &query.accent_phrases,
                            ) {
                                invocations
                                    .push((Box::new(cmd), self.current_selected_tts_line.clone()));
                            }
                            if let Some(true) = should_play {
                                if let Some(instant) = self
                                    .histories
                                    .get_current_time_stamp(&self.current_selected_tts_line)
                                {
                                    let key = (self.current_selected_tts_line.clone(), instant);
                                    if let None = self.synthesis_cache.get(&key) {
                                        let (tx, rx) = tokio::sync::oneshot::channel();
                                        log::debug!(
                                            "send synthesis request for {} @ {:?}",
                                            self.current_selected_tts_line,
                                            instant
                                        );
                                        self.synthesis_cache
                                            .insert(key, SynthesisState::WaitingForSynthesis(rx));
                                        if let Some(ai) = self
                                            .histories
                                            .project
                                            .audioItems
                                            .get(&self.current_selected_tts_line)
                                        {
                                            let ai = ai.clone();
                                            tokio::spawn(async move {
                                                tx.send(
                                                    api::Synthesis {
                                                        speaker: ai.styleId,
                                                        enable_interrogative_upspeak: None,
                                                        core_version: None,
                                                        audio_query: ai.query.unwrap(),
                                                    }
                                                    .call()
                                                    .await,
                                                )
                                                .unwrap()
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
                egui::containers::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical(|ui| {
                        if let Some(toolbar_op) =
                            crate::tool_bar::tool_bar(ui, &self.tool_bar_config, 28.0, false)
                        {
                            match toolbar_op {
                                ToolBarOp::PlayAll => {}
                                ToolBarOp::Stop => {}
                                ToolBarOp::ExportSelected => {}
                                ToolBarOp::ExportAll => {}
                                ToolBarOp::ExportAllInOneFile => {}
                                ToolBarOp::SaveProject => {}
                                ToolBarOp::Undo => {
                                    self.histories.undo();
                                }
                                ToolBarOp::Redo => {
                                    self.histories.redo();
                                }
                                ToolBarOp::LoadText => {}
                                ToolBarOp::Blank => {}
                            }
                        }
                        egui::containers::SidePanel::left("chara_view").show_inside(ui, |ui| {
                            if let Some(portrait_line) = self
                                .histories
                                .project
                                .audioItems
                                .get(&self.current_selected_tts_line)
                            {
                                if let Some(chara) =
                                    crate::chara_change_button::STYLE_ID_AND_CHARA_TABLE
                                        .get()
                                        .unwrap()
                                        .get(&portrait_line.styleId)
                                {
                                    let left_pane = crate::left_pane::LeftPane {
                                        current_character_and_style: (
                                            chara.0.as_str(),
                                            chara.1.as_str(),
                                        ),
                                    };
                                    ui.add(left_pane);
                                }
                            }
                        });
                        egui::containers::SidePanel::right("parameter_control").show_inside(
                            ui,
                            |ui| {
                                if let Some(audio_item) = self
                                    .histories
                                    .project
                                    .audioItems
                                    .get(&self.current_selected_tts_line)
                                {
                                    if let Some(aq) = &audio_item.query {
                                        if let Some(x) =
                                            crate::right_pane::render_synthesis_control(aq, ui)
                                        {
                                            invocations.push((
                                                Box::new(x),
                                                self.current_selected_tts_line.clone(),
                                            ));
                                        }
                                    }
                                }
                            },
                        );
                        egui::containers::CentralPanel::default().show_inside(ui, |ui| {
                            let bottom_right = ui.max_rect().max;
                            let available_with = ui.available_width() - 64.0;

                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.set_min_width(available_with);
                                let len = self.histories.project.audioItems.len();
                                for line in self.histories.project.audioKeys.iter() {
                                    let tts_line =
                                        self.histories.project.audioItems.get_mut(line).unwrap();

                                    ui.horizontal(|ui| {
                                        let ccb = chara_change_button::CharaChangeButton(
                                            tts_line.styleId,
                                        );

                                        let chara_change_notify = ccb.ui(ui, ctx);

                                        let res = if self.current_selected_tts_line.eq(line) {
                                            ui.text_edit_singleline(&mut tts_line.text)
                                        } else {
                                            let mut dt = tts_line.text.clone();
                                            ui.text_edit_singleline(&mut dt)
                                        };
                                        //フォーカスを得たらラインバッファを履歴から取得
                                        if res.gained_focus() {
                                            self.back_up_text = tts_line.text.clone();
                                        }

                                        if res.has_focus() {
                                            self.current_selected_tts_line = line.clone();
                                        }
                                        //フォーカスを失ったら合成リクエストを送る.
                                        if res.lost_focus() && !tts_line.text.is_empty() {
                                            log::debug!("send audio query request for {}", line);
                                            let speaker = tts_line.styleId;
                                            let (tx, rx) = tokio::sync::oneshot::channel();
                                            let text = tts_line.text.clone();
                                            self.audio_query_jobs.insert(
                                                line.clone(),
                                                AudioQueryState::WaitingForQuery(text.clone(), rx),
                                            );
                                            tokio::spawn(async move {
                                                tx.send(
                                                    api::AudioQuery {
                                                        text: text.clone(),
                                                        speaker,
                                                        core_version: None,
                                                    }
                                                    .call()
                                                    .await,
                                                )
                                                .unwrap();
                                            });
                                        }
                                        if len > 1 {
                                            if ui.button("X").clicked() {
                                                invocations.push((
                                                    Box::new(AudioQueryCommands::Remove(0, None)),
                                                    line.clone(),
                                                ));
                                            }
                                        }
                                        if let Some(ccn) = chara_change_notify {
                                            let style_id = ccn.new_chara;
                                            invocations.push((Box::new(ccn), line.clone()));

                                            log::debug!(
                                                "send audio query request for {} with id {}.",
                                                line,
                                                style_id
                                            );
                                            let (tx, rx) = tokio::sync::oneshot::channel();
                                            let text = tts_line.text.clone();
                                            self.audio_query_jobs.insert(
                                                line.clone(),
                                                AudioQueryState::WaitingForQuery(text.clone(), rx),
                                            );
                                            tokio::spawn(async move {
                                                tx.send(
                                                    api::AudioQuery {
                                                        text: text.clone(),
                                                        speaker: style_id,
                                                        core_version: None,
                                                    }
                                                    .call()
                                                    .await,
                                                )
                                                .unwrap();
                                            });
                                        }
                                        if let Some(job) = self.audio_query_jobs.get_mut(line) {
                                            if let AudioQueryState::WaitingForQuery(
                                                text,
                                                ref mut ac,
                                            ) = job
                                            {
                                                if let Ok(aq) = ac.try_recv() {
                                                    match aq {
                                                        Ok(aq) => {
                                                            *job = AudioQueryState::Finished(
                                                                text.clone(),
                                                                aq,
                                                            );
                                                        }
                                                        Err(_) => {
                                                            *job = AudioQueryState::Failed;
                                                        }
                                                    }
                                                } else {
                                                    ui.spinner();
                                                }
                                            } else if let AudioQueryState::Finished(text, aq) = job
                                            {
                                                //inspect history.
                                                invocations.push((
                                                    Box::new(
                                                        AudioQueryCommands::UpdateAccentPhrases {
                                                            new_text: text.clone(),
                                                            accent_phrases: aq
                                                                .accent_phrases
                                                                .clone(),
                                                            prev_text: self.back_up_text.clone(),
                                                        },
                                                    ),
                                                    line.clone(),
                                                ));

                                                *job = AudioQueryState::NoJob;
                                            }
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
                                let uuid = uuid::Uuid::new_v4().to_string();
                                log::debug!("new uuid {}", uuid);
                                invocations.push((
                                    Box::new(AudioQueryCommands::Insert(
                                        crate::project::AudioItem {
                                            text: "".to_string(),
                                            styleId: 2,
                                            query: BLANK_AUDIO_QUERY.get().cloned(),
                                            presetKey: None,
                                        },
                                    )),
                                    uuid,
                                ));
                            }
                        });
                    });
                });

                for invocation in invocations {
                    self.histories.invoke(invocation.0, invocation.1);
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
        for synthesis_state in self.synthesis_cache.values_mut() {
            match synthesis_state {
                SynthesisState::WaitingForSynthesis(rx) => match rx.try_recv() {
                    Ok(v) => match v {
                        Ok(v) => *synthesis_state = SynthesisState::Finished(Cursor::new(v)),
                        Err(_) => *synthesis_state = SynthesisState::Failed,
                    },
                    Err(TryRecvError::Closed) => *synthesis_state = SynthesisState::Failed,
                    Err(TryRecvError::Empty) => {}
                },
                SynthesisState::Finished(_) => {}
                _ => {}
            }
        }
    }
}
#[tokio::main]
async fn main() {
    simple_log::console("debug").unwrap();
    init_blank_audio_query().await;
    chara_change_button::init_icon_store().await;
    let mut app = VoiceVoxRust::new().await;

    eframe::run_native(
        "voice_vox_gui",
        NativeOptions {
            initial_window_size: Some(egui::vec2(800.0, 600.0)),
            ..NativeOptions::default()
        },
        Box::new(|cc| {
            app.setup(cc);
            Box::new(app)
        }),
    );
}
