use egui::{Align2, Rect, Sense, TextStyle, Ui, pos2, vec2};

use crate::Theme;
use crate::paint::{self, Bevel};

/// What was clicked in a [`Table`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TableResponse {
    /// Index of a clicked column header (use it to sort).
    pub header_clicked: Option<usize>,
    /// Index of a clicked row.
    pub row_clicked: Option<usize>,
}

/// A simple data table with button-like column headers (React95 `Table`).
///
/// ```no_run
/// # egui::__run_test_ui(|ui| {
/// # let mut selected = None;
/// rs95::Table::new(&["Name", "Size"])
///     .widths(&[120.0, 60.0])
///     .selected(&mut selected)
///     .show(ui, &[vec!["autoexec.bat".into(), "1 KB".into()]]);
/// # });
/// ```
pub struct Table<'a> {
    headers: &'a [&'a str],
    widths: Option<&'a [f32]>,
    selected: Option<&'a mut Option<usize>>,
    row_height: f32,
}

impl<'a> Table<'a> {
    pub fn new(headers: &'a [&'a str]) -> Self {
        Self {
            headers,
            widths: None,
            selected: None,
            row_height: 20.0,
        }
    }

    /// Column widths in points. Missing columns share the remaining width.
    pub fn widths(mut self, widths: &'a [f32]) -> Self {
        self.widths = Some(widths);
        self
    }

    /// Track the selected row; clicking a row selects it.
    pub fn selected(mut self, selected: &'a mut Option<usize>) -> Self {
        self.selected = Some(selected);
        self
    }

    pub fn row_height(mut self, h: f32) -> Self {
        self.row_height = h;
        self
    }

    pub fn show(self, ui: &mut Ui, rows: &[Vec<String>]) -> TableResponse {
        let t = Theme::get(ui.ctx());
        let n = self.headers.len().max(1);
        let total = ui.available_width();
        let given: f32 = self.widths.map_or(0.0, |w| w.iter().take(n).sum());
        let n_given = self.widths.map_or(0, |w| w.len().min(n));
        let rest = if n > n_given {
            ((total - 4.0 - given) / (n - n_given) as f32).max(40.0)
        } else {
            0.0
        };
        let widths: Vec<f32> = (0..n)
            .map(|i| {
                self.widths
                    .and_then(|w| w.get(i).copied())
                    .unwrap_or(rest)
                    .round()
            })
            .collect();
        let width: f32 = widths.iter().sum::<f32>() + 4.0;
        let header_h = 22.0;
        let height = header_h + rows.len() as f32 * self.row_height + 4.0;

        let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let mut out = TableResponse::default();
        if !ui.is_rect_visible(rect) {
            return out;
        }
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, t.canvas);
        paint::bevel(painter, rect, Bevel::Field, &t);
        let inner = rect.shrink(2.0);
        let font = TextStyle::Body.resolve(ui.style());
        let base = ui.scope_id().with("rs95_table");

        // header
        let mut x = inner.min.x;
        for (i, w) in widths.iter().enumerate() {
            let r = Rect::from_min_size(pos2(x, inner.min.y), vec2(*w, header_h));
            let resp = ui.interact(r, base.with(("h", i)), Sense::click());
            let pressed = resp.is_pointer_button_down_on() && resp.hovered();
            painter.rect_filled(r, 0.0, t.material);
            paint::bevel(
                painter,
                r,
                if pressed {
                    Bevel::ButtonPressed
                } else {
                    Bevel::Button
                },
                &t,
            );
            if let Some(h) = self.headers.get(i) {
                let off = if pressed { 1.0 } else { 0.0 };
                painter.with_clip_rect(r.shrink(2.0)).text(
                    pos2(r.min.x + 6.0 + off, r.center().y + off).round(),
                    Align2::LEFT_CENTER,
                    *h,
                    font.clone(),
                    t.material_text,
                );
            }
            if resp.clicked() {
                out.header_clicked = Some(i);
            }
            x += w;
        }

        // body
        let mut selected = self.selected;
        let mut y = inner.min.y + header_h;
        for (ri, row) in rows.iter().enumerate() {
            let r = Rect::from_min_size(pos2(inner.min.x, y), vec2(width - 4.0, self.row_height));
            let resp = ui.interact(r, base.with(("r", ri)), Sense::click());
            let is_sel = selected.as_deref().is_some_and(|s| *s == Some(ri));
            let highlight = is_sel || resp.hovered();
            if highlight {
                painter.rect_filled(r, 0.0, t.hover_background);
            }
            let color = if highlight {
                t.canvas_text_invert
            } else {
                t.canvas_text
            };
            let mut x = inner.min.x;
            for (ci, w) in widths.iter().enumerate() {
                if let Some(cell) = row.get(ci) {
                    let cell_rect = Rect::from_min_size(pos2(x, y), vec2(*w, self.row_height));
                    painter.with_clip_rect(cell_rect.shrink(1.0)).text(
                        pos2(x + 6.0, cell_rect.center().y).round(),
                        Align2::LEFT_CENTER,
                        cell,
                        font.clone(),
                        color,
                    );
                }
                x += w;
            }
            if resp.clicked() {
                out.row_clicked = Some(ri);
                if let Some(s) = selected.as_deref_mut() {
                    *s = Some(ri);
                }
            }
            y += self.row_height;
        }
        out
    }
}
