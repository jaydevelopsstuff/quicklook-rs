#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![expect(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui;
use quicklook_rs::{PreviewItem, QuickLookPanel, SourceFrame};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 360.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "Native file dialogs and drag-and-drop files",
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
}

impl MyApp {
    fn new() -> Self {
        Self {
            ql_panel: QuickLookPanel::shared().unwrap(),
            picked_images: vec![],
            dirty: false,
        }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
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
                let monitor_frame = ui.input(|i| i.viewport().monitor_size.unwrap());
                let window_frame = ui.input(|i| i.viewport().inner_rect.unwrap());

                self.ql_panel.set_items(
                    self.picked_images
                        .iter()
                        .enumerate()
                        .map(|(i, path)| {
                            PreviewItem::from_file_url(
                                path,
                                Some(SourceFrame {
                                    x: (window_frame.left() + responses[i].rect.left()) as f64,
                                    y: (monitor_frame.y
                                        - (window_frame.top()
                                            + responses[i].rect.top()
                                            + responses[i].rect.height()))
                                        as f64,
                                    width: responses[i].rect.width() as f64,
                                    height: responses[i].rect.height() as f64,
                                }),
                            )
                            .unwrap()
                        })
                        .collect(),
                );
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
            })
        });
    }
}
