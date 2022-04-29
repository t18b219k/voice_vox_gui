use crate::api_schema::AccentPhrase;
use crate::history::Command;
use crate::project::VoiceVoxProject;
use eframe::egui::{FontId, Response, SelectableLabel, Ui, Vec2, Widget};
use std::ops::RangeInclusive;

/// アクセント位置とアクセント句の変化で新しくリクエストを送る必要がある.
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
    uuid: &str,
    edit_targets: &[AccentPhrase],
) -> Option<BottomPaneCommand> {
    let mut rt = None;
    ui.horizontal(|ui| {
        use eframe::egui::{vec2, Color32, Rounding, Sense, Stroke};
        use Displaying::*;

        let radius = 32.0;
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

        let size = vec2(width, height * 3.0 + radius * 2.0);

        ui.add_sized(size, |ui: &mut Ui| {
            ui.vertical_centered(|ui| {
                for button in BUTTONS {
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

                let (response, painter) =
                    ui.allocate_painter(vec2(radius * 2.0, radius * 2.0), Sense::click());
                let center = response.rect.center();
                let box_rect = response.rect.shrink(radius * (3.0 / 4.0));
                painter.circle_filled(center, radius, Color32::DARK_GREEN);

                if *playing {
                    let rounding = Rounding::none();
                    painter.rect(box_rect, rounding, Color32::BLACK, Stroke::none());
                    if response.clicked() {
                        *playing = false;
                    }
                } else {
                    use eframe::egui::epaint::PathShape;
                    use eframe::egui::Shape;
                    let triangle_width = radius / 2.0;
                    let half_height = triangle_width / 3.0_f32.sqrt();
                    let top_left = center - vec2(triangle_width / 2.0, half_height);

                    let positions = vec![
                        top_left,
                        top_left + vec2(0.0, half_height * 2.0),
                        center + vec2(triangle_width / 2.0, 0.0),
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
        let scroll = eframe::egui::containers::ScrollArea::both();
        scroll.show(ui, |ui| {
            match current_displaying {
                Displaying::Accent => {
                    let mut space = ui.spacing().item_spacing;
                    space.y = ui.available_height();
                    space.x *= 2.0;
                    let accent_phrase_len = edit_targets.len();
                    if !edit_targets.is_empty() {
                        ui.horizontal(|ui| {
                            for (ap, edit_target) in edit_targets.iter().enumerate() {
                                let mut accent = edit_target.accent;
                                let mora_len = edit_target.moras.len();
                                let width = mora_len as f32 * space.x;

                                ui.vertical(|ui| {
                                    let slider =
                                        eframe::egui::Slider::new(&mut accent, 1..=mora_len as i32)
                                            .integer()
                                            .show_value(false);
                                    let res = ui.add_sized(vec2(width, 16.0), slider);
                                    if (res.clicked() | res.drag_released())
                                        & (accent != edit_target.accent)
                                    {
                                        //emit signal.
                                        rt = Some(BottomPaneCommand::AccentPhrase {
                                            uuid: uuid.to_owned(),
                                            accent_phrase: ap,
                                            new_accent: accent as usize,
                                            prev_accent: edit_target.accent as usize,
                                        });
                                    }
                                });

                                if ap < accent_phrase_len - 1 {
                                    let button = eframe::egui::Button::new("");
                                    if ui.add_sized(space, button).clicked() {
                                        rt = Some(BottomPaneCommand::Concat {
                                            uuid: uuid.to_owned(),
                                            accent_phrase: ap,
                                            length: mora_len,
                                        });
                                    }
                                }
                            }
                        });
                    }
                }
                Displaying::Intonation => {
                    let mut space = ui.spacing().item_spacing;
                    space.y = ui.available_height();
                    space.x *= 2.0;
                    let accent_phrase_len = edit_targets.len();
                    if !edit_targets.is_empty() {
                        ui.horizontal(|ui| {
                            for (ap, edit_target) in edit_targets.iter().enumerate() {
                                let mora_len = edit_target.moras.len();
                                for (index, mora) in edit_target.moras.iter().enumerate() {
                                    let mut pitch = mora.pitch;
                                    let slider = eframe::egui::Slider::new(&mut pitch, 3.0..=6.5)
                                        .vertical()
                                        .text(&mora.text)
                                        .show_value(false);
                                    let res = ui.add(slider);

                                    if (res.clicked() | res.drag_released())
                                        & ((pitch - mora.pitch).abs() > f32::EPSILON)
                                    {
                                        //emit signal.
                                        rt = Some(BottomPaneCommand::Pitch {
                                            uuid: uuid.to_owned(),
                                            accent_phrase: ap,
                                            mora: index,
                                            pitch_diff: pitch - mora.pitch,
                                        });
                                    }
                                }
                                if ap < accent_phrase_len - 1 {
                                    let button = eframe::egui::Button::new("");
                                    if ui.add_sized(space, button).clicked() {
                                        rt = Some(BottomPaneCommand::Concat {
                                            uuid: uuid.to_owned(),
                                            accent_phrase: ap,
                                            length: mora_len,
                                        });
                                    }
                                }
                            }
                        });
                    }
                }
                Displaying::Length => {
                    let mut space = ui.spacing().item_spacing;
                    space.y = ui.available_height();
                    space.x *= 2.0;
                    let accent_phrase_len = edit_targets.len();
                    if !edit_targets.is_empty() {
                        ui.horizontal(|ui| {
                            for (ap, edit_target) in edit_targets.iter().enumerate() {
                                let mora_len = edit_target.moras.len();
                                for (index, mora) in edit_target.moras.iter().enumerate() {
                                    if let Some(prev_consonant) = mora.consonant_length {
                                        let mut consonant = prev_consonant;
                                        let mut vowel = mora.vowel_length;
                                        let slider = TwoNotchSlider {
                                            a: &mut consonant,
                                            b: &mut vowel,
                                            range: 0.0..=0.30,
                                            text: mora.text.clone(),
                                        };
                                        let res = ui.add(slider);

                                        let vowel_diff = if (res.clicked() | res.drag_released())
                                            & ((vowel - mora.vowel_length).abs() > f32::EPSILON)
                                        {
                                            log::debug!("vowel {}", vowel);
                                            Some(vowel - mora.vowel_length)
                                        } else {
                                            None
                                        };
                                        let consonant_diff = if (res.clicked()
                                            | res.drag_released())
                                            & ((consonant - prev_consonant).abs() > f32::EPSILON)
                                        {
                                            log::debug!("consonant {}", consonant);
                                            Some(consonant - prev_consonant)
                                        } else {
                                            None
                                        };
                                        if res.clicked() | res.drag_released() {
                                            //emit signal.
                                            rt = Some(BottomPaneCommand::VowelAndConsonant {
                                                uuid: uuid.to_owned(),
                                                accent_phrase: ap,
                                                mora: index,
                                                vowel_diff,
                                                consonant_diff,
                                            });
                                        }
                                    } else {
                                        let mut vowel = mora.vowel_length;
                                        let slider =
                                            eframe::egui::Slider::new(&mut vowel, 0.0..=0.30)
                                                .vertical()
                                                .text(&mora.text)
                                                .show_value(false);
                                        let res = ui.add(slider);

                                        if (res.clicked() | res.drag_released())
                                            & ((vowel - mora.vowel_length).abs() > f32::EPSILON)
                                        {
                                            //emit signal.
                                            rt = Some(BottomPaneCommand::VowelAndConsonant {
                                                uuid: uuid.to_owned(),
                                                accent_phrase: ap,
                                                mora: index,
                                                vowel_diff: Some(vowel - mora.vowel_length),
                                                consonant_diff: None,
                                            });
                                        }
                                    };
                                }
                                if ap < accent_phrase_len - 1 {
                                    let button = eframe::egui::Button::new("");
                                    if ui.add_sized(space, button).clicked() {
                                        rt = Some(BottomPaneCommand::Concat {
                                            uuid: uuid.to_owned(),
                                            accent_phrase: ap,
                                            length: mora_len,
                                        });
                                    }
                                }
                            }
                        });
                    }
                }
            }
        });
    });
    rt
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

    AccentPhrase {
        uuid: String,
        accent_phrase: usize,
        new_accent: usize,
        prev_accent: usize,
    },
    Pitch {
        uuid: String,
        accent_phrase: usize,
        mora: usize,
        pitch_diff: f32,
    },
    VowelAndConsonant {
        uuid: String,
        accent_phrase: usize,
        mora: usize,
        vowel_diff: Option<f32>,
        consonant_diff: Option<f32>,
    },
}

impl Command for BottomPaneCommand {
    fn invoke(&mut self, project: &mut VoiceVoxProject) {
        match self {
            BottomPaneCommand::Concat {
                uuid,
                accent_phrase: index,
                length: _,
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
            BottomPaneCommand::AccentPhrase {
                uuid,
                accent_phrase,
                new_accent,
                prev_accent: _,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        aq.accent_phrases[*accent_phrase].accent = *new_accent as i32;
                    }
                }
            }
            BottomPaneCommand::Pitch {
                uuid,
                accent_phrase,
                mora,
                pitch_diff,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        aq.accent_phrases[*accent_phrase].moras[*mora].pitch += *pitch_diff;
                    }
                }
            }
            BottomPaneCommand::VowelAndConsonant {
                uuid,
                accent_phrase,
                mora,
                vowel_diff,
                consonant_diff,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        if let Some(vd) = vowel_diff {
                            aq.accent_phrases[*accent_phrase].moras[*mora].vowel_length += *vd;
                        }
                        if let Some(cd) = consonant_diff {
                            if let Some(consonant) =
                                &mut aq.accent_phrases[*accent_phrase].moras[*mora].consonant_length
                            {
                                *consonant += *cd;
                            }
                        }
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
                mora: _,
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
            BottomPaneCommand::AccentPhrase {
                uuid,
                accent_phrase,
                new_accent: _,
                prev_accent,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        aq.accent_phrases[*accent_phrase].accent = *prev_accent as i32;
                    }
                }
            }
            BottomPaneCommand::Pitch {
                uuid,
                accent_phrase,
                mora,
                pitch_diff,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        aq.accent_phrases[*accent_phrase].moras[*mora].pitch -= *pitch_diff;
                    }
                }
            }
            BottomPaneCommand::VowelAndConsonant {
                uuid,
                accent_phrase,
                mora,
                vowel_diff,
                consonant_diff,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        if let Some(vd) = vowel_diff {
                            aq.accent_phrases[*accent_phrase].moras[*mora].vowel_length -= *vd;
                        }
                        if let Some(cd) = consonant_diff {
                            if let Some(consonant) =
                                &mut aq.accent_phrases[*accent_phrase].moras[*mora].consonant_length
                            {
                                *consonant -= *cd;
                            }
                        }
                    }
                }
            }
        }
    }

    fn op_name(&self) -> &str {
        match self {
            BottomPaneCommand::Concat { .. } => "アクセントフレーズ連結",
            BottomPaneCommand::Split { .. } => "アクセントフレーズ分割",
            BottomPaneCommand::AccentPhrase { .. } => "アクセント位置変更",
            BottomPaneCommand::Pitch { .. } => "ピッチ変更",
            BottomPaneCommand::VowelAndConsonant { .. } => "母音子音長さ変更",
        }
    }
}

pub struct TwoNotchSlider<'a> {
    pub a: &'a mut f32,
    pub b: &'a mut f32,
    pub range: RangeInclusive<f32>,
    pub text: String,
}

impl<'a> Widget for TwoNotchSlider<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        use eframe::egui;
        use egui::epaint::{pos2, vec2, Color32, Rect, Rounding, Stroke};
        use egui::Sense;
        //left half size.
        let size_of_notch = vec2(8.0, 16.0);

        let height = ui.available_height() - (size_of_notch.y * 2.0);

        let (id, rect) = ui.allocate_space(vec2(size_of_notch.x * 2.0, ui.available_height()));
        let origin = rect.min;

        let res_left = ui.allocate_rect(
            Rect::from_min_max(origin, origin + vec2(size_of_notch.x, height)),
            Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);
        painter.vline(
            origin.x + (size_of_notch.x),
            origin.y..=origin.y + height,
            Stroke::new(size_of_notch.x, Color32::LIGHT_GRAY),
        );

        let bottom = origin.y + height;

        let width_of_range = (*self.range.end() - *self.range.start());

        let a = if let Some(pos) = res_left.interact_pointer_pos() {
            let cursor_pos = pos.y;
            let x = bottom - cursor_pos;
            let scale = x / height;
            egui::lerp(self.range.clone(), scale)
        } else {
            *self.a
        };

        let a_diff_from_start = a - *self.range.start();
        let a_points_from_bottom = (a_diff_from_start / width_of_range) * height;
        let center = pos2(
            origin.x + size_of_notch.x * 0.5,
            bottom - a_points_from_bottom,
        );
        painter.rect(
            Rect::from_center_size(center, size_of_notch),
            Rounding::none(),
            Color32::BLUE,
            Stroke::none(),
        );

        let right_origin = origin + vec2(size_of_notch.x, 0.0);
        let res_right = ui.allocate_rect(
            Rect::from_min_max(right_origin, right_origin + vec2(size_of_notch.x, height)),
            Sense::click_and_drag(),
        );
        let b = if let Some(pos) = res_right.interact_pointer_pos() {
            let cursor_pos = pos.y;
            let x = bottom - cursor_pos;
            let scale = x / height;
            egui::lerp(self.range.clone(), scale)
        } else {
            *self.b
        };

        let b_diff_from_start = b - *self.range.start();
        let b_points_from_bottom = (b_diff_from_start / width_of_range) * height;
        let center = pos2(
            right_origin.x + size_of_notch.x * 0.5,
            bottom - b_points_from_bottom,
        );
        painter.rect(
            Rect::from_center_size(center, size_of_notch),
            Rounding::none(),
            Color32::BLUE,
            Stroke::none(),
        );

        *self.a = f32::clamp(a, *self.range.start(), *self.range.end());
        *self.b = f32::clamp(b, *self.range.start(), *self.range.end());
        res_left.union(res_right)
    }
}
