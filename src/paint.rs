//! Low level drawing primitives: bevels, pixel art, dithering, embossed text.
//!
//! Everything is drawn with axis aligned rectangles in whole points, so the
//! result stays crisp at any integer `pixels_per_point`.

use egui::{
    Color32, Context, Galley, Id, Painter, Pos2, Rect, TextureHandle, TextureOptions, Vec2,
    pos2, vec2,
};
use std::sync::Arc;

use crate::Theme;

/// The 3D border styles of Windows 95.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bevel {
    /// Raised 2px border of a button.
    Button,
    /// Sunken 2px border of a pressed button.
    ButtonPressed,
    /// Raised 2px border of a window, menu or popup.
    Window,
    /// Sunken 2px border of a text field, list or checkbox.
    Field,
    /// Sunken 1px border (status bar panes, wells).
    Status,
    /// Raised 1px border (toolbar buttons on hover, thin buttons).
    Thin,
    /// Sunken 1px border (pressed thin buttons).
    ThinPressed,
}

impl Bevel {
    /// Width of the border in points.
    pub fn width(self) -> f32 {
        match self {
            Self::Status | Self::Thin | Self::ThinPressed => 1.0,
            _ => 2.0,
        }
    }
}

/// Paint `bevel` along the inside edge of `rect`. The interior is not filled.
pub fn bevel(painter: &Painter, rect: Rect, bevel: Bevel, t: &Theme) {
    let (outer, inner) = match bevel {
        Bevel::Button => (
            (t.border_lightest, t.border_darkest),
            Some((t.border_light, t.border_dark)),
        ),
        Bevel::ButtonPressed => (
            (t.border_darkest, t.border_lightest),
            Some((t.border_dark, t.border_light)),
        ),
        Bevel::Window => (
            (t.border_light, t.border_darkest),
            Some((t.border_lightest, t.border_dark)),
        ),
        Bevel::Field => (
            (t.border_dark, t.border_lightest),
            Some((t.border_darkest, t.border_light)),
        ),
        Bevel::Status | Bevel::ThinPressed => ((t.border_dark, t.border_lightest), None),
        Bevel::Thin => ((t.border_lightest, t.border_dark), None),
    };
    two_tone(painter, rect, outer.0, outer.1);
    if let Some((tl, br)) = inner {
        two_tone(painter, rect.shrink(1.0), tl, br);
    }
}

/// Fill `rect` with `fill` and paint `bevel` on top.
pub fn bevel_filled(painter: &Painter, rect: Rect, b: Bevel, fill: Color32, t: &Theme) {
    painter.rect_filled(rect, 0.0, fill);
    bevel(painter, rect, b, t);
}

/// A 1px frame: top and left edges in `tl`, bottom and right edges in `br`.
pub fn two_tone(painter: &Painter, rect: Rect, tl: Color32, br: Color32) {
    let Rect { min, max } = rect;
    // top + left (excluding the far corners, which belong to the shadow)
    px_rect(painter, min.x, min.y, max.x - 1.0, min.y + 1.0, tl);
    px_rect(painter, min.x, min.y, min.x + 1.0, max.y - 1.0, tl);
    // bottom + right
    px_rect(painter, min.x, max.y - 1.0, max.x, max.y, br);
    px_rect(painter, max.x - 1.0, min.y, max.x, max.y, br);
}

/// A plain 1px rectangle outline.
pub fn outline(painter: &Painter, rect: Rect, color: Color32) {
    two_tone(painter, rect, color, color);
}

/// Etched (engraved) 2px line frame, as used by group boxes.
pub fn etched(painter: &Painter, rect: Rect, t: &Theme) {
    outline(painter, rect.with_max_x(rect.max.x - 1.0).with_max_y(rect.max.y - 1.0), t.border_dark);
    two_tone(
        painter,
        rect.with_min_x(rect.min.x + 1.0).with_min_y(rect.min.y + 1.0),
        t.border_lightest,
        t.border_lightest,
    );
    // restore the inner dark edge that the white outline overwrote on top/left
    px_rect(painter, rect.min.x + 1.0, rect.min.y, rect.max.x - 1.0, rect.min.y + 1.0, t.border_dark);
    px_rect(painter, rect.min.x, rect.min.y + 1.0, rect.min.x + 1.0, rect.max.y - 1.0, t.border_dark);
}

/// Horizontal etched separator starting at `left`.
pub fn etched_hline(painter: &Painter, left: Pos2, width: f32, t: &Theme) {
    px_rect(painter, left.x, left.y, left.x + width, left.y + 1.0, t.border_dark);
    px_rect(painter, left.x, left.y + 1.0, left.x + width, left.y + 2.0, t.border_lightest);
}

/// Vertical etched separator starting at `top`.
pub fn etched_vline(painter: &Painter, top: Pos2, height: f32, t: &Theme) {
    px_rect(painter, top.x, top.y, top.x + 1.0, top.y + height, t.border_dark);
    px_rect(painter, top.x + 1.0, top.y, top.x + 2.0, top.y + height, t.border_lightest);
}

#[inline]
pub(crate) fn px_rect(painter: &Painter, x0: f32, y0: f32, x1: f32, y1: f32, c: Color32) {
    // snap to whole points so 1pt lines never straddle two pixels
    let (x0, y0, x1, y1) = (x0.round(), y0.round(), x1.round(), y1.round());
    if x1 > x0 && y1 > y0 {
        painter.rect_filled(Rect::from_min_max(pos2(x0, y0), pos2(x1, y1)), 0.0, c);
    }
}

/// Dotted focus rectangle, drawn every other point.
pub fn focus_rect(painter: &Painter, rect: Rect, color: Color32) {
    let Rect { min, max } = rect;
    let mut x = min.x;
    while x < max.x {
        px_rect(painter, x, min.y, x + 1.0, min.y + 1.0, color);
        px_rect(painter, x, max.y - 1.0, x + 1.0, max.y, color);
        x += 2.0;
    }
    let mut y = min.y + 2.0;
    while y < max.y - 1.0 {
        px_rect(painter, min.x, y, min.x + 1.0, y + 1.0, color);
        px_rect(painter, max.x - 1.0, y, max.x, y + 1.0, color);
        y += 2.0;
    }
}

fn checker_texture(ctx: &Context) -> TextureHandle {
    let id = Id::unique("rs95::checker");
    if let Some(t) = ctx.data(|d| d.get_temp::<TextureHandle>(id)) {
        return t;
    }
    let image = egui::ColorImage::new(
        [2, 2],
        vec![
            Color32::WHITE,
            Color32::TRANSPARENT,
            Color32::TRANSPARENT,
            Color32::WHITE,
        ],
    );
    let options = TextureOptions {
        magnification: egui::TextureFilter::Nearest,
        minification: egui::TextureFilter::Nearest,
        wrap_mode: egui::TextureWrapMode::Repeat,
        mipmap_mode: None,
    };
    let tex = ctx.load_texture("rs95-checker", image, options);
    ctx.data_mut(|d| d.insert_temp(id, tex.clone()));
    tex
}

/// Fill `rect` with the 50% dither pattern of `a` and `b`
/// (scrollbar tracks, pressed toggle buttons).
pub fn dither(painter: &Painter, rect: Rect, a: Color32, b: Color32) {
    painter.rect_filled(rect, 0.0, b);
    let tex = checker_texture(painter.ctx());
    // one texel per point, anchored to the screen so patterns line up
    let uv = Rect::from_min_max(
        (rect.min.to_vec2() / 2.0).to_pos2(),
        (rect.max.to_vec2() / 2.0).to_pos2(),
    );
    painter.image(tex.id(), rect, uv, a);
}

/// Draw pixel art. Each string is one row; each char is looked up in `palette`
/// (`None` = transparent). One char is one point.
pub fn pixmap(
    painter: &Painter,
    origin: Pos2,
    rows: &[&str],
    palette: impl Fn(char) -> Option<Color32>,
) {
    for (y, row) in rows.iter().enumerate() {
        let y = origin.y + y as f32;
        let chars: Vec<char> = row.chars().collect();
        let mut x = 0;
        while x < chars.len() {
            let Some(color) = palette(chars[x]) else {
                x += 1;
                continue;
            };
            // merge runs of the same char into one rectangle
            let start = x;
            while x < chars.len() && chars[x] == chars[start] {
                x += 1;
            }
            px_rect(
                painter,
                origin.x + start as f32,
                y,
                origin.x + x as f32,
                y + 1.0,
                color,
            );
        }
    }
}

/// Size of a pixmap in points.
pub fn pixmap_size(rows: &[&str]) -> Vec2 {
    vec2(
        rows.iter().map(|r| r.chars().count()).max().unwrap_or(0) as f32,
        rows.len() as f32,
    )
}

/// Draw a single-color pixmap centered in `rect`.
pub fn icon(painter: &Painter, rect: Rect, rows: &[&str], color: Color32) {
    let origin = (rect.center() - pixmap_size(rows) / 2.0).round();
    pixmap(painter, origin, rows, |c| (c == '#').then_some(color));
}

/// Draw text the way Windows 95 draws disabled text: a white copy offset by
/// one point below a gray one.
pub fn text(painter: &Painter, pos: Pos2, galley: Arc<Galley>, color: Color32, enabled: bool, t: &Theme) {
    let pos = pos.round();
    if enabled {
        painter.galley(pos, galley, color);
    } else {
        painter.galley_with_override_text_color(
            pos + vec2(1.0, 1.0),
            galley.clone(),
            t.material_text_disabled_shadow,
        );
        painter.galley_with_override_text_color(pos, galley, t.material_text_disabled);
    }
}

/// The diagonal resize grip found in the bottom right corner of windows
/// and status bars. `rect` should be at least 12x12.
pub fn resize_grip(painter: &Painter, rect: Rect, t: &Theme) {
    let origin = rect.max - vec2(12.0, 12.0);
    for y in 0..12 {
        for x in 0..12 {
            let s = x + y;
            let color = match s {
                10 | 14 | 18 => t.border_lightest,
                11 | 12 | 15 | 16 | 19 | 20 => t.border_dark,
                _ => continue,
            };
            let p = origin + vec2(x as f32, y as f32);
            px_rect(painter, p.x, p.y, p.x + 1.0, p.y + 1.0, color);
        }
    }
}

/// Pixel art used by the widgets.
pub mod icons {
    pub const CHECK: &[&str] = &[
        "......#", //
        ".....##",
        "#...###",
        "##.###.",
        "#####..",
        ".###...",
        "..#....",
    ];

    /// `d` dark, `k` darkest, `l` light, `w` lightest, `c` canvas.
    pub const RADIO: &[&str] = &[
        "....dddd....",
        "..ddkkkkdd..",
        ".dkkccccklw.",
        ".dkcccccclw.",
        "dkcccccccclw",
        "dkcccccccclw",
        "dkcccccccclw",
        "dkcccccccclw",
        ".dkcccccclw.",
        ".dllccccllw.",
        "..wwllllww..",
        "....wwww....",
    ];

    pub const RADIO_DOT: &[&str] = &[
        ".##.", //
        "####",
        "####",
        ".##.",
    ];

    pub const ARROW_DOWN: &[&str] = &[
        "#######", //
        ".#####.",
        "..###..",
        "...#...",
    ];

    pub const ARROW_UP: &[&str] = &[
        "...#...", //
        "..###..",
        ".#####.",
        "#######",
    ];

    pub const ARROW_LEFT: &[&str] = &[
        "...#", //
        "..##",
        ".###",
        "####",
        ".###",
        "..##",
        "...#",
    ];

    pub const ARROW_RIGHT: &[&str] = &[
        "#...", //
        "##..",
        "###.",
        "####",
        "###.",
        "##..",
        "#...",
    ];

    pub const SMALL_ARROW_UP: &[&str] = &[
        "..#..", //
        ".###.",
        "#####",
    ];

    pub const SMALL_ARROW_DOWN: &[&str] = &[
        "#####", //
        ".###.",
        "..#..",
    ];

    pub const CLOSE: &[&str] = &[
        "##....##", //
        ".##..##.",
        "..####..",
        "...##...",
        "..####..",
        ".##..##.",
        "##....##",
    ];

    pub const MINIMIZE: &[&str] = &[
        "......", //
        "......",
        "......",
        "......",
        "......",
        "######",
        "######",
    ];

    pub const MAXIMIZE: &[&str] = &[
        "#########", //
        "#########",
        "#.......#",
        "#.......#",
        "#.......#",
        "#.......#",
        "#.......#",
        "#########",
    ];

    pub const RESTORE: &[&str] = &[
        "..######", //
        "..######",
        "..#....#",
        "######.#",
        "######.#",
        "#....###",
        "#....#..",
        "#....#..",
        "######..",
    ];

    pub const HELP: &[&str] = &[
        ".####.", //
        "##..##",
        "....##",
        "...##.",
        "..##..",
        "......",
        "..##..",
    ];

}
