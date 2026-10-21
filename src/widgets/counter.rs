use egui::{Color32, Painter, Rect, Response, Sense, Ui, Widget, pos2, vec2};

use crate::paint::{self, Bevel};
use crate::Theme;

/// A red seven-segment display, like the Minesweeper counters (React95 `Counter`).
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct Counter {
    value: i64,
    digits: usize,
    scale: f32,
}

impl Counter {
    pub fn new(value: i64) -> Self {
        Self {
            value,
            digits: 3,
            scale: 1.0,
        }
    }

    /// Number of digits; the value is clamped to fit.
    pub fn digits(mut self, digits: usize) -> Self {
        self.digits = digits.max(1);
        self
    }

    /// Size multiplier (whole numbers keep it crisp).
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale.max(1.0);
        self
    }
}

const ON: Color32 = Color32::from_rgb(0xff, 0x00, 0x00);
const OFF: Color32 = Color32::from_rgb(0x40, 0x00, 0x00);
const CELL: egui::Vec2 = vec2(13.0, 23.0);

// segments: a b c d e f g
const DIGITS: [[bool; 7]; 10] = [
    [true, true, true, true, true, true, false],
    [false, true, true, false, false, false, false],
    [true, true, false, true, true, false, true],
    [true, true, true, true, false, false, true],
    [false, true, true, false, false, true, true],
    [true, false, true, true, false, true, true],
    [true, false, true, true, true, true, true],
    [true, true, true, false, false, false, false],
    [true, true, true, true, true, true, true],
    [true, true, true, true, false, true, true],
];
const MINUS: [bool; 7] = [false, false, false, false, false, false, true];

impl Widget for Counter {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = Theme::get(ui.ctx());
        let s = self.scale;
        let size = vec2(CELL.x * self.digits as f32, CELL.y) * s + vec2(2.0, 2.0);
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, Color32::BLACK);
        paint::bevel(painter, rect, Bevel::Status, &t);

        let max = 10_i64.saturating_pow(self.digits as u32) - 1;
        let min = -(10_i64.saturating_pow(self.digits as u32 - 1) - 1);
        let v = self.value.clamp(min, max);
        let text = format!("{:0>width$}", v.unsigned_abs(), width = self.digits);
        let mut cells: Vec<[bool; 7]> = text
            .bytes()
            .map(|b| DIGITS[(b - b'0') as usize])
            .collect();
        if v < 0 {
            cells[0] = MINUS;
        }
        for (i, segs) in cells.iter().enumerate() {
            let origin = rect.min + vec2(1.0 + i as f32 * CELL.x * s, 1.0);
            digit(painter, Rect::from_min_size(origin, CELL * s), segs, s);
        }
        response
    }
}

fn digit(painter: &Painter, cell: Rect, segs: &[bool; 7], s: f32) {
    // segment geometry in a 13x23 cell, thickness 2, tapered ends
    let x0 = cell.min.x + 1.0 * s;
    let x1 = cell.max.x - 1.0 * s;
    let y0 = cell.min.y + 1.0 * s;
    let ym = cell.center().y.round();
    let y1 = cell.max.y - 1.0 * s;
    let th = 2.0 * s;
    let c = |on: bool| if on { ON } else { OFF };
    let h = |y: f32, top: bool, on: bool| {
        for k in 0..2 {
            let k = k as f32 * s;
            let inset = if top { k } else { s - k };
            let yy = y + k;
            paint::px_rect(painter, x0 + s + inset, yy, x1 - s - inset, yy + s, c(on));
        }
    };
    let v = |x: f32, ya: f32, yb: f32, left: bool, on: bool| {
        for k in 0..2 {
            let k = k as f32 * s;
            let inset = if left { k } else { s - k };
            let xx = x + k;
            paint::px_rect(painter, xx, ya + s + inset, xx + s, yb - s - inset, c(on));
        }
    };
    h(y0, true, segs[0]); // a
    v(x1 - th, y0, ym, false, segs[1]); // b
    v(x1 - th, ym, y1, false, segs[2]); // c
    h(y1 - th, false, segs[3]); // d
    v(x0, ym, y1, true, segs[4]); // e
    v(x0, y0, ym, true, segs[5]); // f
    // g: centered, tapered on both ends
    let on = c(segs[6]);
    paint::px_rect(painter, x0 + 2.0 * s, ym - s, x1 - 2.0 * s, ym, on);
    paint::px_rect(painter, x0 + 2.0 * s, ym, x1 - 2.0 * s, ym + s, on);
    let _ = pos2;
}
