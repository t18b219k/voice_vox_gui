use eframe::egui::Ui;
enum CtxMenuOp {
    Cut,
    Copy,
    Paste,
    SelectAll,
}
fn create_context_menu(ui: &mut Ui) -> Option<CtxMenuOp> {
    let mut op = None;
    ui.vertical(|ui| {
        if ui.button("切り取り").clicked() {
            op = Some(CtxMenuOp::Cut);
        }

        if ui.button("コピー").clicked() {
            op = Some(CtxMenuOp::Copy);
        }

        if ui.button("貼り付け").clicked() {
            op = Some(CtxMenuOp::Paste);
        }
        ui.separator();
        if ui.button("全選択").clicked() {
            op = Some(CtxMenuOp::SelectAll);
        }
    });
    op
}
