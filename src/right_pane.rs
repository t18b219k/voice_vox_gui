use crate::api_schema::AudioQuery;

use crate::commands::AudioQueryEditCommand;
use crate::history::Command;
use eframe::egui::Ui;

pub fn render_synthesis_control(
    aq_prev: &AudioQuery,
    ui: &mut Ui,
) -> Option<AudioQueryEditCommand> {
    let mut rt = None;
    let mut aq = aq_prev.clone();
    ui.vertical(|ui| {
        ui.label(format!("話速 {}", aq.speed_scale));
        let slider = eframe::egui::Slider::new(&mut aq.speed_scale, 0.50..=2.0).show_value(false);
        if ui.add(slider).drag_released() {
            rt = Some(AudioQueryEditCommand::SpeedScale(
                aq.speed_scale - aq_prev.speed_scale,
            ));
            return;
        }
        ui.label(format!("音高 {}", aq.pitch_scale));
        let slider = eframe::egui::Slider::new(&mut aq.pitch_scale, -0.15..=0.15).show_value(false);
        if ui.add(slider).drag_released() {
            rt = Some(AudioQueryEditCommand::PitchScale(
                aq.pitch_scale - aq_prev.pitch_scale,
            ));
            return;
        }
        ui.label(format!("抑揚 {}", aq.intonation_scale));
        let slider =
            eframe::egui::Slider::new(&mut aq.intonation_scale, 0.0..=2.0).show_value(false);
        if ui.add(slider).drag_released() {
            rt = Some(AudioQueryEditCommand::IntonationScale(
                aq.intonation_scale - aq_prev.intonation_scale,
            ));
            return;
        }
        ui.label(format!("音量 {}", aq.volume_scale));
        let slider = eframe::egui::Slider::new(&mut aq.volume_scale, 0.0..=2.0).show_value(false);
        if ui.add(slider).drag_released() {
            rt = Some(AudioQueryEditCommand::VolumeScale(
                aq.volume_scale - aq_prev.volume_scale,
            ));
            return;
        }
        ui.label(format!("開始無音 {}", aq.pre_phoneme_length));
        let slider =
            eframe::egui::Slider::new(&mut aq.pre_phoneme_length, 0.0..=1.5).show_value(false);
        if ui.add(slider).drag_released() {
            rt = Some(AudioQueryEditCommand::PrePhonemeLength(
                aq.pre_phoneme_length - aq_prev.pre_phoneme_length,
            ));
            return;
        }
        ui.label(format!("終了無音 {}", aq.post_phoneme_length));
        let slider =
            eframe::egui::Slider::new(&mut aq.post_phoneme_length, 0.0..=1.5).show_value(false);
        if ui.add(slider).drag_released() {
            rt = Some(AudioQueryEditCommand::PostPhonemeLength(
                aq.post_phoneme_length - aq_prev.post_phoneme_length,
            ));
            return;
        }
    });
    rt
}
