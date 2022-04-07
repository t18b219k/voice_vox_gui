use eframe::egui;
use eframe::egui::{TextStyle, Ui, Widget};
use std::collections::HashMap;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum ToolBarOp {
    PlayAll,
    Stop,
    ExportSelected,
    ExportAll,
    ExportAllInOneFile,
    SaveProject,
    Undo,
    Redo,
    LoadText,
    Blank,
}

pub const TOOL_BAR_OPS: once_cell::sync::Lazy<HashMap<ToolBarOp, String>> =
    once_cell::sync::Lazy::new(|| {
        [
            ("連続再生", ToolBarOp::PlayAll),
            ("停止", ToolBarOp::Stop),
            ("一つ書き出し", ToolBarOp::ExportSelected),
            ("全部書き出し", ToolBarOp::ExportAll),
            ("音声をつなげて書き出し", ToolBarOp::ExportAllInOneFile),
            ("プロジェクト保存", ToolBarOp::SaveProject),
            ("元に戻す", ToolBarOp::Undo),
            ("やり直す", ToolBarOp::Redo),
            ("テキスト読み込み", ToolBarOp::LoadText),
            ("空白", ToolBarOp::Blank),
        ]
        .iter()
        .map(|(txt, op)| (*op, txt.to_string()))
        .collect()
    });

pub fn tool_bar(
    ui: &mut Ui,
    tool_bar_config: &[ToolBarOp],
    unit: f32,
    is_customizing: bool,
) -> Option<ToolBarOp> {
    ui.horizontal(|ui| {
        let mut o = None;
        let mut split = tool_bar_config.split(|op| *op == ToolBarOp::Blank);

        if let Some(ops) = split.next() {
            for op in ops {
                match op {
                    ToolBarOp::Blank => {}
                    x => {
                        if ui
                            .button(egui::RichText::new(&TOOL_BAR_OPS[x]).size(unit).monospace())
                            .clicked()
                        {
                            o = Some(*x)
                        }
                    }
                }
            }
        }

        if let Some(ops) = split.next() {
            ui.with_layout(egui::Layout::right_to_left(), |ui| {
                for op in ops.iter().rev() {
                    match op {
                        ToolBarOp::Blank => {}
                        x => {
                            if ui
                                .button(egui::RichText::new(&TOOL_BAR_OPS[x]).size(unit))
                                .clicked()
                            {
                                o = Some(*x)
                            }
                        }
                    }
                }
                if is_customizing {
                    let sz = ui.available_size();
                    if ui.add_sized(sz, egui::Button::new("")).clicked() {
                        o = Some(ToolBarOp::Blank)
                    }
                }
            });
        }

        o
    })
    .inner
}
