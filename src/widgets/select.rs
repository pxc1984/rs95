use egui::{
    Popup, PopupCloseBehavior, Rect, RectAlign, Response, Role, Sense, TextStyle, TextWrapMode, Ui,
    Widget, WidgetInfo, WidgetText, pos2, vec2,
};

use crate::Theme;
use crate::paint::{self, Bevel, icons};

/// A drop-down list (React95 `Select`, a Win95 combo box).
///
/// ```no_run
/// # egui::__run_test_ui(|ui| {
/// # let mut fruit = 0;
/// ui.add(
///     rs95::Select::new("fruit", &mut fruit)
///         .option(0, "Apple")
///         .option(1, "Banana"),
/// );
/// # });
/// ```
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct Select<'a, T: PartialEq + Clone> {
    id_salt: egui::IdSalt,
    value: &'a mut T,
    options: Vec<(T, WidgetText)>,
    width: Option<f32>,
    max_height: f32,
}

impl<'a, T: PartialEq + Clone> Select<'a, T> {
    pub fn new(id_salt: impl egui::AsIdSalt, value: &'a mut T) -> Self {
        Self {
            id_salt: egui::IdSalt::new(id_salt),
            value,
            options: Vec::new(),
            width: None,
            max_height: 200.0,
        }
    }

    pub fn option(mut self, value: T, text: impl Into<WidgetText>) -> Self {
        self.options.push((value, text.into()));
        self
    }

    pub fn options(
        mut self,
        options: impl IntoIterator<Item = (T, impl Into<WidgetText>)>,
    ) -> Self {
        self.options
            .extend(options.into_iter().map(|(v, t)| (v, t.into())));
        self
    }

    /// Total width. Defaults to fit the longest option.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Maximum height of the drop-down list.
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = max_height;
        self
    }
}

const ARROW_W: f32 = 17.0;
const ITEM_H: f32 = 20.0;

impl<T: PartialEq + Clone> Widget for Select<'_, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = Theme::get(ui.ctx());
        let galleys: Vec<_> = self
            .options
            .iter()
            .map(|(_, text)| {
                text.clone().into_galley(
                    ui,
                    Some(TextWrapMode::Extend),
                    f32::INFINITY,
                    TextStyle::Body,
                )
            })
            .collect();
        let selected = self.options.iter().position(|(v, _)| v == self.value);
        let text_w = galleys.iter().map(|g| g.size().x).fold(0.0, f32::max);
        let width = self.width.unwrap_or(text_w + ARROW_W + 16.0).round();
        let height = ui.spacing().interact_size.y.max(23.0);

        let id = ui.make_persistent_id(self.id_salt);
        let (rect, mut response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
        let enabled = ui.is_enabled();
        let label = selected
            .map(|i| galleys[i].text().to_owned())
            .unwrap_or_default();
        response.widget_info(|| WidgetInfo::labeled(Role::ComboBox, enabled, &label));

        let popup_id = id.with("popup");
        let open = Popup::is_id_open(ui.ctx(), popup_id);

        // keyboard: up/down change the value while focused
        if response.has_focus() && !self.options.is_empty() {
            let (up, down) = ui.input(|i| {
                (
                    i.key_pressed(egui::Key::ArrowUp),
                    i.key_pressed(egui::Key::ArrowDown),
                )
            });
            let cur = selected.unwrap_or(0) as isize;
            let next =
                (cur + down as isize - up as isize).clamp(0, self.options.len() as isize - 1);
            if (up || down) && Some(next as usize) != selected {
                *self.value = self.options[next as usize].0.clone();
                response.mark_changed();
            }
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            painter.rect_filled(rect, 0.0, if enabled { t.canvas } else { t.material });
            paint::bevel(painter, rect, Bevel::Field, &t);
            let inner = rect.shrink(2.0);

            let arrow = Rect::from_min_max(pos2(inner.max.x - ARROW_W, inner.min.y), inner.max);
            let pressed = (response.is_pointer_button_down_on() && response.hovered()) || open;
            painter.rect_filled(arrow, 0.0, t.material);
            paint::bevel(
                painter,
                arrow,
                if pressed {
                    Bevel::ButtonPressed
                } else {
                    Bevel::Button
                },
                &t,
            );
            let off = if pressed {
                vec2(1.0, 1.0)
            } else {
                vec2(0.0, 0.0)
            };
            let arrow_color = if enabled {
                t.material_text
            } else {
                t.material_text_disabled
            };
            paint::icon(
                painter,
                arrow.translate(off),
                icons::ARROW_DOWN,
                arrow_color,
            );

            if let Some(i) = selected {
                let text_area =
                    Rect::from_min_max(inner.min, pos2(arrow.min.x, inner.max.y)).shrink(1.0);
                let g = galleys[i].clone();
                let pos = pos2(
                    text_area.min.x + 3.0,
                    text_area.center().y - g.size().y / 2.0,
                );
                if response.has_focus() && enabled {
                    painter.rect_filled(text_area, 0.0, t.hover_background);
                    paint::focus_rect(painter, text_area, t.canvas_text_invert);
                    painter.galley_with_override_text_color(pos.round(), g, t.canvas_text_invert);
                } else {
                    let color = if enabled {
                        t.canvas_text
                    } else {
                        t.canvas_text_disabled
                    };
                    painter.galley_with_override_text_color(pos.round(), g, color);
                }
            }
        }

        let frame = egui::Frame::NONE
            .fill(t.canvas)
            .stroke(egui::Stroke::new(1.0, t.border_darkest))
            .inner_margin(1);
        let mut changed = false;
        Popup::new(
            popup_id,
            ui.ctx().clone(),
            egui::PopupAnchor::from(rect),
            ui.layer_id(),
        )
        .open_memory(response.clicked().then_some(egui::SetOpenCommand::Toggle))
        .close_behavior(PopupCloseBehavior::CloseOnClick)
        .align(RectAlign::BOTTOM_START)
        .gap(0.0)
        .width(width)
        .frame(frame)
        .show(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            egui::ScrollArea::vertical()
                .max_height(self.max_height)
                .show(ui, |ui| {
                    let any_hovered = ui.rect_contains_pointer(ui.max_rect());
                    for (i, ((value, _), g)) in self.options.iter().zip(&galleys).enumerate() {
                        let (r, resp) = ui.allocate_exact_size(
                            vec2(ui.available_width(), ITEM_H),
                            Sense::click(),
                        );
                        let highlight = resp.hovered() || (!any_hovered && Some(i) == selected);
                        let painter = ui.painter();
                        if highlight {
                            painter.rect_filled(r, 0.0, t.hover_background);
                        }
                        let color = if highlight {
                            t.canvas_text_invert
                        } else {
                            t.canvas_text
                        };
                        let pos = pos2(r.min.x + 3.0, r.center().y - g.size().y / 2.0);
                        painter.galley_with_override_text_color(pos.round(), g.clone(), color);
                        if resp.clicked() && value != self.value {
                            *self.value = value.clone();
                            changed = true;
                        }
                    }
                });
        });
        if changed {
            response.mark_changed();
        }
        response
    }
}
