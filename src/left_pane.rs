use crate::api;
use crate::api::Api;
use crate::api_schema;
use eframe::egui::Ui;
use std::collections::HashMap;

pub struct LeftPane {
    images: HashMap<String, egui_extras::RetainedImage>,
    current_character_and_style: (String, String),
}
impl LeftPane {
    pub fn render_left_pane(&self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label(format!(
                "{}({})",
                self.current_character_and_style.0, self.current_character_and_style.1
            ));
            if let Some(i) = self.images.get(&self.current_character_and_style.0) {
                let t_id = i.texture_id(ui.ctx());
                ui.vertical_centered(|ui| {
                    ui.image(t_id, i.size_vec2());
                });
            }
        });
    }
    pub async fn new() -> Self {
        let speakers = api::Speakers { core_version: None }.call();
        let mut images = HashMap::new();
        for speaker in speakers.await.unwrap().iter() {
            let api_schema::Speaker {
                name,
                speaker_uuid,
                styles,
                version,
            } = speaker;
            let uuid = speaker_uuid.clone();
            let si = api::SpeakerInfo {
                speaker_uuid: uuid,
                core_version: None,
            }
            .call()
            .await
            .unwrap();
            let api_schema::SpeakerInfo {
                policy: _,
                portrait,
                style_infos: _,
            } = si;
            images.insert(
                name.clone(),
                egui_extras::RetainedImage::from_image_bytes(name.clone(), &portrait).unwrap(),
            );
        }

        Self {
            images,
            current_character_and_style: ("".to_string(), "".to_string()),
        }
    }
    pub fn set_displaying_name(&mut self, name: &str) {
        self.current_character_and_style.0 = name.to_owned();
    }
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.images.keys()
    }
}
