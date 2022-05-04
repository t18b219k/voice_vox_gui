use crate::api_schema::AudioQuery;

use crate::commands::AudioQueryEditCommand;

use eframe::egui::Ui;

pub fn render_synthesis_control(
    aq_prev: &AudioQuery,
    ui: &mut Ui,
) -> Option<AudioQueryEditCommand> {
    let mut rt = None;
    let mut aq = aq_prev.clone();
    ui.vertical(|ui| {
        ui.label(format!("話速 {}", aq.speedScale));
        let slider = eframe::egui::Slider::new(&mut aq.speedScale, 0.50..=2.0).show_value(false);
        if ui.add(slider).drag_released() {
            rt = Some(AudioQueryEditCommand::SpeedScale(
                aq.speedScale - aq_prev.speedScale,
            ));
            return;
        }
        ui.label(format!("音高 {}", aq.pitchScale));
        let slider = eframe::egui::Slider::new(&mut aq.pitchScale, -0.15..=0.15).show_value(false);
        if ui.add(slider).drag_released() {
            rt = Some(AudioQueryEditCommand::PitchScale(
                aq.pitchScale - aq_prev.pitchScale,
            ));
            return;
        }
        ui.label(format!("抑揚 {}", aq.intonationScale));
        let slider =
            eframe::egui::Slider::new(&mut aq.intonationScale, 0.0..=2.0).show_value(false);
        if ui.add(slider).drag_released() {
            rt = Some(AudioQueryEditCommand::IntonationScale(
                aq.intonationScale - aq_prev.intonationScale,
            ));
            return;
        }
        ui.label(format!("音量 {}", aq.volumeScale));
        let slider = eframe::egui::Slider::new(&mut aq.volumeScale, 0.0..=2.0).show_value(false);
        if ui.add(slider).drag_released() {
            rt = Some(AudioQueryEditCommand::VolumeScale(
                aq.volumeScale - aq_prev.volumeScale,
            ));
            return;
        }
        ui.label(format!("開始無音 {}", aq.prePhonemeLength));
        let slider =
            eframe::egui::Slider::new(&mut aq.prePhonemeLength, 0.0..=1.5).show_value(false);
        if ui.add(slider).drag_released() {
            rt = Some(AudioQueryEditCommand::PrePhonemeLength(
                aq.prePhonemeLength - aq_prev.prePhonemeLength,
            ));
            return;
        }
        ui.label(format!("終了無音 {}", aq.postPhonemeLength));
        let slider =
            eframe::egui::Slider::new(&mut aq.postPhonemeLength, 0.0..=1.5).show_value(false);
        if ui.add(slider).drag_released() {
            rt = Some(AudioQueryEditCommand::PostPhonemeLength(
                aq.postPhonemeLength - aq_prev.postPhonemeLength,
            ));
            return;
        }
    });
    rt
}
