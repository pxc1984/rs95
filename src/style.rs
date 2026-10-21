use egui::{
    Color32, Context, CornerRadius, FontFamily, FontId, Id, Margin, Shadow, Stroke, Style,
    TextStyle, style::WidgetVisuals, vec2,
};

use crate::Theme;

/// Font size (in points) at which the bundled W95FA font is pixel perfect.
pub const FONT_SIZE: f32 = 16.0;

/// Install `theme` into the context: store it for rs95 widgets, restyle the
/// built-in egui widgets to match, and (with the `font` feature) install the
/// W95FA pixel font.
///
/// Call it once at startup, and again whenever the theme changes.
pub fn install(ctx: &Context, theme: Theme) {
    theme.store(ctx);
    #[cfg(feature = "font")]
    install_font(ctx);
    ctx.all_styles_mut(|style| apply_style(style, &theme));
}

/// Name of the bundled font in egui's font definitions.
#[cfg(feature = "font")]
pub const FONT_NAME: &str = "W95FA";

/// The bundled W95FA font (SIL Open Font License 1.1).
#[cfg(feature = "font")]
pub const FONT_DATA: &[u8] = include_bytes!("../assets/W95FA.otf");

/// Make W95FA the primary proportional and monospace font. Idempotent.
#[cfg(feature = "font")]
pub fn install_font(ctx: &Context) {
    let id = Id::unique("rs95::font_installed");
    if ctx.data(|d| d.get_temp::<bool>(id)).unwrap_or(false) {
        return;
    }
    ctx.data_mut(|d| d.insert_temp(id, true));
    ctx.add_font(egui::epaint::text::FontInsert::new(
        FONT_NAME,
        egui::FontData::from_static(FONT_DATA),
        vec![
            egui::epaint::text::InsertFontFamily {
                family: FontFamily::Proportional,
                priority: egui::epaint::text::FontPriority::Highest,
            },
            egui::epaint::text::InsertFontFamily {
                family: FontFamily::Monospace,
                priority: egui::epaint::text::FontPriority::Highest,
            },
        ],
    ));
}

fn widget(fill: Color32, stroke: Color32, text: Color32) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: Stroke::new(1.0, stroke),
        corner_radius: CornerRadius::ZERO,
        fg_stroke: Stroke::new(1.0, text),
        expansion: 0.0,
    }
}

/// Restyle an egui [`Style`] after `t`. Used by [`install`].
pub fn apply_style(style: &mut Style, t: &Theme) {
    let v = &mut style.visuals;
    v.dark_mode = false;
    v.override_text_color = None;
    v.weak_text_color = Some(t.material_text_disabled);

    v.widgets.noninteractive = widget(t.material, t.border_dark, t.material_text);
    v.widgets.inactive = widget(t.material, t.border_darkest, t.material_text);
    v.widgets.hovered = widget(t.material, t.border_darkest, t.material_text);
    v.widgets.active = widget(t.material_dark, t.border_darkest, t.material_text);
    v.widgets.open = widget(t.material_dark, t.border_darkest, t.material_text);

    v.selection.bg_fill = t.hover_background;
    v.selection.stroke = Stroke::new(1.0, t.canvas_text_invert);

    v.hyperlink_color = t.anchor;
    v.faint_bg_color = t.flat_light;
    v.extreme_bg_color = t.canvas;
    v.text_edit_bg_color = Some(t.canvas);
    v.code_bg_color = t.canvas;
    v.warn_fg_color = Color32::from_rgb(0x80, 0x80, 0x00);
    v.error_fg_color = Color32::from_rgb(0x80, 0x00, 0x00);

    v.window_corner_radius = CornerRadius::ZERO;
    v.window_shadow = Shadow::NONE;
    v.window_fill = t.material;
    v.window_stroke = Stroke::new(1.0, t.border_darkest);
    v.menu_corner_radius = CornerRadius::ZERO;
    v.panel_fill = t.material;
    v.popup_shadow = Shadow::NONE;

    v.text_cursor.stroke = Stroke::new(1.0, t.canvas_text);
    v.text_cursor.blink = true;
    v.button_frame = true;
    v.striped = false;
    v.slider_trailing_fill = false;
    v.handle_shape = egui::style::HandleShape::Rect { aspect_ratio: 0.5 };
    v.indent_has_left_vline = false;

    let s = &mut style.spacing;
    s.item_spacing = vec2(6.0, 6.0);
    s.button_padding = vec2(8.0, 3.0);
    s.interact_size = vec2(40.0, 23.0);
    s.menu_margin = Margin::same(3);
    s.window_margin = Margin::same(4);
    s.icon_width = 13.0;
    s.icon_width_inner = 7.0;
    s.icon_spacing = 6.0;
    s.combo_height = 300.0;
    s.scroll.bar_width = 16.0;
    s.scroll.floating = false;
    s.scroll.bar_inner_margin = 0.0;
    s.scroll.bar_outer_margin = 0.0;
    s.scroll.handle_min_length = 8.0;

    style.text_styles = [
        (TextStyle::Small, FontId::proportional(FONT_SIZE)),
        (TextStyle::Body, FontId::proportional(FONT_SIZE)),
        (TextStyle::Button, FontId::proportional(FONT_SIZE)),
        (TextStyle::Monospace, FontId::monospace(FONT_SIZE)),
        (TextStyle::Heading, FontId::proportional(FONT_SIZE * 2.0)),
    ]
    .into();

    style.animation_time = 0.0;
    style.interaction.tooltip_delay = 0.4;
}
