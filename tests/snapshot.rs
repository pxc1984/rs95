//! Renders the showcase and compares it to `tests/snapshots/showcase.png`.
//!
//! Update the snapshot with `UPDATE_SNAPSHOTS=1 cargo test`.

#[path = "../examples/showcase/mod.rs"]
mod showcase;

use std::sync::Arc;

use egui_kittest::Harness;

/// The default kittest setup, but accept whatever limits the adapter has
/// (software rasterizers such as old lavapipe report very low ones).
fn wgpu_setup() -> egui_wgpu::WgpuSetup {
    let egui_wgpu::WgpuSetup::CreateNew(mut setup) = egui_kittest::wgpu::default_wgpu_setup()
    else {
        unreachable!()
    };
    // old Vulkan drivers lack VK_EXT_debug_utils
    setup.instance_descriptor.flags = egui_wgpu::wgpu::InstanceFlags::empty();
    setup.device_descriptor = Arc::new(|adapter| egui_wgpu::wgpu::DeviceDescriptor {
        label: Some("rs95 snapshot device"),
        required_limits: adapter.limits(),
        ..Default::default()
    });
    egui_wgpu::WgpuSetup::CreateNew(setup)
}

#[test]
fn showcase() {
    let mut state = showcase::Showcase::default();
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1300.0, 820.0))
        .with_pixels_per_point(1.0)
        .with_accessibility_check(false)
        .wgpu_setup(wgpu_setup())
        .build_ui(move |ui| state.ui(ui));
    harness.run();
    harness.snapshot("showcase");
}
