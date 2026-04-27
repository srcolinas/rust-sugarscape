mod app;

use app::SugarscapeApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Sugarscape GUI",
        options,
        Box::new(|_cc| Ok(Box::new(SugarscapeApp::new()))),
    )
}
