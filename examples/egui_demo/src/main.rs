#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![expect(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui;
use egui::Response;
use quicklook::{PreviewItem, QuickLookPanel, SourceFrame};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 360.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "quicklook-rs egui demo",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(MyApp::new()))
        }),
    )
}

struct MyApp {
    ql_panel: QuickLookPanel,

    picked_images: Vec<String>,
    dirty: bool,

    prev_screen_height: f32,
}

impl MyApp {
    fn new() -> Self {
        Self {
            ql_panel: QuickLookPanel::shared().unwrap(),
            picked_images: vec![],
            dirty: false,
            prev_screen_height: 0.,
        }
    }

    fn sync_preview_items(
        &self,
        window_height: f32,
        frame: &eframe::Frame,
        img_responses: &Vec<Response>,
    ) {
        self.ql_panel.set_items(
            self.picked_images
                .iter()
                .enumerate()
                .map(|(i, path)| {
                    PreviewItem::from_file_url(
                        path,
                        Some(
                            SourceFrame::window(
                                frame,
                                img_responses[i].rect.left() as f64,
                                // Convert y coordinate from relative to top of window to relative to bottom of window
                                (window_height
                                    - (img_responses[i].rect.top()
                                        + img_responses[i].rect.height()))
                                    as f64,
                                img_responses[i].rect.width() as f64,
                                img_responses[i].rect.height() as f64,
                            )
                            .unwrap(),
                        ),
                    )
                    .unwrap()
                })
                .collect(),
        );
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let current_screen_height = ui.ctx().content_rect().height();

            let mut responses = vec![];
            ui.horizontal_wrapped(|ui| {
                for path in &self.picked_images {
                    responses.push(
                        ui.add(
                            egui::Image::new(format!("file://{path}"))
                                .fit_to_exact_size(egui::Vec2::new(150., 150.))
                                .maintain_aspect_ratio(false),
                        ),
                    );
                }
            });

            if self.dirty {
                self.sync_preview_items(current_screen_height, frame, &responses);
                self.ql_panel.reload_if_dirty();
                self.dirty = false;
            }

            ui.horizontal(|ui| {
                if ui.button("Select images...").clicked()
                    && let Some(paths) = rfd::FileDialog::new()
                        .add_filter("Images Only", &["png", "jpg", "jpeg"])
                        .pick_files()
                {
                    self.picked_images = paths
                        .iter()
                        .map(|f| f.to_str().unwrap().to_string())
                        .collect();
                    self.dirty = true;
                }
                if ui.button("Show Preview Pane").clicked() {
                    self.ql_panel.show();
                }
            });

            if current_screen_height != self.prev_screen_height {
                // Resize detected, recalculate and update preview item source frames
                self.sync_preview_items(current_screen_height, frame, &responses);
            }
            self.prev_screen_height = current_screen_height;
        });
    }
}
