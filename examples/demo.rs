mod showcase;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        // no OS title bar: rs95::NativeWindow draws a Win95 one instead
        viewport: rs95::native_viewport(egui::ViewportBuilder::default())
            .with_inner_size([1300.0, 820.0])
            .with_title("rs95 demo"),
        ..Default::default()
    };
    let mut showcase = showcase::Showcase::default();
    eframe::run_ui_native("rs95 demo", options, move |ui, _frame| {
        showcase.ui(ui);
    })
}
