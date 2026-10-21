use egui::{
    InnerResponse, Rect, Sense, Ui, Vec2, pos2, scroll_area::ScrollBarVisibility, vec2,
};

use crate::paint::{self, Bevel, icons};
use crate::{Frame, Theme};

/// Width of a Win95 scrollbar.
pub const SCROLLBAR_W: f32 = 16.0;

/// A vertically scrolling area with Windows 95 scrollbars (React95 `ScrollView`).
///
/// The scrollbar has arrow buttons, a dithered track and a proportional thumb.
pub struct ScrollView {
    id_salt: egui::IdSalt,
    max_height: f32,
    frame: Option<Frame>,
}

impl ScrollView {
    pub fn new(id_salt: impl egui::AsIdSalt) -> Self {
        Self {
            id_salt: egui::IdSalt::new(id_salt),
            max_height: 200.0,
            frame: Some(Frame::field().inner_margin(0)),
        }
    }

    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = max_height;
        self
    }

    /// The frame around content and scrollbar. Defaults to a sunken field.
    pub fn frame(mut self, frame: Option<Frame>) -> Self {
        self.frame = frame;
        self
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let frame = self.frame;
        let body = |ui: &mut Ui| self.show_inner(ui, add_contents);
        match frame {
            Some(f) => f.show(ui, body),
            None => {
                let r = ui.scope(body);
                InnerResponse::new(r.inner, r.response)
            }
        }
    }

    fn show_inner<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        let t = Theme::get(ui.ctx());
        let id = ui.make_persistent_id(self.id_salt);
        let pending_id = id.with("pending");
        let pending: Option<f32> = ui.data_mut(|d| d.remove_temp(pending_id));

        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            let content_w = (ui.available_width() - SCROLLBAR_W).max(0.0);

            let mut area = egui::ScrollArea::vertical()
                .id_salt(id)
                .max_height(self.max_height)
                .auto_shrink([false, true])
                .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden);
            if let Some(offset) = pending {
                area = area.vertical_scroll_offset(offset);
            }
            let out = ui
                .allocate_ui(vec2(content_w, self.max_height), |ui| {
                    ui.set_width(content_w);
                    area.show(ui, |ui| {
                        egui::Frame::NONE.inner_margin(4).show(ui, add_contents).inner
                    })
                })
                .inner;

            let viewport = out.inner_rect.height();
            let content = out.content_size.y;
            let max_offset = (content - viewport).max(0.0);
            let offset = out.state.clamped_offset().y.clamp(0.0, max_offset);

            let (bar, _) = ui.allocate_exact_size(vec2(SCROLLBAR_W, viewport.max(2.0 * SCROLLBAR_W)), Sense::hover());
            let painter = ui.painter();
            let scrollable = max_offset > 0.0 && ui.is_enabled();

            let up = Rect::from_min_size(bar.min, Vec2::splat(SCROLLBAR_W));
            let down = Rect::from_min_size(pos2(bar.min.x, bar.max.y - SCROLLBAR_W), Vec2::splat(SCROLLBAR_W));
            let track = Rect::from_min_max(pos2(bar.min.x, up.max.y), pos2(bar.max.x, down.min.y));
            paint::dither(painter, track, t.border_lightest, t.material);

            let mut new_offset = offset;
            let step = 20.0;
            for (rect, rows, dir, salt) in [(up, icons::ARROW_UP, -1.0, "up"), (down, icons::ARROW_DOWN, 1.0, "down")] {
                let r = ui.interact(rect, id.with(salt), Sense::click());
                let held = scrollable && r.is_pointer_button_down_on() && r.hovered();
                painter.rect_filled(rect, 0.0, t.material);
                paint::bevel(painter, rect, if held { Bevel::ButtonPressed } else { Bevel::Button }, &t);
                let off = if held { vec2(1.0, 1.0) } else { Vec2::ZERO };
                if scrollable {
                    paint::icon(painter, rect.translate(off), rows, t.material_text);
                } else {
                    paint::icon(painter, rect.translate(vec2(1.0, 1.0)), rows, t.material_text_disabled_shadow);
                    paint::icon(painter, rect, rows, t.material_text_disabled);
                }
                if scrollable && r.clicked() {
                    new_offset += dir * step;
                }
            }

            if scrollable {
                let thumb_h = (track.height() * viewport / content).clamp(8.0, track.height());
                let free = track.height() - thumb_h;
                let thumb_y = track.min.y + free * offset / max_offset;
                let thumb = Rect::from_min_size(pos2(track.min.x, thumb_y.round()), vec2(SCROLLBAR_W, thumb_h.round()));

                let track_resp = ui.interact(track, id.with("track"), Sense::click());
                if track_resp.clicked() {
                    if let Some(p) = track_resp.interact_pointer_pos() {
                        let page = viewport - step;
                        new_offset += if p.y < thumb.min.y { -page } else if p.y > thumb.max.y { page } else { 0.0 };
                    }
                }
                let thumb_resp = ui.interact(thumb, id.with("thumb"), Sense::drag());
                if thumb_resp.dragged() && free > 0.0 {
                    new_offset += thumb_resp.drag_delta().y * max_offset / free;
                }
                painter.rect_filled(thumb, 0.0, t.material);
                paint::bevel(painter, thumb, Bevel::Button, &t);
            }

            let new_offset = new_offset.clamp(0.0, max_offset);
            if (new_offset - offset).abs() > 0.0 {
                ui.data_mut(|d| d.insert_temp(pending_id, new_offset));
                ui.ctx().request_repaint();
            }
            out.inner
        })
        .inner
    }
}
