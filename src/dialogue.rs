use eframe::egui;
use eframe::egui::{Align2, Color32, Context, FontFamily, FontId, Layout, Response, Vec2};

pub trait DialogueSelectable<T: PartialEq + Clone> {
    /// layout of control.
    ///  if button clicked returned value set to Dialogue result.
    fn layout(&self) -> Vec<Option<(T, &str)>>;
}

pub struct ExitControl;

impl DialogueSelectable<bool> for ExitControl {
    fn layout(&self) -> Vec<Option<(bool, &str)>> {
        vec![Some((false, "キャンセル")), None, Some((true, "終了"))]
    }
}
pub struct Dialogue<'a, 'cell, T: PartialEq + Clone> {
    pub(crate) title: &'a str,
    pub(crate) text: &'a str,
    pub(crate) control_constructor: Box<dyn DialogueSelectable<T>>,
    pub(crate) cell: Option<&'cell mut Option<T>>,
}

impl<'a, 'control, T: Clone + PartialEq> Dialogue<'a, 'control, T> {
    pub fn show(mut self, ctx: &Context) -> Response {
        egui::containers::Window::new(self.title)
            .title_bar(false)
            .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
            .resizable(false)
            .show(ctx, |ui| {
                let galley = ui.painter().layout_no_wrap(
                    self.text.to_owned(),
                    FontId::new(28.0, FontFamily::Proportional),
                    Color32::WHITE,
                );
                ui.set_height(galley.rect.height() * 3.0);

                ui.label(egui::RichText::new(self.title).size(28.0).strong());

                ui.label(self.text);
                ui.horizontal(|ui| {
                    let layout = self.control_constructor.layout();
                    let mut split = layout.split(|x| *x == None);
                    for v in split.next().unwrap() {
                        let v = v.clone();
                        if let Some((v, txt)) = v {
                            if ui.button(egui::RichText::new(txt).size(28.0)).clicked() {
                                if let Some(cell) = self.cell.as_mut() {
                                    **cell = Some(v)
                                }
                            }
                        }
                    }
                    // rtl layout
                    ui.with_layout(Layout::right_to_left(), |ui| {
                        if let Some(split) = split.next() {
                            for v in split.iter().rev() {
                                let v = v.clone();
                                if let Some((v, txt)) = v {
                                    if ui.button(egui::RichText::new(txt).size(28.0)).clicked() {
                                        if let Some(cell) = self.cell.as_mut() {
                                            **cell = Some(v)
                                        }
                                    }
                                }
                            }
                        }
                    });
                });
            })
            .unwrap()
            .response
    }
}
