//! Color schemes.
//!
//! The field names follow the React95 theme so palettes can be ported 1:1.

use egui::{Color32, Context, Id};

/// A Windows 95 color scheme.
///
/// The active theme is stored in the [`Context`] by [`crate::install`] /
/// [`Theme::store`], and every rs95 widget reads it with [`Theme::get`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    /// Human readable name of the scheme.
    pub name: &'static str,

    /// Face color of buttons, windows, bars ("3D objects").
    pub material: Color32,
    /// Slightly darker face, used for pressed toggle states.
    pub material_dark: Color32,
    pub material_text: Color32,
    pub material_text_disabled: Color32,
    pub material_text_disabled_shadow: Color32,
    pub material_text_invert: Color32,

    /// Inner highlight of raised bevels.
    pub border_light: Color32,
    /// Outer highlight of raised bevels.
    pub border_lightest: Color32,
    /// Inner shadow of raised bevels.
    pub border_dark: Color32,
    /// Outer shadow of raised bevels.
    pub border_darkest: Color32,

    /// Background of text fields, lists and other "documents".
    pub canvas: Color32,
    pub canvas_text: Color32,
    pub canvas_text_disabled: Color32,
    pub canvas_text_invert: Color32,

    pub checkmark: Color32,
    pub checkmark_disabled: Color32,

    /// Background of an active window's title bar.
    pub header_background: Color32,
    /// If set, the title bar is a horizontal gradient (Windows 98 style).
    pub header_gradient: Option<Color32>,
    pub header_text: Color32,
    pub header_not_active_background: Color32,
    pub header_not_active_gradient: Option<Color32>,
    pub header_not_active_text: Color32,

    /// Selection / hovered menu item background.
    pub hover_background: Color32,
    pub progress: Color32,
    pub tooltip: Color32,
    pub desktop_background: Color32,
    pub anchor: Color32,

    /// Colors of `flat` variants (widgets placed on a canvas).
    pub flat_light: Color32,
    pub flat_dark: Color32,
}

const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

impl Default for Theme {
    fn default() -> Self {
        Self::original()
    }
}

impl Theme {
    /// The default React95 palette.
    pub const fn original() -> Self {
        Self {
            name: "original",
            material: hex(0xc6c6c6),
            material_dark: hex(0x9a9e9c),
            material_text: hex(0x0a0a0a),
            material_text_disabled: hex(0x848584),
            material_text_disabled_shadow: hex(0xfefefe),
            material_text_invert: hex(0xfefefe),
            border_light: hex(0xdfdfdf),
            border_lightest: hex(0xfefefe),
            border_dark: hex(0x848584),
            border_darkest: hex(0x0a0a0a),
            canvas: hex(0xffffff),
            canvas_text: hex(0x0a0a0a),
            canvas_text_disabled: hex(0x848584),
            canvas_text_invert: hex(0xfefefe),
            checkmark: hex(0x0a0a0a),
            checkmark_disabled: hex(0x848584),
            header_background: hex(0x060084),
            header_gradient: None,
            header_text: hex(0xfefefe),
            header_not_active_background: hex(0x7f787f),
            header_not_active_gradient: None,
            header_not_active_text: hex(0xc6c6c6),
            hover_background: hex(0x060084),
            progress: hex(0x060084),
            tooltip: hex(0xfefbcc),
            desktop_background: hex(0x008080),
            anchor: hex(0x1034a6),
            flat_light: hex(0xd8d8d8),
            flat_dark: hex(0x9e9e9e),
        }
    }

    /// The exact "Windows Standard" scheme of Windows 95.
    pub const fn classic() -> Self {
        Self {
            name: "classic",
            material: hex(0xc0c0c0),
            material_dark: hex(0xa0a0a0),
            material_text: hex(0x000000),
            material_text_disabled: hex(0x808080),
            material_text_disabled_shadow: hex(0xffffff),
            material_text_invert: hex(0xffffff),
            border_light: hex(0xdfdfdf),
            border_lightest: hex(0xffffff),
            border_dark: hex(0x808080),
            border_darkest: hex(0x000000),
            canvas: hex(0xffffff),
            canvas_text: hex(0x000000),
            canvas_text_disabled: hex(0x808080),
            canvas_text_invert: hex(0xffffff),
            checkmark: hex(0x000000),
            checkmark_disabled: hex(0x808080),
            header_background: hex(0x000080),
            header_gradient: None,
            header_text: hex(0xffffff),
            header_not_active_background: hex(0x808080),
            header_not_active_gradient: None,
            header_not_active_text: hex(0xc0c0c0),
            hover_background: hex(0x000080),
            progress: hex(0x000080),
            tooltip: hex(0xffffe1),
            desktop_background: hex(0x008080),
            anchor: hex(0x0000ff),
            flat_light: hex(0xd8d8d8),
            flat_dark: hex(0x9e9e9e),
        }
    }

    /// Windows 98: the classic scheme with gradient title bars.
    pub const fn win98() -> Self {
        let mut t = Self::classic();
        t.name = "win98";
        t.header_gradient = Some(hex(0x1084d0));
        t.header_not_active_gradient = Some(hex(0xb5b5b5));
        t
    }

    /// Approximation of the "Lilac" scheme.
    pub const fn lilac() -> Self {
        Self::tinted("lilac", hex(0xb0a8d8), hex(0x5a4e9a), hex(0x587070))
    }

    /// Approximation of the "Marine" scheme.
    pub const fn marine() -> Self {
        Self::tinted("marine", hex(0x88b8b0), hex(0x004860), hex(0x003840))
    }

    /// Approximation of the "Rose" scheme.
    pub const fn rose() -> Self {
        Self::tinted("rose", hex(0xd8b0b8), hex(0x803848), hex(0x804050))
    }

    /// Approximation of the "Spruce" scheme.
    pub const fn spruce() -> Self {
        Self::tinted("spruce", hex(0xa0c8a0), hex(0x386838), hex(0x305830))
    }

    /// Approximation of the "Wheat" scheme.
    pub const fn wheat() -> Self {
        Self::tinted("wheat", hex(0xd8d0a8), hex(0x808000), hex(0x707048))
    }

    /// Build a scheme from a face color, a title bar color and a desktop color.
    /// Bevel shades are derived from the face color.
    pub const fn tinted(
        name: &'static str,
        material: Color32,
        header: Color32,
        desktop: Color32,
    ) -> Self {
        let mut t = Self::original();
        t.name = name;
        t.material = material;
        t.material_dark = scale(material, 80);
        t.border_light = mix_white(material, 50);
        t.border_dark = scale(material, 55);
        t.material_text_disabled = scale(material, 55);
        t.header_background = header;
        t.hover_background = header;
        t.progress = header;
        t.header_not_active_background = scale(material, 65);
        t.header_not_active_text = material;
        t.desktop_background = desktop;
        t
    }

    /// All built-in schemes.
    pub fn all() -> [Self; 8] {
        [
            Self::original(),
            Self::classic(),
            Self::win98(),
            Self::lilac(),
            Self::marine(),
            Self::rose(),
            Self::spruce(),
            Self::wheat(),
        ]
    }

    fn id() -> Id {
        Id::unique("rs95::theme")
    }

    /// The theme stored in this context, or [`Theme::original`].
    pub fn get(ctx: &Context) -> Self {
        ctx.data(|d| d.get_temp(Self::id())).unwrap_or_default()
    }

    /// Store this theme in the context without touching egui's own visuals.
    /// Prefer [`crate::install`], which does both.
    pub fn store(self, ctx: &Context) {
        ctx.data_mut(|d| d.insert_temp(Self::id(), self));
    }
}

const fn scale(c: Color32, percent: u32) -> Color32 {
    Color32::from_rgb(
        (c.r() as u32 * percent / 100) as u8,
        (c.g() as u32 * percent / 100) as u8,
        (c.b() as u32 * percent / 100) as u8,
    )
}

const fn mix_white(c: Color32, percent: u32) -> Color32 {
    const fn ch(v: u8, percent: u32) -> u8 {
        (v as u32 + (255 - v as u32) * percent / 100) as u8
    }
    Color32::from_rgb(ch(c.r(), percent), ch(c.g(), percent), ch(c.b(), percent))
}
