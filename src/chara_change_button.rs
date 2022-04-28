use crate::api;
use crate::api::Api;
use crate::history::Command;
use crate::project::VoiceVoxProject;
use eframe::egui;
use eframe::egui::Ui;
use egui::Context;
use std::collections::{BTreeMap, HashMap};

pub(crate) static ICON_AND_PORTRAIT_STORE: once_cell::race::OnceBox<
    HashMap<(String, String), egui_extras::RetainedImage>,
> = once_cell::race::OnceBox::new();

/// used for construct chara changing menu.
static STYLE_STRUCTURE: once_cell::race::OnceBox<Vec<(String, Vec<(String, i32)>)>> =
    once_cell::race::OnceBox::new();

/// used for generic usage.
pub static STYLE_ID_AND_CHARA_TABLE: once_cell::race::OnceBox<BTreeMap<i32, (String, String)>> =
    once_cell::race::OnceBox::new();

pub(crate) async fn init_icon_store() -> Option<()> {
    let mut style_structure = Vec::new();
    let mut style_and_chara_table = BTreeMap::new();

    let icons = {
        let mut map = HashMap::new();

        let mut speakers = api::Speakers { core_version: None }.call().await.ok()?;
        speakers.sort_by(|a, b| a.name.cmp(&b.name));
        // fetch style and gfx.
        for speaker in speakers {
            let speaker_uuid = speaker.speaker_uuid;
            let speaker_info = api::SpeakerInfo {
                speaker_uuid,
                core_version: None,
            }
            .call()
            .await
            .ok()?;

            let name = speaker.name;
            map.insert(
                (name.clone(), "portrait".to_owned()),
                egui_extras::RetainedImage::from_image_bytes(
                    format!("{}({})", &name, "portrait"),
                    &speaker_info.portrait,
                )
                .ok()?,
            );
            let mut style_infos = speaker_info.style_infos;
            style_infos.sort_by(|a, b| a.id.cmp(&b.id));
            let mut style_names = Vec::new();
            for (style, info) in speaker.styles.iter().zip(style_infos.iter()) {
                let sty_name = style.name.clone();
                let value = egui_extras::RetainedImage::from_image_bytes(
                    format!("{}({})", &name, &sty_name),
                    &info.icon,
                )
                .ok()?;
                style_names.push((sty_name.clone(), style.id));
                log::debug!("add icon for {}({})", name, sty_name);
                let key = (name.clone(), sty_name);
                style_and_chara_table.insert(style.id, key.clone());
                map.insert(key, value);
            }
            style_structure.push((name, style_names));
        }
        map
    };
    STYLE_ID_AND_CHARA_TABLE
        .set(Box::new(style_and_chara_table))
        .ok();
    STYLE_STRUCTURE.set(Box::new(style_structure)).ok();
    ICON_AND_PORTRAIT_STORE.set(Box::new(icons)).ok()
}

#[tokio::test]
async fn test_init_icon_store() {
    api::init();
    init_icon_store().await;
    ICON_AND_PORTRAIT_STORE.get().unwrap();
}

pub struct CharaChangeButton(pub i32);

impl CharaChangeButton {
    pub fn ui(self, line: &String, ui: &mut Ui, ctx: &Context) -> Option<CharaChangeCommand> {
        let mut rt = None;
        let image = ICON_AND_PORTRAIT_STORE.get()?;
        let style_structure = STYLE_STRUCTURE.get()?;
        let style_id_mapping = STYLE_ID_AND_CHARA_TABLE.get()?;
        let current_character = style_id_mapping.get(&self.0)?;
        let image_ref = image.get(current_character)?;
        ui.menu_image_button(
            &current_character.0,
            image_ref.texture_id(ctx),
            egui::vec2(32.0, 32.0),
            |ui| {
                for (character, styles) in style_structure {
                    if let Some((default_style, speaker)) = styles.get(0) {
                        if let Some(default_icon) =
                            image.get(&(character.clone(), default_style.clone()))
                        {
                            let default_style_button = egui::Button::image_and_text(
                                default_icon.texture_id(ctx),
                                egui::epaint::vec2(32.0, 32.0),
                                character,
                            );

                            if styles.len() > 1 {
                                if ui
                                    .menu_button_with_image(
                                        character,
                                        default_icon.texture_id(ctx),
                                        egui::vec2(32.0, 32.0),
                                        |ui| {
                                            for (style, speaker) in styles {
                                                if let Some(style_icon) =
                                                    image.get(&(character.clone(), style.clone()))
                                                {
                                                    let style_button = egui::Button::image_and_text(
                                                        style_icon.texture_id(ctx),
                                                        egui::epaint::vec2(32.0, 32.0),
                                                        format!("{}({})", character, style),
                                                    );
                                                    if ui.add(style_button).clicked() {
                                                        rt = Some(CharaChangeCommand {
                                                            line: line.clone(),
                                                            prev_chara: self.0,
                                                            new_chara: *speaker,
                                                        });
                                                    }
                                                }
                                            }
                                        },
                                    )
                                    .response
                                    .clicked()
                                {
                                    rt = Some(CharaChangeCommand {
                                        line: line.clone(),
                                        prev_chara: self.0,
                                        new_chara: *speaker,
                                    });
                                }
                            } else {
                                if ui.add(default_style_button).clicked() {
                                    rt = Some(CharaChangeCommand {
                                        line: line.clone(),
                                        prev_chara: self.0,
                                        new_chara: *speaker,
                                    });
                                }
                            }
                        }
                    }
                }
            },
        );
        rt.and_then(|rt| {
            if rt.new_chara == rt.prev_chara {
                None
            } else {
                Some(rt)
            }
        })
    }
}

pub struct CharaChangeCommand {
    line: String,
    pub prev_chara: i32,
    pub new_chara: i32,
}

impl Command for CharaChangeCommand {
    fn invoke(&mut self, project: &mut VoiceVoxProject) {
        if let Some(audio_item) = project.audioItems.get_mut(&self.line) {
            audio_item.styleId = self.new_chara;
        }
    }

    fn undo(&mut self, project: &mut VoiceVoxProject) {
        if let Some(audio_item) = project.audioItems.get_mut(&self.line) {
            audio_item.styleId = self.prev_chara;
        }
    }
    fn op_name(&self) -> &str {
        "キャラクター変更"
    }
}
