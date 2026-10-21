use egui::{
    CursorIcon, Id, InnerResponse, Margin, Rect, ResizeDirection, Sense, Ui, UiBuilder,
    ViewportBuilder, ViewportCommand, WidgetText, pos2, vec2,
};

use super::window::{TitleBar, TitleButtons, title_bar};
use crate::Theme;
use crate::paint::{self, Bevel};

const BORDER: f32 = 3.0;
const HEADER_H: f32 = 18.0;
/// Width of the invisible resize zone along the window edges.
const GRAB: f32 = 5.0;

/// Turn off the operating system's window decorations so that
/// [`NativeWindow`] can draw a Windows 95 frame instead.
/// Works with eframe on Windows, macOS and Linux (X11 and Wayland).
///
/// ```no_run
/// let options = eframe::NativeOptions {
///     viewport: rs95::native_viewport(egui::ViewportBuilder::default())
///         .with_inner_size([800.0, 600.0]),
///     ..Default::default()
/// };
/// ```
pub fn native_viewport(builder: ViewportBuilder) -> ViewportBuilder {
    builder
        .with_decorations(false)
        .with_transparent(false)
        .with_min_inner_size([160.0, 80.0])
}

/// The window frame of the whole application: a Win95 title bar with
/// minimize / maximize / close buttons, a raised border and resizable edges,
/// replacing the native decorations of Windows, macOS and Linux.
///
/// Create the viewport with [`native_viewport`] (no OS decorations) and wrap
/// your root UI in [`NativeWindow::show`]:
///
/// ```no_run
/// # struct App;
/// # impl eframe::App for App {
/// fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
///     rs95::NativeWindow::new("My App").show(ui, |ui| {
///         ui.label("Hello, Windows 95!");
///     });
/// }
/// # }
/// ```
///
/// Dragging the title bar moves the window, double-clicking it maximizes,
/// and the edges and corners resize. Everything is done with
/// [`ViewportCommand`]s, so the OS window manager handles snapping etc.
pub struct NativeWindow {
    title: WidgetText,
    buttons: TitleButtons,
    resizable: bool,
    padding: Margin,
    set_os_title: bool,
}

impl NativeWindow {
    pub fn new(title: impl Into<WidgetText>) -> Self {
        Self {
            title: title.into(),
            buttons: TitleButtons {
                minimize: true,
                maximize: true,
                close: true,
                help: false,
            },
            resizable: true,
            padding: Margin::same(2),
            set_os_title: true,
        }
    }

    pub fn buttons(mut self, buttons: TitleButtons) -> Self {
        self.buttons = buttons;
        self
    }

    /// Allow resizing by dragging the edges (default `true`).
    /// Also hides the maximize button when `false`.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        if !resizable {
            self.buttons.maximize = false;
        }
        self
    }

    /// Padding between the frame and the content.
    pub fn padding(mut self, padding: impl Into<Margin>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Also set the OS window title (shown in the taskbar / dock). Default `true`.
    pub fn set_os_title(mut self, set: bool) -> Self {
        self.set_os_title = set;
        self
    }

    /// Fill `ui` (normally the root ui of the app) with the window frame and
    /// put the contents inside it.
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let ctx = ui.ctx().clone();
        let t = Theme::get(&ctx);
        let id = Id::unique("rs95::native_window");
        let rect = ui.max_rect();
        let (maximized, fullscreen, focused, os_title) = ctx.input(|i| {
            let v = i.viewport();
            (
                v.maximized.unwrap_or(false),
                v.fullscreen.unwrap_or(false),
                v.focused.unwrap_or(true),
                v.title.clone(),
            )
        });

        if self.set_os_title {
            // send once per change; not every backend reports the title back
            let title = self.title.text();
            let sent_id = id.with("sent_title");
            let sent = ctx.data(|d| d.get_temp::<String>(sent_id));
            if os_title.as_deref() != Some(title) && sent.as_deref() != Some(title) {
                ctx.send_viewport_cmd(ViewportCommand::Title(title.to_owned()));
                ctx.data_mut(|d| d.insert_temp(sent_id, title.to_owned()));
            }
        }

        if fullscreen {
            // no chrome at all in fullscreen
            let inner = ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
                ui.painter().rect_filled(rect, 0.0, t.material);
                add_contents(ui)
            });
            return inner;
        }

        // background + border (a maximized Win95 window keeps its frame
        // but it is pushed off screen, so we drop it)
        let border = if maximized { 0.0 } else { BORDER };
        let painter = ui.painter().clone();
        painter.rect_filled(rect, 0.0, t.material);
        if !maximized {
            paint::bevel(&painter, rect, Bevel::Window, &t);
        }

        let inner_rect = rect.shrink(border);
        let header_rect = Rect::from_min_size(
            inner_rect.min + vec2(0.0, if maximized { 1.0 } else { 0.0 }),
            vec2(inner_rect.width(), HEADER_H),
        );
        let content_rect = Rect::from_min_max(
            pos2(inner_rect.min.x, header_rect.max.y + 1.0),
            inner_rect.max,
        );

        // contents first, so the frame's widgets end up on top for hit testing
        let inner = ui.scope_builder(
            UiBuilder::new().max_rect(content_rect - self.padding),
            |ui| {
                ui.set_clip_rect(content_rect);
                add_contents(ui)
            },
        );

        let buttons = if self.resizable {
            self.buttons
        } else {
            TitleButtons {
                maximize: false,
                ..self.buttons
            }
        };
        let TitleBar { header, clicked } =
            title_bar(ui, header_rect, id, self.title, focused, buttons, maximized);
        let [close, maximize, minimize, _help] = clicked;

        if header.drag_started_by(egui::PointerButton::Primary) {
            ctx.send_viewport_cmd(ViewportCommand::StartDrag);
        }
        if (header.double_clicked() && self.resizable) || maximize {
            ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
        }
        if minimize {
            ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
        }
        if close {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }

        if self.resizable && !maximized {
            resize_edges(ui, rect, id);
            paint::resize_grip(&painter, inner_rect, &t);
        }

        InnerResponse::new(inner.inner, header)
    }
}

/// Invisible drag zones along the edges that start an OS resize.
fn resize_edges(ui: &Ui, rect: Rect, id: Id) {
    use ResizeDirection::*;
    let (l, r, t, b) = (rect.min.x, rect.max.x, rect.min.y, rect.max.y);
    let corner = 14.0;
    let zones = [
        (
            North,
            Rect::from_min_max(pos2(l + corner, t), pos2(r - corner, t + GRAB)),
            CursorIcon::ResizeVertical,
        ),
        (
            South,
            Rect::from_min_max(pos2(l + corner, b - GRAB), pos2(r - corner, b)),
            CursorIcon::ResizeVertical,
        ),
        (
            West,
            Rect::from_min_max(pos2(l, t + corner), pos2(l + GRAB, b - corner)),
            CursorIcon::ResizeHorizontal,
        ),
        (
            East,
            Rect::from_min_max(pos2(r - GRAB, t + corner), pos2(r, b - corner)),
            CursorIcon::ResizeHorizontal,
        ),
        (
            NorthWest,
            Rect::from_min_max(pos2(l, t), pos2(l + corner, t + GRAB)),
            CursorIcon::ResizeNwSe,
        ),
        (
            NorthWest,
            Rect::from_min_max(pos2(l, t), pos2(l + GRAB, t + corner)),
            CursorIcon::ResizeNwSe,
        ),
        (
            NorthEast,
            Rect::from_min_max(pos2(r - corner, t), pos2(r, t + GRAB)),
            CursorIcon::ResizeNeSw,
        ),
        (
            NorthEast,
            Rect::from_min_max(pos2(r - GRAB, t), pos2(r, t + corner)),
            CursorIcon::ResizeNeSw,
        ),
        (
            SouthWest,
            Rect::from_min_max(pos2(l, b - GRAB), pos2(l + corner, b)),
            CursorIcon::ResizeNeSw,
        ),
        (
            SouthWest,
            Rect::from_min_max(pos2(l, b - corner), pos2(l + GRAB, b)),
            CursorIcon::ResizeNeSw,
        ),
        // the whole grip in the bottom right corner
        (
            SouthEast,
            Rect::from_min_max(pos2(r - corner, b - corner), pos2(r, b)),
            CursorIcon::ResizeNwSe,
        ),
    ];
    for (i, (dir, zone, cursor)) in zones.into_iter().enumerate() {
        let resp = ui.interact(zone, id.with(("rs95_resize", i)), Sense::drag());
        if resp.hovered() || resp.dragged() {
            ui.ctx().set_cursor_icon(cursor);
        }
        if resp.drag_started_by(egui::PointerButton::Primary) {
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::BeginResize(dir));
        }
    }
}
