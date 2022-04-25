use crate::api_schema::AudioQuery;
use eframe::egui;
use eframe::egui::{Color32, Rect, Response, Sense, Stroke, Ui, Widget};
use std::ops::RangeInclusive;

pub fn render_synthesis_control(aq: &mut AudioQuery, ui: &mut Ui) {
    ui.vertical(|ui| {
        ui.label("話速");
        let slider = eframe::egui::Slider::new(&mut aq.speed_scale, 0.50..=2.0);
        ui.add(slider);
        ui.label("音高");
        let slider = eframe::egui::Slider::new(&mut aq.pitch_scale, -0.15..=0.15);
        ui.add(slider);
        ui.label("抑揚");
        let slider = eframe::egui::Slider::new(&mut aq.intonation_scale, 0.0..=2.0);
        ui.add(slider);
        ui.label("音量");
        let slider = eframe::egui::Slider::new(&mut aq.volume_scale, 0.0..=2.0);
        ui.add(slider);
        ui.label("開始無音");
        let slider = eframe::egui::Slider::new(&mut aq.pre_phoneme_length, 0.0..=1.5);
        ui.add(slider);
        ui.label("終了無音");
        let slider = eframe::egui::Slider::new(&mut aq.post_phoneme_length, 0.0..=1.5);
        ui.add(slider);
    });
}
