use egui::{
    Response, Role, Sense, TextStyle, TextWrapMode, Ui, Vec2, Widget, WidgetInfo, WidgetText, vec2,
};

use crate::Theme;
use crate::paint::{self, Bevel};

/// Visual variant of a [`Button`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// The classic raised 3D push button.
    #[default]
    Default,
    /// Flat button for use on a white canvas.
    Flat,
    /// Borderless until hovered, then a thin 1px bevel (toolbar buttons).
    Thin,
    /// Like [`ButtonVariant::Thin`], used for menu bar entries.
    Menu,
}

enum Content {
    Text(WidgetText),
    Icon(&'static [&'static str]),
}

/// A Windows 95 push button.
///
/// ```no_run
/// # egui::__run_test_ui(|ui| {
/// if ui.add(rs95::Button::new("OK").primary(true)).clicked() {
///     // ...
/// }
/// # });
/// ```
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct Button {
    content: Content,
    variant: ButtonVariant,
    primary: bool,
    active: bool,
    square: bool,
    min_size: Vec2,
}

impl Button {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self::with_content(Content::Text(text.into()))
    }

    /// A button showing pixel art (see [`crate::paint::icons`]), `#` = text color.
    pub fn icon(rows: &'static [&'static str]) -> Self {
        Self::with_content(Content::Icon(rows))
    }

    fn with_content(content: Content) -> Self {
        Self {
            content,
            variant: ButtonVariant::Default,
            primary: false,
            active: false,
            square: false,
            min_size: Vec2::ZERO,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Shorthand for `.variant(ButtonVariant::Thin)`.
    pub fn thin(self) -> Self {
        self.variant(ButtonVariant::Thin)
    }

    /// Shorthand for `.variant(ButtonVariant::Flat)`.
    pub fn flat(self) -> Self {
        self.variant(ButtonVariant::Flat)
    }

    /// The default button of a dialog gets an extra black outline.
    pub fn primary(mut self, primary: bool) -> Self {
        self.primary = primary;
        self
    }

    /// Render as pushed in (toggle buttons, open menus).
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Make the button as wide as it is tall.
    pub fn square(mut self, square: bool) -> Self {
        self.square = square;
        self
    }

    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }
}

impl Widget for Button {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = Theme::get(ui.ctx());
        let padding = ui.spacing().button_padding;
        let border = 2.0 + if self.primary { 1.0 } else { 0.0 };

        let (galley, content_size) = match &self.content {
            Content::Text(text) => {
                let g = text.clone().into_galley(
                    ui,
                    Some(TextWrapMode::Extend),
                    f32::INFINITY,
                    TextStyle::Button,
                );
                let s = g.size();
                (Some(g), s)
            }
            Content::Icon(rows) => (None, paint::pixmap_size(rows)),
        };

        let mut size = content_size + 2.0 * (padding + Vec2::splat(border));
        size.y = size.y.max(ui.spacing().interact_size.y);
        if self.square {
            size.x = size.y;
        } else if galley.is_none() {
            size.x = size.x.max(size.y);
        }
        let size = size.max(self.min_size).round();

        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let enabled = ui.is_enabled();
        let label = galley
            .as_ref()
            .map(|g| g.text().to_owned())
            .unwrap_or_default();
        response.widget_info(|| {
            if self.active {
                WidgetInfo::selected(Role::Button, enabled, true, &label)
            } else {
                WidgetInfo::labeled(Role::Button, enabled, &label)
            }
        });

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let held = enabled && response.is_pointer_button_down_on() && response.hovered();
            let pressed = held || self.active;
            let hovered = enabled && response.hovered();

            let mut face = rect;
            if self.primary {
                paint::outline(painter, rect, t.border_darkest);
                face = rect.shrink(1.0);
            }

            match self.variant {
                ButtonVariant::Default => {
                    if self.active && !held {
                        paint::dither(painter, face, t.border_lightest, t.material);
                    } else {
                        painter.rect_filled(face, 0.0, t.material);
                    }
                    let b = if pressed {
                        Bevel::ButtonPressed
                    } else {
                        Bevel::Button
                    };
                    paint::bevel(painter, face, b, &t);
                }
                ButtonVariant::Flat => {
                    painter.rect_filled(face, 0.0, if pressed { t.flat_light } else { t.canvas });
                    paint::outline(painter, face, t.flat_dark);
                    paint::outline(painter, face.shrink(1.0), t.flat_light);
                }
                ButtonVariant::Thin | ButtonVariant::Menu => {
                    if self.active && !held && self.variant == ButtonVariant::Thin {
                        paint::dither(painter, face, t.border_lightest, t.material);
                    }
                    if pressed {
                        paint::bevel(painter, face, Bevel::ThinPressed, &t);
                    } else if hovered {
                        paint::bevel(painter, face, Bevel::Thin, &t);
                    }
                }
            }

            let offset = if pressed { vec2(1.0, 1.0) } else { Vec2::ZERO };
            let content_rect = egui::Rect::from_center_size(face.center() + offset, content_size);
            match (&self.content, galley) {
                (Content::Text(_), Some(galley)) => {
                    paint::text(
                        painter,
                        content_rect.min,
                        galley,
                        t.material_text,
                        enabled,
                        &t,
                    );
                }
                (Content::Icon(rows), _) => {
                    if enabled {
                        paint::icon(painter, content_rect, rows, t.material_text);
                    } else {
                        paint::icon(
                            painter,
                            content_rect.translate(vec2(1.0, 1.0)),
                            rows,
                            t.material_text_disabled_shadow,
                        );
                        paint::icon(painter, content_rect, rows, t.material_text_disabled);
                    }
                }
                _ => {}
            }

            if response.has_focus() && self.variant == ButtonVariant::Default {
                paint::focus_rect(painter, face.shrink(4.0), t.material_text);
            }
        }

        response
    }
}
