use eframe::egui::Ui;

pub enum TopMenuOp {
    NewProject,
    AudioOutput,
    OutputOne,
    OutputConnected,
    LoadText,
    OverwriteProject,
    SaveProjectAs,
    LoadProject,
    RebootEngine,
    KeyConfig,
    ToolBarCustomize,
    SampleVoice,
    DefaultStyle,
    Dictionary,
    Option,
    Help,
}

pub fn create_menu_bar(ui: &mut Ui) -> Option<TopMenuOp> {
    ui.horizontal(|ui| {
        let mut op = None;
        ui.menu_button("ファイル", |ui| {
            if ui.button("新規プロジェクト").clicked() {
                op = Some(TopMenuOp::NewProject);
            }
            if ui.button("音声書き出し").clicked() {
                op = Some(TopMenuOp::AudioOutput);
            }
            if ui.button("一つだけ書き出し").clicked() {
                op = Some(TopMenuOp::OutputOne);
            }
            if ui.button("音声をつなげて書き出し").clicked() {
                op = Some(TopMenuOp::OutputConnected);
            }
            if ui.button("テキスト読み込み").clicked() {
                op = Some(TopMenuOp::LoadText);
            }
            ui.separator();
            if ui.button("プロジェクトを上書き保存").clicked() {
                op = Some(TopMenuOp::OverwriteProject);
            }
            if ui.button("プロジェクトを名前を付けて保存").clicked() {
                op = Some(TopMenuOp::SaveProjectAs);
            }
            if ui.button("プロジェクト読み込み").clicked() {
                op = Some(TopMenuOp::LoadProject);
            }
        });
        ui.menu_button("エンジン", |ui| {
            if ui.button("再起動").clicked() {
                op = Some(TopMenuOp::RebootEngine);
            }
        });
        ui.menu_button("設定", |ui| {
            if ui.button("キー割り当て").clicked() {
                op = Some(TopMenuOp::KeyConfig)
            }
            if ui.button("ツールバーのカスタマイズ").clicked() {
                op = Some(TopMenuOp::ToolBarCustomize)
            }
            if ui.button("キャラクター並び替え・試聴").clicked() {
                op = Some(TopMenuOp::SampleVoice)
            }
            if ui.button("デフォルトスタイル").clicked() {
                op = Some(TopMenuOp::DefaultStyle)
            }
            if ui.button("読み方&アクセント辞書").clicked() {
                op = Some(TopMenuOp::Dictionary)
            }
            ui.separator();
            if ui.button("オプション").clicked() {
                op = Some(TopMenuOp::Option)
            }
        });

        if ui.button("ヘルプ").clicked() {
            op = Some(TopMenuOp::Help);
        }

        op
    })
    .inner
}
