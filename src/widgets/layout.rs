use egui::{
    Color32, InnerResponse, Margin, Rect, Response, Sense, Shape, TextStyle, TextWrapMode, Ui,
    Widget, WidgetText, pos2, vec2,
};

use crate::Theme;
use crate::paint::{self, Bevel};

/// A container drawn with one of the Win95 bevels (React95 `Frame`).
///
/// ```no_run
/// # egui::__run_test_ui(|ui| {
/// rs95::Frame::field().show(ui, |ui| ui.label("inside a sunken field"));
/// # });
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    bevel: Option<Bevel>,
    fill: Option<Color32>,
    inner_margin: Margin,
}

impl Frame {
    /// A frame with the given bevel and the material color as fill.
    pub fn new(bevel: Bevel) -> Self {
        Self {
            bevel: Some(bevel),
            fill: None,
            inner_margin: Margin::same(4),
        }
    }

    /// Raised 2px window border.
    pub fn window() -> Self {
        Self::new(Bevel::Window)
    }

    /// Raised 2px button border.
    pub fn button() -> Self {
        Self::new(Bevel::Button)
    }

    /// Sunken white field.
    pub fn field() -> Self {
        Self::new(Bevel::Field).fill_canvas()
    }

    /// Thin sunken border, as in status bars.
    pub fn status() -> Self {
        Self::new(Bevel::Status).inner_margin(Margin::symmetric(4, 2))
    }

    /// Thin sunken border with material fill.
    pub fn well() -> Self {
        Self::new(Bevel::Status)
    }

    /// No border, just fill and margin.
    pub fn none() -> Self {
        Self {
            bevel: None,
            ..Self::new(Bevel::Thin)
        }
    }

    pub fn fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Use the theme's canvas (white) as fill.
    pub fn fill_canvas(mut self) -> Self {
        self.fill = Some(Color32::PLACEHOLDER);
        self
    }

    /// Margin inside the border.
    pub fn inner_margin(mut self, margin: impl Into<Margin>) -> Self {
        self.inner_margin = margin.into();
        self
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let t = Theme::get(ui.ctx());
        let border = self.bevel.map_or(0.0, Bevel::width) as i8;
        let bg = ui.painter().add(Shape::Noop);
        let r = egui::Frame::NONE
            .inner_margin(self.inner_margin + Margin::same(border))
            .show(ui, |ui| {
                ui.style_mut().visuals.override_text_color = None;
                add_contents(ui)
            });
        let rect = r.response.rect;
        let fill = match self.fill {
            Some(Color32::PLACEHOLDER) => t.canvas,
            Some(c) => c,
            None => t.material,
        };
        ui.painter().set(bg, Shape::rect_filled(rect, 0.0, fill));
        if let Some(b) = self.bevel {
            paint::bevel(ui.painter(), rect, b, &t);
        }
        r
    }
}

/// A labeled, etched frame around a group of controls (React95 `GroupBox`).
pub struct GroupBox {
    label: WidgetText,
}

impl GroupBox {
    pub fn new(label: impl Into<WidgetText>) -> Self {
        Self {
            label: label.into(),
        }
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let t = Theme::get(ui.ctx());
        let galley = self.label.into_galley(
            ui,
            Some(TextWrapMode::Extend),
            f32::INFINITY,
            TextStyle::Body,
        );
        let label_h = galley.size().y;
        let top = (label_h / 2.0).round();
        let r = egui::Frame::NONE
            .inner_margin(Margin {
                left: 10,
                right: 10,
                top: label_h as i8 + 4,
                bottom: 10,
            })
            .show(ui, |ui| {
                ui.set_min_width(galley.size().x + 16.0);
                add_contents(ui)
            });
        // the frame lives in the margin, so it can be painted after the contents
        let rect = r.response.rect;
        let painter = ui.painter();
        paint::etched(
            painter,
            Rect::from_min_max(pos2(rect.min.x, rect.min.y + top), rect.max),
            &t,
        );
        let label_pos = pos2(rect.min.x + 8.0, rect.min.y);
        painter.rect_filled(
            Rect::from_min_size(label_pos, galley.size() + vec2(4.0, 0.0)),
            0.0,
            t.material,
        );
        paint::text(
            painter,
            label_pos + vec2(2.0, 0.0),
            galley,
            t.material_text,
            ui.is_enabled(),
            &t,
        );
        r
    }
}

/// An etched separator line (React95 `Separator`).
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct Separator {
    vertical: bool,
}

impl Separator {
    pub fn horizontal() -> Self {
        Self { vertical: false }
    }

    pub fn vertical() -> Self {
        Self { vertical: true }
    }
}

impl Widget for Separator {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = Theme::get(ui.ctx());
        let size = if self.vertical {
            vec2(2.0, ui.available_height().min(ui.spacing().interact_size.y))
        } else {
            vec2(ui.available_width(), 2.0)
        };
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        if self.vertical {
            paint::etched_vline(ui.painter(), rect.min.round(), rect.height(), &t);
        } else {
            paint::etched_hline(ui.painter(), rect.min.round(), rect.width(), &t);
        }
        response
    }
}

/// A full width raised bar (React95 `AppBar`). Put a [`Toolbar`] or buttons inside.
pub struct AppBar;

impl AppBar {
    pub fn show<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        Frame::window()
            .inner_margin(Margin::same(2))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(add_contents).inner
            })
    }
}

/// A horizontal row of tightly spaced controls (React95 `Toolbar`).
pub struct Toolbar;

impl Toolbar {
    pub fn show<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            add_contents(ui)
        })
    }
}

/// A sunken status bar pane with text.
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct StatusField {
    text: WidgetText,
    width: Option<f32>,
}

impl StatusField {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            width: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl Widget for StatusField {
    fn ui(self, ui: &mut Ui) -> Response {
        let width = self.width;
        Frame::status()
            .show(ui, |ui| {
                if let Some(w) = width {
                    ui.set_width(w - 10.0);
                }
                ui.add(egui::Label::new(self.text).truncate());
            })
            .response
    }
}

/// Show a pale yellow Win95 tooltip when `response` is hovered.
pub fn tooltip(response: Response, text: impl Into<WidgetText>) -> Response {
    let text = text.into();
    let t = Theme::get(&response.ctx);
    let mut tip = egui::Tooltip::for_enabled(&response);
    tip.popup = tip.popup.frame(
        egui::Frame::NONE
            .fill(t.tooltip)
            .stroke(egui::Stroke::new(1.0, t.border_darkest))
            .inner_margin(Margin::symmetric(4, 2)),
    );
    tip.show(|ui| {
        ui.style_mut().visuals.override_text_color = Some(t.canvas_text);
        ui.add(egui::Label::new(text).extend());
    });
    response
}

/// Paint the teal desktop background over `ui`'s whole clip rect.
pub fn desktop(ui: &Ui) {
    let t = Theme::get(ui.ctx());
    ui.painter()
        .rect_filled(ui.clip_rect(), 0.0, t.desktop_background);
}
