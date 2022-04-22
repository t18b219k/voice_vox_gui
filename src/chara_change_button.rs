use crate::api;
use crate::api::Api;
use eframe::egui;
use eframe::egui::{Layout, Ui};
use egui::Context;
use std::collections::HashMap;

pub(crate) static ICON_AND_PORTRAIT_STORE: once_cell::race::OnceBox<
    HashMap<(String, String), egui_extras::RetainedImage>,
> = once_cell::race::OnceBox::new();

static STYLE_STRUCTURE: once_cell::race::OnceBox<Vec<(String, Vec<String>)>> =
    once_cell::race::OnceBox::new();

pub(crate) async fn init_icon_store() -> Option<()> {
    let mut style_structure = Vec::new();
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
                style_names.push(sty_name.clone());
                log::debug!("add icon for {}({})", name, sty_name);
                let key = (name.clone(), sty_name);
                map.insert(key, value);
            }
            style_structure.push((name, style_names));
        }
        map
    };

    STYLE_STRUCTURE.set(Box::new(style_structure));
    ICON_AND_PORTRAIT_STORE.set(Box::new(icons)).ok()
}

#[tokio::test]
async fn test_init_icon_store() {
    api::init();
    init_icon_store().await;
    ICON_AND_PORTRAIT_STORE.get().unwrap();
}

pub struct CharaChangeButton<'a> {
    current_character: (&'a str, &'a str),
}

impl<'a> CharaChangeButton<'a> {
    ///
    /// notify はいま開いているボタンを通知するために使用する.
    ///
    pub fn new(character: &'a str, style: &'a str) -> Self {
        Self {
            current_character: (character, style),
        }
    }
    pub fn ui(&mut self, ui: &mut Ui, ctx: &Context) -> Option<(&'a str, &'a str)> {
        let mut rt = None;
        let image = ICON_AND_PORTRAIT_STORE.get()?;
        let style_structure = STYLE_STRUCTURE.get()?;
        let image_ref = image.get(&(
            self.current_character.0.to_owned(),
            self.current_character.1.to_owned(),
        ))?;
        ui.menu_image_button(
            self.current_character.0,
            image_ref.texture_id(ctx),
            egui::vec2(32.0, 32.0),
            |ui| {
                for (character, styles) in style_structure {
                    if let Some(default_style) = styles.get(0) {
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
                                            for style in styles {
                                                if let Some(style_icon) =
                                                    image.get(&(character.clone(), style.clone()))
                                                {
                                                    let style_button = egui::Button::image_and_text(
                                                        style_icon.texture_id(ctx),
                                                        egui::epaint::vec2(32.0, 32.0),
                                                        format!("{}({})", character, style),
                                                    );
                                                    if ui.add(style_button).clicked() {
                                                        log::debug!(
                                                            "emit character select ({},{})",
                                                            &character,
                                                            &style
                                                        );
                                                        rt = Some((
                                                            character.as_str(),
                                                            style.as_str(),
                                                        ));
                                                    }
                                                }
                                            }
                                        },
                                    )
                                    .response
                                    .clicked()
                                {
                                    log::debug!(
                                        "emit character select ({},{})",
                                        &character,
                                        &default_style
                                    );
                                    rt = Some((character.as_str(), default_style.as_str()));
                                }
                            } else {
                                if ui.add(default_style_button).clicked() {
                                    log::debug!(
                                        "emit character select ({},{})",
                                        &character,
                                        &default_style
                                    );
                                    rt = Some((character.as_str(), default_style.as_str()));
                                }
                            }
                        }
                    }
                }
            },
        );

        rt
    }
}
