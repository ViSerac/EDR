mod collectors;
mod core;
mod gui;
mod models;
mod pipeline;
mod rules;

use eframe::egui;

use crate::gui::app::EdrApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("mini EDR")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([960.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "mini_edr",
        options,
        Box::new(|cc| Ok(Box::new(EdrApp::new(cc)))),
    )
}
