use std::ops::RangeInclusive;

use egui::{
    Key, Rect, Response, Sense, Ui, Widget, WidgetInfo, emath::Numeric, pos2, remap_clamp, vec2,
};

use crate::Theme;
use crate::paint::{self, Bevel};

/// A Windows 95 trackbar.
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct Slider<'a, N: Numeric> {
    value: &'a mut N,
    range: RangeInclusive<N>,
    step: Option<f64>,
    width: Option<f32>,
    marks: usize,
}

impl<'a, N: Numeric> Slider<'a, N> {
    pub fn new(value: &'a mut N, range: RangeInclusive<N>) -> Self {
        Self {
            value,
            range,
            step: N::INTEGRAL.then_some(1.0),
            width: None,
            marks: 0,
        }
    }

    /// Snap the value to multiples of `step`.
    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Draw `n` evenly spaced tick marks below the track (`n >= 2`).
    pub fn marks(mut self, n: usize) -> Self {
        self.marks = n;
        self
    }
}

const THUMB: egui::Vec2 = vec2(11.0, 21.0);

impl<N: Numeric> Widget for Slider<'_, N> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = Theme::get(ui.ctx());
        let width = self.width.unwrap_or(ui.spacing().slider_width.max(150.0));
        let height = THUMB.y + if self.marks >= 2 { 8.0 } else { 2.0 };
        let (rect, mut response) =
            ui.allocate_exact_size(vec2(width, height), Sense::click_and_drag());

        let (min, max) = (self.range.start().to_f64(), self.range.end().to_f64());
        let x_range = (rect.min.x + THUMB.x / 2.0)..=(rect.max.x - THUMB.x / 2.0);
        let snap = |v: f64| {
            let v = match self.step {
                Some(s) if s > 0.0 => min + ((v - min) / s).round() * s,
                _ => v,
            };
            v.clamp(min.min(max), max.max(min))
        };

        let mut v = self.value.to_f64();
        if let Some(p) = response.interact_pointer_pos() {
            v = snap(remap_clamp(
                p.x as f64,
                (*x_range.start() as f64)..=(*x_range.end() as f64),
                min..=max,
            ));
        }
        if response.has_focus() {
            let step = self.step.unwrap_or((max - min) / 20.0);
            let (l, r) = ui.input(|i| {
                (
                    i.key_pressed(Key::ArrowLeft) || i.key_pressed(Key::ArrowDown),
                    i.key_pressed(Key::ArrowRight) || i.key_pressed(Key::ArrowUp),
                )
            });
            if l {
                v = snap(v - step);
            }
            if r {
                v = snap(v + step);
            }
        }
        let new = N::from_f64(v);
        if new != *self.value {
            *self.value = new;
            response.mark_changed();
        }
        let enabled = ui.is_enabled();
        response.widget_info(|| WidgetInfo::slider(enabled, v, ""));

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let cy = (rect.min.y + THUMB.y / 2.0).round();
            let track = Rect::from_min_max(pos2(rect.min.x, cy - 2.0), pos2(rect.max.x, cy + 2.0));
            painter.rect_filled(track, 0.0, t.material);
            paint::bevel(painter, track, Bevel::Field, &t);

            if self.marks >= 2 {
                for i in 0..self.marks {
                    let x = egui::lerp(x_range.clone(), i as f32 / (self.marks - 1) as f32).round();
                    paint::px_rect(
                        painter,
                        x,
                        rect.max.y - 5.0,
                        x + 1.0,
                        rect.max.y,
                        t.material_text,
                    );
                }
            }

            let x = remap_clamp(
                v,
                min..=max,
                (*x_range.start() as f64)..=(*x_range.end() as f64),
            ) as f32;
            let thumb = Rect::from_center_size(pos2(x.round(), rect.min.y + THUMB.y / 2.0), THUMB);
            let thumb = Rect::from_min_size(thumb.min.round(), THUMB);
            painter.rect_filled(thumb, 0.0, t.material);
            paint::bevel(painter, thumb, Bevel::Button, &t);
            if response.has_focus() {
                paint::focus_rect(painter, rect.expand2(vec2(2.0, 0.0)), t.material_text);
            }
        }
        response
    }
}
