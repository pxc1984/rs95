use egui::{Align2, Rect, Response, Sense, TextStyle, Ui, Widget, pos2, vec2};

use crate::paint::{self, Bevel};
use crate::Theme;

/// Visual variant of a [`ProgressBar`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProgressVariant {
    /// Solid fill with a centered percentage (inverted over the filled part).
    #[default]
    Default,
    /// Discrete blue blocks, as in the Windows 95 file copy dialog.
    Tile,
}

/// A progress bar. `value` is a fraction in `0..=1`.
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct ProgressBar {
    value: f32,
    variant: ProgressVariant,
    width: Option<f32>,
    text: Option<String>,
    show_percentage: bool,
}

impl ProgressBar {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            variant: ProgressVariant::Default,
            width: None,
            text: None,
            show_percentage: true,
        }
    }

    pub fn variant(mut self, variant: ProgressVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Shorthand for `.variant(ProgressVariant::Tile)`.
    pub fn tile(self) -> Self {
        self.variant(ProgressVariant::Tile)
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Replace the percentage label.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }
}

impl Widget for ProgressBar {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = Theme::get(ui.ctx());
        let width = self.width.unwrap_or_else(|| ui.available_width()).round();
        let (rect, response) = ui.allocate_exact_size(vec2(width, 24.0), Sense::hover());
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let painter = ui.painter();
        let inner = rect.shrink(2.0);

        match self.variant {
            ProgressVariant::Default => {
                painter.rect_filled(rect, 0.0, t.canvas);
                paint::bevel(painter, rect, Bevel::Field, &t);
                let filled = Rect::from_min_max(
                    inner.min,
                    pos2((inner.min.x + inner.width() * self.value).round(), inner.max.y),
                );
                painter.rect_filled(filled, 0.0, t.progress);

                let label = self.text.clone().or_else(|| {
                    self.show_percentage
                        .then(|| format!("{}%", (self.value * 100.0).round() as i32))
                });
                if let Some(label) = label {
                    let font = TextStyle::Body.resolve(ui.style());
                    let center = inner.center().round();
                    painter.with_clip_rect(inner).text(center, Align2::CENTER_CENTER, &label, font.clone(), t.canvas_text);
                    painter
                        .with_clip_rect(filled.intersect(inner))
                        .text(center, Align2::CENTER_CENTER, &label, font, t.canvas_text_invert);
                }
            }
            ProgressVariant::Tile => {
                painter.rect_filled(rect, 0.0, t.material);
                paint::bevel(painter, rect, Bevel::Field, &t);
                let area = inner.shrink(2.0);
                let (tile, gap) = (16.0, 2.0);
                let filled_w = area.width() * self.value;
                let n = ((filled_w + gap) / (tile + gap)).floor() as i32;
                let clip = painter.with_clip_rect(area);
                for i in 0..n {
                    let x = area.min.x + i as f32 * (tile + gap);
                    clip.rect_filled(
                        Rect::from_min_max(pos2(x, area.min.y), pos2(x + tile, area.max.y)),
                        0.0,
                        t.progress,
                    );
                }
            }
        }
        response
    }
}
