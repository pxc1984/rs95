use std::ops::RangeInclusive;

use egui::{Margin, Rect, Response, Sense, TextEdit, Ui, Widget, WidgetText, emath::Numeric, vec2};

use crate::Theme;
use crate::paint::{self, Bevel, icons};

/// A sunken white text field.
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct TextInput<'a> {
    text: &'a mut String,
    multiline: bool,
    rows: usize,
    width: Option<f32>,
    hint: Option<WidgetText>,
    password: bool,
    id: Option<egui::Id>,
}

impl<'a> TextInput<'a> {
    pub fn singleline(text: &'a mut String) -> Self {
        Self {
            text,
            multiline: false,
            rows: 1,
            width: None,
            hint: None,
            password: false,
            id: None,
        }
    }

    pub fn multiline(text: &'a mut String) -> Self {
        Self {
            multiline: true,
            rows: 4,
            ..Self::singleline(text)
        }
    }

    /// Number of visible rows of a multiline field.
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows;
        self
    }

    /// Total width including the border.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn hint_text(mut self, hint: impl Into<WidgetText>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    pub fn id(mut self, id: egui::Id) -> Self {
        self.id = Some(id);
        self
    }
}

const FIELD_MARGIN: Margin = Margin {
    left: 5,
    right: 5,
    top: 4,
    bottom: 3,
};

/// Wrap a widget into a sunken field. Returns the inner response and the field rect.
pub(crate) fn field<R>(
    ui: &mut Ui,
    fill: egui::Color32,
    margin: Margin,
    add: impl FnOnce(&mut Ui) -> R,
) -> (R, Rect) {
    let t = Theme::get(ui.ctx());
    let r = egui::Frame::NONE
        .fill(fill)
        .inner_margin(margin)
        .show(ui, add);
    paint::bevel(ui.painter(), r.response.rect, Bevel::Field, &t);
    (r.inner, r.response.rect)
}

impl Widget for TextInput<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = Theme::get(ui.ctx());
        let width = self.width.unwrap_or(ui.spacing().text_edit_width);
        let inner_width = width - (FIELD_MARGIN.left + FIELD_MARGIN.right) as f32;
        let fill = if ui.is_enabled() {
            t.canvas
        } else {
            t.material
        };

        let mut edit = if self.multiline {
            TextEdit::multiline(self.text).desired_rows(self.rows)
        } else {
            TextEdit::singleline(self.text)
        };
        edit = edit
            .frame(egui::Frame::NONE)
            .margin(Margin::ZERO)
            .desired_width(inner_width)
            .text_color(t.canvas_text)
            .password(self.password);
        if let Some(hint) = self.hint {
            edit = edit.hint_text(hint);
        }
        if let Some(id) = self.id {
            edit = edit.id(id);
        }
        field(ui, fill, FIELD_MARGIN, |ui| ui.add(edit)).0
    }
}

/// A numeric text field with spin buttons (React95 `NumberInput`).
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct NumberInput<'a, N: Numeric> {
    value: &'a mut N,
    range: RangeInclusive<N>,
    step: f64,
    width: f32,
}

impl<'a, N: Numeric> NumberInput<'a, N> {
    pub fn new(value: &'a mut N) -> Self {
        Self {
            value,
            range: N::MIN..=N::MAX,
            step: 1.0,
            width: 80.0,
        }
    }

    pub fn range(mut self, range: RangeInclusive<N>) -> Self {
        self.range = range;
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    /// Total width including the spin buttons.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

fn format_number<N: Numeric>(v: N) -> String {
    if N::INTEGRAL {
        format!("{}", v.to_f64() as i64)
    } else {
        format!("{}", v.to_f64())
    }
}

impl<N: Numeric> Widget for NumberInput<'_, N> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = Theme::get(ui.ctx());
        let id = ui.next_auto_id();
        ui.skip_ahead_auto_ids(1);
        let text_id = id.with("text");
        let (min, max) = (self.range.start().to_f64(), self.range.end().to_f64());
        let clamp = |v: f64| N::from_f64(v.clamp(min, max));

        let focused = ui.memory(|m| m.has_focus(text_id));
        let mut buf: String = if focused {
            ui.data(|d| d.get_temp(id))
                .unwrap_or_else(|| format_number(*self.value))
        } else {
            format_number(*self.value)
        };

        let fill = if ui.is_enabled() {
            t.canvas
        } else {
            t.material
        };
        let spin_w = 16.0;
        let inner_width = self.width - spin_w - (FIELD_MARGIN.left + FIELD_MARGIN.right) as f32;

        let mut changed = false;
        let (mut response, field_rect) = field(
            ui,
            fill,
            Margin {
                right: FIELD_MARGIN.right + spin_w as i8,
                ..FIELD_MARGIN
            },
            |ui| {
                ui.add(
                    TextEdit::singleline(&mut buf)
                        .id(text_id)
                        .frame(egui::Frame::NONE)
                        .margin(Margin::ZERO)
                        .desired_width(inner_width)
                        .text_color(t.canvas_text),
                )
            },
        );

        if response.changed() {
            if let Ok(v) = buf.trim().parse::<f64>() {
                let v = clamp(v);
                if v != *self.value {
                    *self.value = v;
                    changed = true;
                }
            }
        }
        if response.lost_focus() {
            buf = format_number(*self.value);
        }
        ui.data_mut(|d| d.insert_temp(id, buf));

        // spin buttons live inside the field's right border
        let inner = field_rect.shrink(2.0);
        let spin = Rect::from_min_max(egui::pos2(inner.max.x - spin_w, inner.min.y), inner.max);
        let half = spin.height() / 2.0;
        let up = Rect::from_min_size(spin.min, vec2(spin_w, half.floor()));
        let down = Rect::from_min_max(egui::pos2(spin.min.x, up.max.y), spin.max);
        let painter = ui.painter().clone();
        for (rect, rows, delta, salt) in [
            (up, icons::SMALL_ARROW_UP, self.step, "up"),
            (down, icons::SMALL_ARROW_DOWN, -self.step, "down"),
        ] {
            let r = ui.interact(rect, id.with(salt), Sense::click());
            let pressed = r.is_pointer_button_down_on() && r.hovered();
            painter.rect_filled(rect, 0.0, t.material);
            paint::bevel(
                &painter,
                rect,
                if pressed {
                    Bevel::ButtonPressed
                } else {
                    Bevel::Button
                },
                &t,
            );
            let color = if ui.is_enabled() {
                t.material_text
            } else {
                t.material_text_disabled
            };
            let off = if pressed {
                vec2(1.0, 1.0)
            } else {
                vec2(0.0, 0.0)
            };
            paint::icon(&painter, rect.translate(off), rows, color);
            if r.clicked() {
                let v = clamp(self.value.to_f64() + delta);
                if v != *self.value {
                    *self.value = v;
                    changed = true;
                }
            }
            response = response.union(r);
        }

        if changed {
            response.mark_changed();
        }
        response
    }
}
