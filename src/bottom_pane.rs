use crate::api_schema::AccentPhrase;
use crate::history::Command;
use crate::project::VoiceVoxProject;
use eframe::egui::{Align, FontId, Layout, SelectableLabel, Ui, Vec2};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Displaying {
    Accent,
    Intonation,
    Length,
}

pub fn create_bottom_pane(
    current_displaying: &mut Displaying,
    playing: &mut bool,
    ui: &mut Ui,
    edit_target: &[AccentPhrase],
) -> Option<BottomPaneCommand> {
    ui.horizontal(|ui| {
        use eframe::egui::{pos2, vec2, Color32, Rect, Rounding, Sense, Stroke};
        use Displaying::*;

        const BUTTONS: [(Displaying, &str); 3] = [
            (Accent, "アクセント"),
            (Intonation, "イントネーション"),
            (Length, "長さ"),
        ];

        let Vec2 {
            x: mut width,
            y: mut height,
        } = ui
            .painter()
            .layout(
                BUTTONS[1].1.to_owned(),
                FontId::default(),
                Default::default(),
                ui.available_width(),
            )
            .rect
            .size();

        width += 2.0 * ui.spacing().button_padding.x;
        height += 2.0 * ui.spacing().button_padding.y;

        let size = vec2(width, height * 3.0 + 64.0);

        ui.add_sized(size, |ui: &mut Ui| {
            ui.vertical_centered(|ui| {
                for button in BUTTONS {
                    use eframe::egui::Button;
                    if ui
                        .add_sized(
                            vec2(width, height),
                            SelectableLabel::new(*current_displaying == button.0, button.1),
                        )
                        .clicked()
                    {
                        *current_displaying = button.0;
                    }
                }

                let (response, painter) = ui.allocate_painter(vec2(64.0, 64.0), Sense::click());
                let center = response.rect.center();
                painter.circle_filled(center, 32.0, Color32::DARK_GREEN);

                if *playing {
                    let size = vec2(8.0, 8.0);
                    let top_left = center - size;
                    let bottom_right = center + size;
                    let rounding = Rounding::none();
                    painter.rect(
                        Rect::from_min_max(top_left, bottom_right),
                        rounding,
                        Color32::BLACK,
                        Stroke::none(),
                    );
                    if response.clicked() {
                        *playing = false;
                    }
                } else {
                    use eframe::egui::epaint::PathShape;
                    use eframe::egui::Shape;
                    let triangle_width = 16.0;
                    let half_height = (triangle_width / 3.0_f32.sqrt());
                    let top_left = center - vec2((triangle_width / 2.0), half_height);

                    let positions = vec![
                        top_left,
                        top_left + vec2(0.0, half_height * 2.0),
                        top_left + vec2(16.0, half_height),
                    ];
                    let points =
                        PathShape::convex_polygon(positions, Color32::BLACK, Stroke::none());
                    let shape = Shape::Path(points);
                    painter.add(shape);
                    if response.clicked() {
                        *playing = true;
                    }
                }
            })
            .response
        });

        ui.separator();
        match current_displaying {
            Displaying::Accent => {}
            Displaying::Intonation => {}
            Displaying::Length => {}
        }
    });
    None
}

pub enum BottomPaneCommand {
    ///
    /// [[ニ],[ホ],[ン]]*[[シ],[マ],[グ],[ニ]]
    ///  ->
    /// ```
    ///  Concat{
    ///     accent_phrase:0,
    ///     length:3,
    ///    }
    /// ```
    ///
    Concat {
        uuid: String,
        accent_phrase: usize,
        length: usize,
    },
    ///
    ///
    ///  [ [ニ] [ホ] * [ン] ],[[シ] [マ] [グ] [ニ]]
    ///
    /// ->```
    /// Split{accent_phrase:0,mora:2}
    /// ```
    Split {
        uuid: String,
        accent_phrase: usize,
        mora: usize,
    },
}

impl Command for BottomPaneCommand {
    fn invoke(&mut self, project: &mut VoiceVoxProject) {
        match self {
            BottomPaneCommand::Concat {
                uuid,
                accent_phrase: index,
                length,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        assert!(*index + 1 < aq.accent_phrases.len());
                        let right_moras = aq.accent_phrases[*index + 1].moras.clone();
                        aq.accent_phrases[*index]
                            .moras
                            .extend_from_slice(&right_moras);
                        aq.accent_phrases.remove(*index + 1);
                    }
                }
            }
            BottomPaneCommand::Split {
                uuid,
                accent_phrase: index,
                mora,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        assert!(*index < aq.accent_phrases.len());
                        let insert = crate::api_schema::AccentPhrase {
                            moras: aq.accent_phrases[*index].moras.split_off(*mora),
                            accent: 0,
                            pause_mora: None,
                            is_interrogative: None,
                        };
                        aq.accent_phrases.insert(*index + 1, insert);
                    }
                }
            }
        }
    }

    fn undo(&mut self, project: &mut VoiceVoxProject) {
        match self {
            BottomPaneCommand::Concat {
                uuid,
                accent_phrase: index,
                length,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        assert!(*index < aq.accent_phrases.len());
                        let insert = crate::api_schema::AccentPhrase {
                            moras: aq.accent_phrases[*index].moras.split_off(*length),
                            accent: 0,
                            pause_mora: None,
                            is_interrogative: None,
                        };
                        aq.accent_phrases.insert(*index + 1, insert);
                    }
                }
            }
            BottomPaneCommand::Split {
                uuid,
                accent_phrase: index,
                mora,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        assert!(*index + 1 < aq.accent_phrases.len());
                        let right_moras = aq.accent_phrases[*index + 1].moras.clone();
                        aq.accent_phrases[*index]
                            .moras
                            .extend_from_slice(&right_moras);
                        aq.accent_phrases.remove(*index + 1);
                    }
                }
            }
        }
    }

    fn op_name(&self) -> &str {
        match self {
            BottomPaneCommand::Concat { .. } => "アクセントフレーズ連結",
            BottomPaneCommand::Split { .. } => "アクセントフレーズ分割",
        }
    }
}
