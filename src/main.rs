#![windows_subsystem = "windows"]

mod app;
mod utils;

use eframe::egui;

const WINDOW_TITLE: &str = "GDExtension Project Creator";
const MIN_WINDOW_SIZE: (f32, f32) = (500.0, 460.0);
const MAX_WINDOW_SIZE: (f32, f32) = (500.0, 460.0);
const RESIZABLE: bool = false;
const MAXIMIZE_BUTTON: bool = false;

#[tokio::main]
async fn main() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder {
            min_inner_size: Some(egui::vec2(MIN_WINDOW_SIZE.0, MIN_WINDOW_SIZE.1)),
            max_inner_size: Some(egui::vec2(MAX_WINDOW_SIZE.0, MAX_WINDOW_SIZE.1)),
            resizable: Some(RESIZABLE),
            maximize_button: Some(MAXIMIZE_BUTTON),
            ..Default::default()
        },
        ..Default::default()
    };

    let _ = eframe::run_native(WINDOW_TITLE, native_options, Box::new(|_| Ok(Box::new(app::App::default()))));
}
