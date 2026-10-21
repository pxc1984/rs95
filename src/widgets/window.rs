use egui::{
    Context, Id, Margin, Mesh, Order, Pos2, Rect, Response, Sense, Shape,
    TextStyle, TextWrapMode, Ui, Vec2, WidgetText, pos2, vec2,
};

use crate::paint::{self, Bevel, icons};
use crate::Theme;

const HEADER_H: f32 = 18.0;
const BORDER: f32 = 3.0;

/// Which title bar buttons to show.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TitleButtons {
    pub help: bool,
    pub minimize: bool,
    pub maximize: bool,
    pub close: bool,
}

/// What happened in the title bar this frame.
#[derive(Clone, Debug)]
pub struct WindowResponse<R> {
    /// The whole window.
    pub response: Response,
    /// The title bar (drag it to move the window).
    pub header: Response,
    pub inner: R,
    pub help_clicked: bool,
    pub minimize_clicked: bool,
    pub maximize_clicked: bool,
    pub close_clicked: bool,
}

/// A Windows 95 window with a title bar (React95 `Window` + `WindowHeader` + `WindowContent`).
///
/// Use [`Window::show`] for a floating, draggable window, or
/// [`Window::show_inline`] to draw it as part of a layout.
///
/// ```no_run
/// # let ctx = egui::Context::default();
/// let mut open = true;
/// rs95::Window::new("Notepad")
///     .open(&mut open)
///     .show(&ctx, |ui| ui.label("Hello"));
/// ```
pub struct Window<'a> {
    title: WidgetText,
    id: Id,
    open: Option<&'a mut bool>,
    active: Option<bool>,
    buttons: TitleButtons,
    default_pos: Option<Pos2>,
    default_size: Option<Vec2>,
    resizable: bool,
    padding: Margin,
    min_size: Vec2,
}

#[derive(Clone, Copy, Debug)]
struct State {
    pos: Pos2,
    size: Option<Vec2>,
}

impl<'a> Window<'a> {
    pub fn new(title: impl Into<WidgetText>) -> Self {
        let title = title.into();
        Self {
            id: Id::unique(title.text()),
            title,
            open: None,
            active: None,
            buttons: TitleButtons {
                close: true,
                ..Default::default()
            },
            default_pos: None,
            default_size: None,
            resizable: false,
            padding: Margin::same(6),
            min_size: vec2(120.0, 40.0),
        }
    }

    /// Use a custom id instead of one derived from the title.
    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    /// Show a close button which sets `*open = false`. The window is
    /// not shown while `*open` is false.
    pub fn open(mut self, open: &'a mut bool) -> Self {
        self.open = Some(open);
        self
    }

    /// Force the title bar to the active (blue) or inactive (gray) look.
    /// By default floating windows are active when they are the topmost.
    pub fn active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }

    pub fn buttons(mut self, buttons: TitleButtons) -> Self {
        self.buttons = buttons;
        self
    }

    /// Show minimize and maximize buttons next to close (you handle the clicks).
    pub fn minimize_maximize(mut self, show: bool) -> Self {
        self.buttons.minimize = show;
        self.buttons.maximize = show;
        self
    }

    pub fn help_button(mut self, show: bool) -> Self {
        self.buttons.help = show;
        self
    }

    pub fn close_button(mut self, show: bool) -> Self {
        self.buttons.close = show;
        self
    }

    pub fn default_pos(mut self, pos: impl Into<Pos2>) -> Self {
        self.default_pos = Some(pos.into());
        self
    }

    /// Initial size of the content area.
    pub fn default_size(mut self, size: impl Into<Vec2>) -> Self {
        self.default_size = Some(size.into());
        self
    }

    /// Add a resize grip in the bottom right corner.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Padding around the content.
    pub fn padding(mut self, padding: impl Into<Margin>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn min_size(mut self, size: impl Into<Vec2>) -> Self {
        self.min_size = size.into();
        self
    }

    /// Show as a floating, draggable window. Returns `None` when closed.
    pub fn show<R>(
        mut self,
        ctx: &Context,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<WindowResponse<R>> {
        if self.open.as_deref() == Some(&false) {
            return None;
        }
        let state_id = self.id.with("rs95_window_state");
        let mut state = ctx.data(|d| d.get_temp::<State>(state_id)).unwrap_or(State {
            pos: self.default_pos.unwrap_or(pos2(32.0, 32.0)),
            size: self.default_size,
        });

        let area = egui::Area::new(self.id)
            .order(Order::Middle)
            .movable(false)
            .constrain(true)
            .current_pos(state.pos);
        let layer = egui::LayerId::new(Order::Middle, self.id);
        let active = self.active.unwrap_or_else(|| {
            ctx.memory(|m| m.areas().top_layer_id(Order::Middle)) == Some(layer)
        });
        self.active = Some(active);
        let open = self.open.take();
            let min_size = self.min_size;

        let out = area.show(ctx, |ui| self.ui(ui, state.size, add_contents)).inner;

        if out.header.dragged() {
            state.pos += out.header.drag_delta();
        }
        if out.response.is_pointer_button_down_on() || out.header.is_pointer_button_down_on() {
            ctx.move_to_top(layer);
        }
        if let Some(delta) = out.grip_drag {
            let current = state.size.unwrap_or(out.content_size);
            state.size = Some((current + delta).max(min_size));
        }
        if out.close_clicked {
            if let Some(open) = open {
                *open = false;
            }
        }
        ctx.data_mut(|d| d.insert_temp(state_id, state));
        Some(out.into_public())
    }

    /// Draw the window inside the current layout.
    pub fn show_inline<R>(
        mut self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<WindowResponse<R>> {
        if self.open.as_deref() == Some(&false) {
            return None;
        }
        let open = self.open.take();
        let size = self.default_size;
        let out = self.ui(ui, size, add_contents);
        if out.close_clicked {
            if let Some(open) = open {
                *open = false;
            }
        }
        Some(out.into_public())
    }

    fn ui<R>(
        self,
        ui: &mut Ui,
        size: Option<Vec2>,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Inner<R> {
        let t = Theme::get(ui.ctx());
        let active = self.active.unwrap_or(true);
        let bg = ui.painter().add(Shape::Noop);

        let mut content_rect = Rect::NOTHING;
        let frame = egui::Frame::NONE
            .inner_margin(Margin::same(BORDER as i8))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.add_space(HEADER_H + 1.0);
                let r = egui::Frame::NONE.inner_margin(self.padding).show(ui, |ui| {
                    ui.reset_style();
                    if let Some(size) = size {
                        ui.set_min_size(size);
                        ui.set_max_width(size.x);
                    } else {
                        ui.set_min_size(self.min_size);
                    }
                    add_contents(ui)
                });
                content_rect = r.response.rect;
                r.inner
            });

        let rect = frame.response.rect;
        let response = ui.interact(rect, self.id.with("rs95_window"), Sense::click());
        let painter = ui.painter();
        painter.set(bg, Shape::rect_filled(rect, 0.0, t.material));
        paint::bevel(painter, rect, Bevel::Window, &t);

        let header_rect = Rect::from_min_size(
            rect.min + Vec2::splat(BORDER),
            vec2(rect.width() - 2.0 * BORDER, HEADER_H),
        );
        let header = ui.interact(header_rect, self.id.with("rs95_header"), Sense::click_and_drag());

        let (bg_l, bg_r, fg) = if active {
            (t.header_background, t.header_gradient.unwrap_or(t.header_background), t.header_text)
        } else {
            (
                t.header_not_active_background,
                t.header_not_active_gradient.unwrap_or(t.header_not_active_background),
                t.header_not_active_text,
            )
        };
        painter.add(gradient(header_rect, bg_l, bg_r));

        // title buttons, right to left
        let btn = vec2(16.0, 14.0);
        let mut x = header_rect.max.x - 2.0;
        let y = header_rect.center().y - btn.y / 2.0;
        let mut clicked = [false; 4];
        let specs = [
            (self.buttons.close, icons::CLOSE, "close", 2.0),
            (self.buttons.maximize, icons::MAXIMIZE, "max", 0.0),
            (self.buttons.minimize, icons::MINIMIZE, "min", 2.0),
            (self.buttons.help, icons::HELP, "help", 0.0),
        ];
        let enabled = ui.is_enabled();
        for (i, (show, rows, salt, gap)) in specs.into_iter().enumerate() {
            if !show {
                continue;
            }
            let r = Rect::from_min_size(pos2(x - btn.x, y.round()), btn);
            let resp = ui.interact(r, self.id.with(("rs95_title_btn", salt)), Sense::click());
            let pressed = resp.is_pointer_button_down_on() && resp.hovered();
            painter.rect_filled(r, 0.0, t.material);
            paint::bevel(painter, r, if pressed { Bevel::ButtonPressed } else { Bevel::Button }, &t);
            let off = if pressed { vec2(1.0, 1.0) } else { Vec2::ZERO };
            let color = if enabled { t.material_text } else { t.material_text_disabled };
            paint::icon(painter, r.translate(off), rows, color);
            clicked[i] = resp.clicked();
            x -= btn.x + gap;
        }

        // bold title: draw twice, one point apart
        let galley = self.title.into_galley(ui, Some(TextWrapMode::Truncate), (x - header_rect.min.x - 6.0).max(0.0), TextStyle::Body);
        let pos = pos2(header_rect.min.x + 3.0, header_rect.center().y - galley.size().y / 2.0).round();
        painter.galley_with_override_text_color(pos, galley.clone(), fg);
        painter.galley_with_override_text_color(pos + vec2(1.0, 0.0), galley, fg);

        let mut grip_drag = None;
        if self.resizable {
            let inner = rect.shrink(BORDER);
            let grip_rect = Rect::from_min_max(inner.max - vec2(12.0, 12.0), inner.max);
            let grip = ui.interact(grip_rect, self.id.with("rs95_grip"), Sense::drag());
            if grip.hovered() || grip.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeNwSe);
            }
            if grip.dragged() {
                grip_drag = Some(grip.drag_delta());
            }
            paint::resize_grip(painter, inner, &t);
        }

        Inner {
            response,
            header,
            content_size: content_rect.size() - self.padding.sum(),
            grip_drag,
            inner: frame.inner,
            close_clicked: clicked[0],
            maximize_clicked: clicked[1],
            minimize_clicked: clicked[2],
            help_clicked: clicked[3],
        }
    }
}

struct Inner<R> {
    response: Response,
    header: Response,
    content_size: Vec2,
    grip_drag: Option<Vec2>,
    inner: R,
    help_clicked: bool,
    minimize_clicked: bool,
    maximize_clicked: bool,
    close_clicked: bool,
}

impl<R> Inner<R> {
    fn into_public(self) -> WindowResponse<R> {
        WindowResponse {
            response: self.response,
            header: self.header,
            inner: self.inner,
            help_clicked: self.help_clicked,
            minimize_clicked: self.minimize_clicked,
            maximize_clicked: self.maximize_clicked,
            close_clicked: self.close_clicked,
        }
    }
}

fn gradient(rect: Rect, left: egui::Color32, right: egui::Color32) -> Shape {
    let mut mesh = Mesh::default();
    mesh.colored_vertex(rect.left_top(), left);
    mesh.colored_vertex(rect.right_top(), right);
    mesh.colored_vertex(rect.right_bottom(), right);
    mesh.colored_vertex(rect.left_bottom(), left);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    Shape::mesh(mesh)
}
