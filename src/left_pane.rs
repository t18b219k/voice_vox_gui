use crate::chara_change_button::ICON_AND_PORTRAIT_STORE;
use eframe::egui::{Response, Ui, Widget};

pub struct LeftPane<'a> {
    pub(crate) current_character_and_style: (&'a str, &'a str),
}

impl<'a> Widget for LeftPane<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.label(format!(
                "{}({})",
                self.current_character_and_style.0, self.current_character_and_style.1
            ));
            if let Some(store) = ICON_AND_PORTRAIT_STORE.get() {
                if let Some(i) = store.get(&(
                    self.current_character_and_style.0.to_owned(),
                    "portrait".to_owned(),
                )) {
                    let t_id = i.texture_id(ui.ctx());
                    ui.vertical_centered(|ui| {
                        ui.image(t_id, eframe::egui::vec2(128.0, 256.0));
                    });
                }
            }
        })
        .response
    }
}
