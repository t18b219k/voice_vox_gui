use crate::api_schema::AudioQuery;

use crate::history::Command;
use eframe::egui::Ui;

pub fn render_synthesis_control(
    aq_prev: &AudioQuery,
    uuid: &str,
    ui: &mut Ui,
) -> Option<AudioQueryEditCommand> {
    let mut rt = None;
    let mut aq = aq_prev.clone();
    ui.vertical(|ui| {
        ui.label("話速");
        let slider = eframe::egui::Slider::new(&mut aq.speed_scale, 0.50..=2.0);
        if ui.add(slider).drag_released() {
            rt = Some(AudioQueryEditCommand {
                diff: AudioQueryEditTarget::SpeedScale(aq.speed_scale - aq_prev.speed_scale),
                uuid: uuid.to_owned(),
            });
            return;
        }
        ui.label("音高");
        let slider = eframe::egui::Slider::new(&mut aq.pitch_scale, -0.15..=0.15);
        if ui.add(slider).drag_released() {
            rt = Some(AudioQueryEditCommand {
                diff: AudioQueryEditTarget::PitchScale(aq.pitch_scale - aq_prev.pitch_scale),
                uuid: uuid.to_owned(),
            });
            return;
        }
        ui.label("抑揚");
        let slider = eframe::egui::Slider::new(&mut aq.intonation_scale, 0.0..=2.0);
        if ui.add(slider).drag_released() {
            rt = Some(AudioQueryEditCommand {
                diff: AudioQueryEditTarget::IntonationScale(
                    aq.intonation_scale - aq_prev.intonation_scale,
                ),
                uuid: uuid.to_owned(),
            });
            return;
        }
        ui.label("音量");
        let slider = eframe::egui::Slider::new(&mut aq.volume_scale, 0.0..=2.0);
        if ui.add(slider).drag_released() {
            rt = Some(AudioQueryEditCommand {
                diff: AudioQueryEditTarget::VolumeScale(aq.volume_scale - aq_prev.volume_scale),
                uuid: uuid.to_owned(),
            });
            return;
        }
        ui.label("開始無音");
        let slider = eframe::egui::Slider::new(&mut aq.pre_phoneme_length, 0.0..=1.5);
        if ui.add(slider).drag_released() {
            rt = Some(AudioQueryEditCommand {
                diff: AudioQueryEditTarget::PrePhonemeLength(
                    aq.pre_phoneme_length - aq_prev.pre_phoneme_length,
                ),
                uuid: uuid.to_owned(),
            });
            return;
        }
        ui.label("終了無音");
        let slider = eframe::egui::Slider::new(&mut aq.post_phoneme_length, 0.0..=1.5);
        if ui.add(slider).drag_released() {
            rt = Some(AudioQueryEditCommand {
                diff: AudioQueryEditTarget::PostPhonemeLength(
                    aq.post_phoneme_length - aq_prev.post_phoneme_length,
                ),
                uuid: uuid.to_owned(),
            });
            return;
        }
    });
    rt
}

pub struct AudioQueryEditCommand {
    uuid: String,
    diff: AudioQueryEditTarget,
}

pub enum AudioQueryEditTarget {
    SpeedScale(f32),
    PitchScale(f32),
    IntonationScale(f32),
    VolumeScale(f32),
    PrePhonemeLength(f32),
    PostPhonemeLength(f32),
}

impl Command for AudioQueryEditCommand {
    fn invoke(&mut self, project: &mut crate::VoiceVoxProject) {
        if let Some(cell) = project.audioItems.get_mut(&self.uuid) {
            if let Some(query) = &mut cell.query {
                match self.diff {
                    AudioQueryEditTarget::SpeedScale(x) => {
                        query.speed_scale += x;
                    }
                    AudioQueryEditTarget::PitchScale(x) => {
                        query.pitch_scale += x;
                    }
                    AudioQueryEditTarget::VolumeScale(x) => {
                        query.volume_scale += x;
                    }
                    AudioQueryEditTarget::PrePhonemeLength(x) => {
                        query.pre_phoneme_length += x;
                    }
                    AudioQueryEditTarget::PostPhonemeLength(x) => {
                        query.post_phoneme_length += x;
                    }
                    AudioQueryEditTarget::IntonationScale(x) => {
                        query.intonation_scale += x;
                    }
                };
            }
        }
    }

    fn undo(&mut self, project: &mut crate::VoiceVoxProject) {
        if let Some(cell) = project.audioItems.get_mut(&self.uuid) {
            if let Some(query) = &mut cell.query {
                match self.diff {
                    AudioQueryEditTarget::SpeedScale(x) => {
                        query.speed_scale -= x;
                    }
                    AudioQueryEditTarget::PitchScale(x) => {
                        query.pitch_scale -= x;
                    }
                    AudioQueryEditTarget::VolumeScale(x) => {
                        query.volume_scale -= x;
                    }
                    AudioQueryEditTarget::PrePhonemeLength(x) => {
                        query.pre_phoneme_length -= x;
                    }
                    AudioQueryEditTarget::PostPhonemeLength(x) => {
                        query.post_phoneme_length -= x;
                    }
                    AudioQueryEditTarget::IntonationScale(x) => {
                        query.intonation_scale -= x;
                    }
                };
            }
        }
    }

    fn op_name(&self) -> &str {
        match self.diff {
            AudioQueryEditTarget::SpeedScale(_) => "話速",
            AudioQueryEditTarget::PitchScale(_) => "音高",
            AudioQueryEditTarget::VolumeScale(_) => "音量",
            AudioQueryEditTarget::PrePhonemeLength(_) => "開始無音",
            AudioQueryEditTarget::PostPhonemeLength(_) => "終了無音",
            AudioQueryEditTarget::IntonationScale(_) => "抑揚",
        }
    }
}
