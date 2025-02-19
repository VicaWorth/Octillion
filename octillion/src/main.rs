// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui::{self, Color32};
use egui::vec2;

struct AppColors {
    primary: Color32,
    secondary: Color32,
    background: Color32,
    accent: Color32,
    secondary_accent: Color32
}

impl Default for AppColors {
    fn default() -> Self {
        Self {
            primary: Color32::from_rgb(142, 202, 230), // #8ecae6
            secondary: Color32::from_rgb(33, 158, 188), // #219ebb
            background: Color32::from_rgb(2, 48, 71),   // #023047
            accent: Color32::from_rgb(255, 183, 3),      // #ffb703
            secondary_accent: Color32::from_rgb(251, 133, 0), // #fb8500
        }
    }
}

fn main() -> eframe::Result {
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Octillion Desktop",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<MyApp>::default())
        }),
    )
}

struct Row {
    item: String,
    price: f32,
    quantity: i32,
}

struct MyApp {
    rows: Vec<Row>,
    app_colors: AppColors,
    show_popup: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            rows: vec![
                Row {
                    item: "Initial Item".to_string(),
                    price: 10.0,
                    quantity: 1,
                },
                Row {
                    item: "Another Item".to_string(),
                    price: 20.0,
                    quantity: 2,
                },
            ],
            app_colors: AppColors::default(),
            show_popup: false,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        ctx.style_mut(|style| {
            style.visuals.override_text_color = Some(self.app_colors.primary);

            style.spacing.item_spacing = egui::Vec2::new(8.0, 4.0);
            style.spacing.button_padding = egui::Vec2::new(12.0, 6.0);
            style.spacing.window_margin = egui::Margin::symmetric(25, 30);
            style.visuals.window_fill = self.app_colors.secondary;
            style.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::new(32.0, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            );
            // ... other style modifications ...
        });

        // central panel building
        egui::CentralPanel::default()
        .show(ctx, |ui| {
            
            let panel_width = ctx.available_rect().width();
            let content_width = panel_width * 0.5; // Half the panel width

            ui.vertical_centered(|ui| {
                egui::Frame::NONE
                .show(ui, |ui| {
                    ui.style_mut()
                        .spacing.item_spacing = vec2(16.0, 16.0);
                    ui.set_width(content_width);

                    
                    ui.heading("Budget App");

                    egui::Grid::new("budget_table")
                    .striped(true)
                    .min_col_width(content_width/3.0)
                    .spacing(egui::Vec2::new(8.0, 4.0))
                    .show(ui, |ui| {
                        ui.label("Item");
                        ui.label("Price");
                        ui.label("Quantity");
                        ui.end_row(); 
        
                        for row in &mut self.rows {
                            ui.vertical_centered_justified(|ui| {
                                ui.horizontal(|ui| { 
                                    ui.text_edit_singleline(&mut row.item);
                                });
                            });
                            ui.vertical_centered_justified(|ui| {
                                ui.add(egui::DragValue::new(&mut row.price)
                                .speed(0.1)
                                .update_while_editing(true));
                            });
                            ui.vertical_centered_justified(|ui| {
                                ui.add(egui::DragValue::new(&mut row.quantity)
                                .speed(1)
                                .update_while_editing(true));
                            });
                            ui.end_row(); 
                        }
                    });
        
                    ui.separator();
        
                    let total_price: f32 = self.rows.iter().map(|row| row.price * row.quantity as f32).sum();
                    
                    egui::Grid::new("summary_table").show(ui, |ui| {
                        ui.label("Total:");
                        ui.label(format!("${:.2}", total_price));
                        ui.end_row(); // Correctly end the summary row
                    });
        
                    ui.separator();
        
                    ui.horizontal_centered(|ui| {
                        if ui.button("Add Row").clicked() {
                            self.rows.push(Row {
                                item: "New Item".to_string(),
                                price: 0.0,
                                quantity: 1,
                            });
                        }
    
                        if ui.button("Open Popup").clicked() {
                            self.show_popup = true; // Show the popup when the button is clicked
                        }
            
                        // Popup window:
                        if self.show_popup {
                            egui::Window::new("Popup Window").show(ctx, |ui| {
                                // Content of the popup window
                                ui.label("This is a popup!");
                                if ui.button("Close").clicked() {
                                    self.show_popup = false; // Hide the popup when the close button is clicked
                                }
                            });
                        }
                    });
                })
            }) 
            
            // ui.image(egui::include_image!(
            //    "../../../crates/egui/assets/ferris.png"
            // ));

        });
    }
}