use egui::{
    InnerResponse, Popup, PopupCloseBehavior, RectAlign, Response, Sense, TextStyle,
    TextWrapMode, Ui, Widget, WidgetText, pos2, vec2,
};

use crate::paint::{self, icons};
use crate::{Button, ButtonVariant, Frame, Theme};

/// A raised popup menu panel (React95 `MenuList`). Fill it with [`MenuItem`]s.
pub struct MenuList;

impl MenuList {
    /// Draw a menu list inline.
    pub fn show<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        Frame::window().inner_margin(1).show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            ui.set_min_width(120.0);
            ui.vertical(add_contents).inner
        })
    }
}

/// An entry of a [`MenuList`] (React95 `MenuListItem`). Highlighted on hover.
#[must_use = "You should put this widget in a ui with `ui.add(widget);`"]
pub struct MenuItem {
    text: WidgetText,
    shortcut: Option<WidgetText>,
    checked: bool,
    submenu: bool,
}

impl MenuItem {
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self {
            text: text.into(),
            shortcut: None,
            checked: false,
            submenu: false,
        }
    }

    /// Right aligned shortcut hint, e.g. `"Ctrl+O"`.
    pub fn shortcut(mut self, shortcut: impl Into<WidgetText>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Show a check mark in front of the label.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Show a submenu arrow.
    pub fn submenu(mut self, submenu: bool) -> Self {
        self.submenu = submenu;
        self
    }
}

impl Widget for MenuItem {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = Theme::get(ui.ctx());
        let layout = |w: WidgetText| {
            w.into_galley(ui, Some(TextWrapMode::Extend), f32::INFINITY, TextStyle::Body)
        };
        let galley = layout(self.text);
        let shortcut = self.shortcut.map(layout);
        let gutter = 20.0;
        let mut w = gutter + galley.size().x + 20.0;
        if let Some(s) = &shortcut {
            w += s.size().x + 20.0;
        }
        let w = w.max(ui.available_width());
        let (rect, response) = ui.allocate_exact_size(vec2(w, 22.0), Sense::click());
        let enabled = ui.is_enabled();
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let highlight = response.hovered() && enabled;
            if highlight {
                painter.rect_filled(rect, 0.0, t.hover_background);
            }
            let color = if highlight { t.canvas_text_invert } else { t.material_text };
            let y = rect.center().y - galley.size().y / 2.0;
            if self.checked {
                let r = egui::Rect::from_min_max(rect.min, pos2(rect.min.x + gutter, rect.max.y));
                paint::icon(painter, r, icons::CHECK, if enabled { color } else { t.material_text_disabled });
            }
            if highlight {
                painter.galley_with_override_text_color(pos2(rect.min.x + gutter, y).round(), galley, color);
            } else {
                paint::text(painter, pos2(rect.min.x + gutter, y), galley, color, enabled, &t);
            }
            if let Some(s) = shortcut {
                let pos = pos2(rect.max.x - 20.0 - s.size().x, y);
                if highlight {
                    painter.galley_with_override_text_color(pos.round(), s, color);
                } else {
                    paint::text(painter, pos, s, color, enabled, &t);
                }
            }
            if self.submenu {
                let r = egui::Rect::from_min_max(pos2(rect.max.x - 16.0, rect.min.y), rect.max);
                paint::icon(painter, r, icons::ARROW_RIGHT, color);
            }
        }
        response
    }
}

/// A separator line between groups of [`MenuItem`]s.
pub fn menu_separator(ui: &mut Ui) {
    let t = Theme::get(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 8.0), Sense::hover());
    paint::etched_hline(
        ui.painter(),
        pos2(rect.min.x + 2.0, rect.center().y - 1.0).round(),
        rect.width() - 4.0,
        &t,
    );
}

/// A button that opens a drop-down [`MenuList`] (menu bars, the Start button).
pub struct MenuButton {
    button: Button,
    align: RectAlign,
}

impl MenuButton {
    /// A borderless menu bar entry, like "File".
    pub fn new(text: impl Into<WidgetText>) -> Self {
        Self::from_button(Button::new(text).variant(ButtonVariant::Menu))
    }

    /// Use any [`Button`] as the trigger.
    pub fn from_button(button: Button) -> Self {
        Self {
            button,
            align: RectAlign::BOTTOM_START,
        }
    }

    /// Where the menu opens relative to the button (e.g. `RectAlign::TOP_START`
    /// for a taskbar Start menu).
    pub fn align(mut self, align: RectAlign) -> Self {
        self.align = align;
        self
    }

    /// Clicking a [`MenuItem`] closes the menu.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> (Response, Option<R>) {
        let id = ui.next_auto_id().with("rs95_menu");
        let open = Popup::is_id_open(ui.ctx(), id);
        let response = ui.add(self.button.active(open));
        let inner = Popup::from_toggle_button_response(&response)
            .id(id)
            .align(self.align)
            .gap(0.0)
            .close_behavior(PopupCloseBehavior::CloseOnClick)
            .frame(egui::Frame::NONE)
            .show(|ui| MenuList::show(ui, add_contents).inner)
            .map(|r| r.inner);
        (response, inner)
    }
}
