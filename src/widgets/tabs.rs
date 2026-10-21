use egui::{
    InnerResponse, Margin, Rect, Sense, Shape, TextStyle, TextWrapMode, Ui, WidgetText, pos2, vec2,
};

use crate::Theme;
use crate::paint::{self, Bevel};

/// A tab strip with a body (React95 `Tabs` + `Tab` + `TabBody`).
///
/// ```no_run
/// # egui::__run_test_ui(|ui| {
/// # let mut tab = 0;
/// rs95::Tabs::new(&mut tab)
///     .tab("General")
///     .tab("Advanced")
///     .show(ui, |ui, tab| {
///         ui.label(format!("page {tab}"));
///     });
/// # });
/// ```
pub struct Tabs<'a> {
    selected: &'a mut usize,
    tabs: Vec<WidgetText>,
}

const TAB_H: f32 = 20.0;

impl<'a> Tabs<'a> {
    pub fn new(selected: &'a mut usize) -> Self {
        Self {
            selected,
            tabs: Vec::new(),
        }
    }

    pub fn tab(mut self, text: impl Into<WidgetText>) -> Self {
        self.tabs.push(text.into());
        self
    }

    /// Show the tabs and the body of the selected tab.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_body: impl FnOnce(&mut Ui, usize) -> R,
    ) -> InnerResponse<R> {
        let t = Theme::get(ui.ctx());
        let enabled = ui.is_enabled();
        let n = self.tabs.len();
        if n > 0 && *self.selected >= n {
            *self.selected = n - 1;
        }
        let selected = *self.selected;

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            // tab strip, leaving 2pt on the left for the selected tab to grow into
            let galleys: Vec<_> = self
                .tabs
                .into_iter()
                .map(|text| {
                    text.into_galley(
                        ui,
                        Some(TextWrapMode::Extend),
                        f32::INFINITY,
                        TextStyle::Body,
                    )
                })
                .collect();
            let strip_w: f32 = galleys
                .iter()
                .map(|g| g.size().x.round() + 16.0)
                .sum::<f32>()
                + 4.0;
            let (strip, _) = ui.allocate_exact_size(vec2(strip_w, TAB_H + 2.0), Sense::hover());

            let mut rects = Vec::with_capacity(n);
            let mut x = strip.min.x + 2.0;
            for (i, g) in galleys.iter().enumerate() {
                let w = g.size().x.round() + 16.0;
                let r = Rect::from_min_max(pos2(x, strip.min.y + 2.0), pos2(x + w, strip.max.y));
                let resp = ui.interact(r, ui.scope_id().with(("rs95_tab", i)), Sense::click());
                if resp.clicked() {
                    *self.selected = i;
                }
                rects.push((r, resp));
                x += w;
            }

            // body
            let bg = ui.painter().add(Shape::Noop);
            let body = egui::Frame::NONE
                .inner_margin(Margin::same(10))
                .show(ui, |ui| {
                    ui.set_min_width(strip_w - 4.0);
                    add_body(ui, selected)
                });
            let body_rect = body.response.rect;
            ui.painter()
                .set(bg, Shape::rect_filled(body_rect, 0.0, t.material));
            let painter = ui.painter();
            paint::bevel(painter, body_rect, Bevel::Button, &t);

            // tabs on top of the body's border; the selected one last since it overlaps
            for pass_active in [false, true] {
                for (i, ((r, resp), g)) in rects.iter().zip(&galleys).enumerate() {
                    let active = i == *self.selected;
                    if active != pass_active {
                        continue;
                    }
                    let r = if active {
                        Rect::from_min_max(r.min - vec2(2.0, 2.0), r.max + vec2(2.0, 2.0))
                    } else {
                        *r
                    };
                    paint_tab(painter, r, &t);
                    let pos = pos2(
                        r.center().x - g.size().x / 2.0,
                        r.center().y - g.size().y / 2.0,
                    );
                    let galley_rect = Rect::from_min_size(pos.round(), g.size());
                    paint::text(
                        painter,
                        galley_rect.min,
                        g.clone(),
                        t.material_text,
                        enabled,
                        &t,
                    );
                    if active && resp.has_focus() {
                        paint::focus_rect(painter, galley_rect.expand(2.0), t.material_text);
                    }
                }
            }
            body.inner
        })
    }
}

fn paint_tab(painter: &egui::Painter, r: Rect, t: &Theme) {
    let Rect { min, max } = r;
    let px = |x0: f32, y0: f32, x1: f32, y1: f32, c| paint::px_rect(painter, x0, y0, x1, y1, c);
    // face
    px(min.x + 1.0, min.y + 1.0, max.x - 1.0, max.y, t.material);
    // left highlight
    px(min.x, min.y + 2.0, min.x + 1.0, max.y, t.border_lightest);
    px(min.x + 1.0, min.y + 2.0, min.x + 2.0, max.y, t.border_light);
    // rounded corner + top
    px(
        min.x + 1.0,
        min.y + 1.0,
        min.x + 2.0,
        min.y + 2.0,
        t.border_lightest,
    );
    px(
        min.x + 2.0,
        min.y,
        max.x - 2.0,
        min.y + 1.0,
        t.border_lightest,
    );
    px(
        min.x + 2.0,
        min.y + 1.0,
        max.x - 2.0,
        min.y + 2.0,
        t.border_light,
    );
    // right shadow
    px(
        max.x - 2.0,
        min.y + 1.0,
        max.x - 1.0,
        min.y + 2.0,
        t.border_darkest,
    );
    px(max.x - 1.0, min.y + 2.0, max.x, max.y, t.border_darkest);
    px(max.x - 2.0, min.y + 2.0, max.x - 1.0, max.y, t.border_dark);
}
