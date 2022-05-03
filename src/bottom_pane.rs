use crate::api_schema::AccentPhrase;
use crate::commands::BottomPaneCommand;

use eframe::egui::{
    Align, Align2, FontId, Layout, NumExt, Response, SelectableLabel, TextStyle, Ui, Vec2, Widget,
};
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
    should_play: &mut Option<bool>,
    ui: &mut Ui,
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

                if false {
                    let rounding = Rounding::none();
                    painter.rect(box_rect, rounding, Color32::BLACK, Stroke::none());
                    if response.clicked() {
                        should_play.replace(false);
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
                        should_play.replace(true);
                    }
                }
            })
            .response
        });

        ui.separator();
        let alloc_height = ui.available_height() * 1.2;

        let scroll = eframe::egui::containers::ScrollArea::horizontal()
            .max_height(alloc_height)
            .auto_shrink([false, false]);
        scroll.show(ui, |ui| {
            ui.set_height(alloc_height / 1.2);
            ui.vertical(|ui| {
                let mut space = ui.spacing().item_spacing;
                space.y = ui.available_height() / 1.2;
                space.x *= 6.0;
                match current_displaying {
                    Displaying::Accent => {
                        let accent_phrase_len = edit_targets.len();
                        if !edit_targets.is_empty() {
                            ui.horizontal(|ui| {
                                for (ap, edit_target) in edit_targets.iter().enumerate() {
                                    let mut accent = edit_target.accent;
                                    let mora_len = edit_target.moras.len();
                                    let width = mora_len as f32 * space.x;
                                    ui.set_height(space.y);
                                    ui.vertical(|ui| {
                                        let slider = eframe::egui::Slider::new(
                                            &mut accent,
                                            1..=mora_len as i32,
                                        )
                                        .integer()
                                        .show_value(false);
                                        ui.style_mut().spacing.slider_width = width;
                                        let thickness = ui
                                            .text_style_height(&TextStyle::Body)
                                            .at_least(ui.spacing().interact_size.y);
                                        let radius = thickness / 2.5;
                                        let res = ui.add(slider);
                                        if (res.clicked() | res.drag_released())
                                            & (accent != edit_target.accent)
                                        {
                                            //emit signal.
                                            rt = Some(BottomPaneCommand::AccentPhrase {
                                                accent_phrase: ap,
                                                new_accent: accent as usize,
                                                prev_accent: edit_target.accent as usize,
                                            });
                                        }
                                        let h = ui.available_height();
                                        let w = res.rect.width();
                                        let (r, painter) = ui.allocate_painter(
                                            vec2(w, h),
                                            Sense::focusable_noninteractive(),
                                        );
                                        let rect = r.rect;

                                        let left = rect.left();
                                        let top = rect.top();
                                        let bottom = rect.bottom();

                                        let text_height = thickness;
                                        //
                                        let mut graph_pos = bottom - text_height;

                                        use eframe::egui::pos2;
                                        let mut line_points = vec![];
                                        let width_per_mora = (w - radius * 2.0)
                                            / ((edit_target.moras.len() - 1) as f32);
                                        for (idx, mora) in edit_target.moras.iter().enumerate() {
                                            let x = width_per_mora * idx as f32;
                                            if (idx + 1) == edit_target.accent as usize {
                                                painter.vline(
                                                    left + x + radius,
                                                    top..=bottom - text_height,
                                                    Stroke::new(2.0, Color32::LIGHT_GREEN),
                                                );
                                            } else {
                                                let dash_len = height / 10.0;
                                                painter.add(Shape::dashed_line(
                                                    &[
                                                        pos2(left + x + radius, top),
                                                        pos2(
                                                            left + x + radius,
                                                            bottom - text_height,
                                                        ),
                                                    ],
                                                    Stroke::new(1.0, Color32::LIGHT_GREEN),
                                                    dash_len,
                                                    dash_len,
                                                ));
                                            };

                                            painter.text(
                                                pos2(left + x, bottom - text_height),
                                                Align2::LEFT_TOP,
                                                &mora.text,
                                                FontId::default(),
                                                Color32::BLACK,
                                            );
                                            if idx + 1 == accent as usize {
                                                graph_pos = top;
                                            } else if idx + 1 > accent as usize {
                                                graph_pos = bottom - text_height;
                                            } else if (idx + 1 < accent as usize) & (idx + 1 != 1) {
                                                graph_pos = top;
                                            }
                                            line_points.push(pos2(left + x + radius, graph_pos));
                                        }
                                        use eframe::egui::epaint;
                                        use epaint::Shape;
                                        let shape = Shape::line(
                                            line_points,
                                            Stroke::new(2.0, Color32::BLACK),
                                        );

                                        painter.add(shape);
                                    });

                                    if ap < accent_phrase_len - 1 {
                                        let button = eframe::egui::Button::new("");
                                        if ui.add_sized(space, button).clicked() {
                                            rt = Some(BottomPaneCommand::Concat {
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
                        let accent_phrase_len = edit_targets.len();
                        if !edit_targets.is_empty() {
                            ui.horizontal(|ui| {
                                for (ap, edit_target) in edit_targets.iter().enumerate() {
                                    let mora_len = edit_target.moras.len();
                                    for (index, mora) in edit_target.moras.iter().enumerate() {
                                        let mut pitch = mora.pitch;
                                        let slider =
                                            eframe::egui::Slider::new(&mut pitch, 3.0..=6.5)
                                                .vertical()
                                                .text(&mora.text)
                                                .show_value(false);
                                        let res = ui.add(slider);

                                        if (res.clicked() | res.drag_released())
                                            & ((pitch - mora.pitch).abs() > f32::EPSILON)
                                        {
                                            //emit signal.
                                            rt = Some(BottomPaneCommand::Pitch {
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

                                            let vowel_diff = if (res.clicked()
                                                | res.drag_released())
                                                & ((vowel - mora.vowel_length).abs() > f32::EPSILON)
                                            {
                                                log::debug!("vowel {}", vowel);
                                                Some(vowel - mora.vowel_length)
                                            } else {
                                                None
                                            };
                                            let consonant_diff = if (res.clicked()
                                                | res.drag_released())
                                                & ((consonant - prev_consonant).abs()
                                                    > f32::EPSILON)
                                            {
                                                log::debug!("consonant {}", consonant);
                                                Some(consonant - prev_consonant)
                                            } else {
                                                None
                                            };
                                            if res.clicked() | res.drag_released() {
                                                //emit signal.
                                                rt = Some(BottomPaneCommand::VowelAndConsonant {
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
    });
    rt
}

pub struct TwoNotchSlider<'a> {
    pub a: &'a mut f32,
    pub b: &'a mut f32,
    pub range: RangeInclusive<f32>,
    pub text: String,
}

impl<'a> TwoNotchSlider<'a> {
    fn slider_ui(self, ui: &mut Ui) -> Response {
        use eframe::egui;
        use egui::epaint::{pos2, vec2, Color32, Rect, Stroke};
        use egui::Sense;
        let width_of_rail = 8.0;

        let height = ui.available_height();

        let (_id, rect) = ui.allocate_space(vec2(width_of_rail * 2.0, ui.available_height()));
        let origin = rect.min;

        let res_left = ui.allocate_rect(
            Rect::from_min_max(origin, origin + vec2(width_of_rail, height)),
            Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);
        painter.vline(
            origin.x + width_of_rail,
            origin.y..=origin.y + height,
            Stroke::new(width_of_rail, Color32::LIGHT_GRAY),
        );

        let bottom = origin.y + height;

        let width_of_range = *self.range.end() - *self.range.start();

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
        let center = pos2(origin.x + width_of_rail, bottom - a_points_from_bottom);

        let thickness = ui
            .text_style_height(&TextStyle::Body)
            .at_least(ui.spacing().interact_size.y);
        let radius = thickness / 2.5;

        let (left, right) = HALF_CIRCLE.get_or_init(|| {
            use eframe::epaint::Shape;
            // - pi/2 -> pi/2
            let offset = -std::f32::consts::FRAC_PI_2;
            let unit_angle = std::f32::consts::TAU / 24.0;

            let right = (0..=12).into_iter().map(|x| {
                let phase = x as f32 * unit_angle + offset;
                let (sin, cos) = phase.sin_cos();
                pos2(cos * radius, sin * radius)
            });
            let left = (12..=24).into_iter().map(|x| {
                let phase = x as f32 * unit_angle + offset;
                let (sin, cos) = phase.sin_cos();
                pos2(cos * radius, sin * radius)
            });
            let right = Shape::convex_polygon(
                right.collect(),
                Color32::LIGHT_GRAY,
                Stroke::new(1.0, Color32::BLACK),
            );
            let left = Shape::convex_polygon(
                left.collect(),
                Color32::LIGHT_GRAY,
                Stroke::new(1.0, Color32::BLACK),
            );
            (left, right)
        });

        let mut left = left.clone();
        left.translate(center.to_vec2());
        if res_left.hovered() {
            painter.circle_filled(center, radius * 1.4, Color32::LIGHT_GREEN);
        }
        painter.add(left);

        let mut right = right.clone();

        let right_origin = origin + vec2(width_of_rail, 0.0);
        let res_right = ui.allocate_rect(
            Rect::from_min_max(right_origin, right_origin + vec2(width_of_rail, height)),
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
        let center = pos2(origin.x + width_of_rail, bottom - b_points_from_bottom);
        right.translate(center.to_vec2());
        if res_right.hovered() {
            painter.circle_filled(center, radius * 1.4, Color32::LIGHT_GREEN);
        }
        painter.add(right);
        *self.a = f32::clamp(a, *self.range.start(), *self.range.end());
        *self.b = f32::clamp(b, *self.range.start(), *self.range.end());
        res_left.union(res_right)
    }
}

static HALF_CIRCLE: once_cell::sync::OnceCell<(eframe::epaint::Shape, eframe::epaint::Shape)> =
    once_cell::sync::OnceCell::new();

impl<'a> Widget for TwoNotchSlider<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
            ui.label(&self.text);
            {
                self.slider_ui(ui)
            }
        })
        .inner
    }
}
