use egui::{
    Rect, Response, Role, Sense, TextStyle, TextWrapMode, Ui, Vec2, Widget, WidgetInfo, WidgetText,
    pos2, vec2,
};

use crate::Theme;
use crate::paint::{self, Bevel, icons};

const BOX: f32 = 13.0;

/// Lay out `[indicator] label`, returning (indicator rect, label pos, galley, response).
fn layout_toggle(
    ui: &mut Ui,
    indicator: Vec2,
    text: WidgetText,
) -> (Rect, Option<(Rect, std::sync::Arc<egui::Galley>)>, Response) {
    let spacing = ui.spacing().icon_spacing;
    let galley = (!text.is_empty()).then(|| {
        text.into_galley(
            ui,
            Some(TextWrapMode::Extend),
            f32::INFINITY,
            TextStyle::Body,
        )
    });
    let text_size = galley
        .as_ref()
        .map_or(Vec2::ZERO, |g| g.size() + vec2(4.0, 2.0));
    let size = vec2(
        indicator.x
            + if galley.is_some() {
                spacing + text_size.x
            } else {
                0.0
            },
        indicator
            .y
            .max(text_size.y)
            .max(ui.spacing().interact_size.y),
    );
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let ind = Rect::from_min_size(
        pos2(rect.min.x, (rect.center().y - indicator.y / 2.0).round()),
        indicator,
    );
    let label = galley.map(|g| {
        let r = Rect::from_min_size(
            pos2(
                ind.max.x + spacing,
                (rect.center().y - text_size.y / 2.0).round(),
            ),
            text_size,
        );
        (r, g)
    });
    (ind, label, response)
}

fn paint_label(
    ui: &Ui,
    label: Option<(Rect, std::sync::Arc<egui::Galley>)>,
    response: &Response,
    t: &Theme,
) {
    if let Some((r, galley)) = label {
        let painter = ui.painter();
        paint::text(
            painter,
            r.min + vec2(2.0, 1.0),
            galley,
            t.material_text,
            ui.is_enabled(),
            t,
        );
        if response.has_focus() {
            paint::focus_rect(painter, r, t.material_text);
        }
    }
}

/// A Windows 95 checkbox.
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct Checkbox<'a> {
    checked: &'a mut bool,
    text: WidgetText,
    indeterminate: bool,
}

impl<'a> Checkbox<'a> {
    pub fn new(checked: &'a mut bool, text: impl Into<WidgetText>) -> Self {
        Self {
            checked,
            text: text.into(),
            indeterminate: false,
        }
    }

    /// Show the "mixed" state (a gray check on a dithered box).
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }
}

impl Widget for Checkbox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = Theme::get(ui.ctx());
        let text = self.text.text().to_owned();
        let (ind, label, mut response) = layout_toggle(ui, Vec2::splat(BOX), self.text);
        if response.clicked() {
            *self.checked = !*self.checked;
            response.mark_changed();
        }
        let enabled = ui.is_enabled();
        response
            .widget_info(|| WidgetInfo::selected(Role::CheckBox, enabled, *self.checked, &text));

        if ui.is_rect_visible(response.rect) {
            let painter = ui.painter();
            let held = response.is_pointer_button_down_on() && response.hovered();
            if self.indeterminate {
                paint::dither(painter, ind, t.material, t.canvas);
            } else {
                let fill = if enabled && !held {
                    t.canvas
                } else {
                    t.material
                };
                painter.rect_filled(ind, 0.0, fill);
            }
            paint::bevel(painter, ind, Bevel::Field, &t);
            if *self.checked || self.indeterminate {
                let color = if enabled && !self.indeterminate {
                    t.checkmark
                } else {
                    t.checkmark_disabled
                };
                paint::icon(painter, ind, icons::CHECK, color);
            }
            paint_label(ui, label, &response, &t);
        }
        response
    }
}

/// A Windows 95 radio button. See also [`radio_value`].
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct Radio {
    checked: bool,
    text: WidgetText,
}

impl Radio {
    pub fn new(checked: bool, text: impl Into<WidgetText>) -> Self {
        Self {
            checked,
            text: text.into(),
        }
    }
}

impl Widget for Radio {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = Theme::get(ui.ctx());
        let text = self.text.text().to_owned();
        let (ind, label, response) = layout_toggle(ui, Vec2::splat(12.0), self.text);
        let enabled = ui.is_enabled();
        response
            .widget_info(|| WidgetInfo::selected(Role::RadioButton, enabled, self.checked, &text));

        if ui.is_rect_visible(response.rect) {
            let painter = ui.painter();
            let held = response.is_pointer_button_down_on() && response.hovered();
            let inside = if enabled && !held {
                t.canvas
            } else {
                t.material
            };
            paint::pixmap(painter, ind.min, icons::RADIO, |c| match c {
                'd' => Some(t.border_dark),
                'k' => Some(t.border_darkest),
                'l' => Some(t.border_light),
                'w' => Some(t.border_lightest),
                'c' => Some(inside),
                _ => None,
            });
            if self.checked {
                let color = if enabled {
                    t.checkmark
                } else {
                    t.checkmark_disabled
                };
                paint::pixmap(painter, ind.min + vec2(4.0, 4.0), icons::RADIO_DOT, |c| {
                    (c == '#').then_some(color)
                });
            }
            paint_label(ui, label, &response, &t);
        }
        response
    }
}

/// A radio button that sets `*current = value` when clicked.
pub fn radio_value<T: PartialEq>(
    ui: &mut Ui,
    current: &mut T,
    value: T,
    text: impl Into<WidgetText>,
) -> Response {
    let mut response = ui.add(Radio::new(*current == value, text));
    if response.clicked() && *current != value {
        *current = value;
        response.mark_changed();
    }
    response
}
