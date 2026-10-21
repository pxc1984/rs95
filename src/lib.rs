//! # rs95
//!
//! An authentic Windows 95 look for [egui](https://github.com/emilk/egui),
//! with the components of [React95](https://react95.io).
//!
//! ```no_run
//! # let ctx = egui::Context::default();
//! // once at startup (e.g. in your eframe creation callback)
//! rs95::install(&ctx, rs95::Theme::original());
//!
//! # egui::__run_test_ui(|ui| {
//! // then use the widgets
//! if ui.add(rs95::Button::new("OK").primary(true)).clicked() { /* ... */ }
//! # });
//! ```
//!
//! [`install`] also restyles egui's own widgets (square corners, no shadows,
//! the Win95 palette and the W95FA pixel font), so plain `ui.label` and
//! friends fit in. Everything else lives in the widgets of this crate,
//! which draw pixel-exact bevels with the [`paint`] helpers.

pub mod paint;
mod style;
mod theme;
mod widgets;

pub use paint::Bevel;
#[cfg(feature = "font")]
pub use style::{FONT_DATA, FONT_NAME, install_font};
pub use style::{FONT_SIZE, apply_style, install};
pub use theme::Theme;
pub use widgets::*;
