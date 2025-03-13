#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui::{self, Color32};
use egui::vec2;

use futures::executor::block_on;

use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;
// use sqlx::Row; // Import Row
use dotenvy::dotenv;
use std::default::Default;
use std::env; // Import Default
              // use tokio::sync::mpsc;

// Creates the SQL Connection
async fn init_sql_connection() -> Result<MySqlPool, sqlx::Error> {
    dotenv().ok();
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file or environment");

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
}
#[derive(Debug, sqlx::FromRow, Clone)]
#[sqlx(rename_all = "camelCase")]
struct Product {
    // id: String, // VARCHAR(255) in SQL -> String in Rust
    name: Option<String>, // VARCHAR(255) -> String
    inventory_available: Option<i32>,
    image_url: Option<String>,
    price: Option<f64>,
    base_price: Option<f64>,
    base_price_per: Option<f64>,     // DECIMAL(10, 2) -> Option<f64>
    price_per: Option<f64>,          // DECIMAL(10, 2) -> Option<f64>
    unit_of_measure: Option<String>, // VARCHAR(10) -> Option<String>
}
// To handle the messages from the database
enum DbMessage {
    Products(Result<Vec<Product>, sqlx::Error>), // Changed to Products
    Error(sqlx::Error),
}

struct AppColors {
    primary: Color32,
    secondary: Color32,
    background: Color32,
    accent: Color32,
    secondary_accent: Color32,
}

impl Default for AppColors {
    fn default() -> Self {
        Self {
            primary: Color32::from_rgb(142, 202, 230),        // #8ecae6
            secondary: Color32::from_rgb(33, 158, 188),       // #219ebb
            background: Color32::from_rgb(2, 48, 71),         // #023047
            accent: Color32::from_rgb(255, 183, 3),           // #ffb703
            secondary_accent: Color32::from_rgb(251, 133, 0), // #fb8500
        }
    }
}

struct UIRow {
    item: String,
    price: f32,
    quantity: i32,
}

struct MyApp {
    rows: Vec<UIRow>,
    product_rows: Vec<Product>,
    app_colors: AppColors,
    show_popup: bool,
    loading_products: bool,
    search: String,
    error_message: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            rows: vec![],
            product_rows: vec![],
            app_colors: AppColors::default(),
            show_popup: false,
            loading_products: true,
            search: String::new(),
            error_message: String::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.style_mut(|style| {
            // style.visuals.override_text_color = Some(self.app_colors.primary);

            style.spacing.item_spacing = egui::Vec2::new(8.0, 4.0);
            style.spacing.button_padding = egui::Vec2::new(12.0, 6.0);
            style.spacing.window_margin = egui::Margin::symmetric(25.0, 30.0);
            // style.visuals.window_fill = self.app_colors.secondary;
            style.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::new(32.0, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            );
        });

        // central panel building
        egui::CentralPanel::default().show(ctx, |ui| {
            
            let panel_width = ctx.available_rect().width();
            let content_width = panel_width * 0.5; // Half the panel width

            ui.vertical_centered(|ui| {
                egui::Frame::default().show(ui, |ui| {
                    ui.style_mut().spacing.item_spacing = vec2(16.0, 16.0);
                    ui.set_width(content_width);

                    ui.heading("Budget App");

                    egui::Grid::new("budget_table")
                        .striped(true)
                        .min_col_width(content_width / 3.0)
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
                                    ui.add(
                                        egui::DragValue::new(&mut row.price)
                                            .speed(0.1)
                                            .update_while_editing(true),
                                    );
                                });
                                ui.vertical_centered_justified(|ui| {
                                    ui.add(
                                        egui::DragValue::new(&mut row.quantity)
                                            .speed(1)
                                            .update_while_editing(true),
                                    );
                                });
                                ui.end_row();
                            }
                        });

                    ui.separator();

                    let total_price: f32 = self
                        .rows
                        .iter()
                        .map(|row| row.price * row.quantity as f32)
                        .sum();

                    egui::Grid::new("summary_table").show(ui, |ui| {
                        ui.label("Total:");
                        ui.label(format!("${:.2}", total_price));
                        ui.end_row(); // Correctly end the summary row
                    });

                    ui.separator();

                    ui.horizontal_centered(|ui| {
                        if ui.button("Add Row").clicked() {
                            self.show_popup = true; // Show the popup when the button is clicked
                        }

                        // Popup window:
                        if self.show_popup {
                            egui::Window::new("Safeway Products").show(ctx, |ui| {
                                // Top Search Bar
                                ui.horizontal(|ui| {
                                    ui.label("Search:");
                                    ui.text_edit_singleline(&mut self.search);
                                    if ui.button("Search").clicked() {
                                        self.loading_products = true;
                                        // self.search = String::new(); // Don't clear the search here!
                                    }
                                });
                                ui.separator();

                                // Product Table
                                egui::ScrollArea::both().show(ui, |ui| {
                                    egui::Grid::new("product_table")
                                    .striped(true)
                                    .min_col_width(content_width / 6.0)  // Adjust as needed
                                    .spacing(egui::Vec2::new(2.0, 6.0))
                                    .show(ui, |ui| {
                                        // Header row
                                        ui.label("Item");
                                        ui.label("Inventory");
                                        ui.label("Price");
                                        ui.label("Base Price");
                                        ui.label("Price/Per");
                                        ui.label("Base Price/Per");
                                        ui.label("Unit");
                                        ui.end_row();

                                        if !self.error_message.is_empty() {
                                            ui.add(egui::Label::new(&self.error_message).wrap(true));
                                            ui.end_row();
                                        }

                                        if self.loading_products {
                                            ui.label("Loading products...");
                                        } else {
                                            // Display products *after* they are loaded.
                                            for product in &self.product_rows {
                                                // Item Name
                                                ui.label(product.name.clone().unwrap_or_default());

                                                // Inventory Available
                                                ui.label(format!("{}", product.inventory_available.unwrap_or(0)));


                                                // Price
                                                ui.label(format!("${:.2}", product.price.unwrap_or(0.0)));

                                                // Base Price
                                                ui.label(format!("${:.2}", product.base_price.unwrap_or(0.0)));

                                                // Price Per
                                                ui.label(format!("${:.2}", product.price_per.unwrap_or(0.0)));

                                                // Base Price Per
                                                ui.label(format!("${:.2}", product.base_price_per.unwrap_or(0.0)));
                                                
                                                // Unit of Measure
                                                ui.label(product.unit_of_measure.clone().unwrap_or_default());

                                                ui.end_row();
                                            }
                                        }
                                    });

                                if self.loading_products {
                                    // Get the database connection (synchronously, for this example ONLY)
                                    let pool_result =
                                        futures::executor::block_on(init_sql_connection()); // Correct blocking
                                    let pool = match pool_result {
                                        Ok(pool_val) => pool_val,
                                        Err(e) => {
                                            self.error_message = format!("Database connection error: {}", e); //store the error
                                            self.loading_products = false; // Set to false on error
                                            return; // Return early on connection error, but after setting the error
                                        }
                                    };

                                    // Execute the query (synchronously)
                                    let products_result = block_on(sqlx::query_as::<_, Product>(
                                    "SELECT name, inventoryAvailable, imageUrl, price, basePrice, pricePer, basePricePer, unitOfMeasure FROM products WHERE name LIKE ? ORDER BY basePricePer"
                                )
                                .bind(format!("%{}%", self.search))
                                .fetch_all(&pool));

                                    match products_result {
                                        Ok(products) => {
                                            self.product_rows = products;  // Populate product_rows
                                            self.loading_products = false; // Set to false after loading
                                        }
                                        Err(e) => {
                                            self.error_message =
                                                format!("Error fetching products: {}", e);
                                            self.loading_products = false;
                                        }
                                    }
                                } // End of moved block
                                ui.separator();
                                // Bottom Buttons
                                egui::Grid::new("popup_add_table").show(ui, |ui| {
                                    if ui.button("Add Row").clicked() {
                                        self.rows.push(UIRow {
                                            item: "New Item".to_string(),
                                            price: 0.0,
                                            quantity: 1,
                                        });
                                    }

                                    if ui.button("Close").clicked() {
                                        self.show_popup = false; // Hide the popup when the close button is clicked
                                    }
                                    ui.end_row();
                                });
                                })
                                
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

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    // Add back the Result
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

            Box::<MyApp>::default()
        }),
    )?; // Use ? to propagate errors from run_native

    Ok(()) // Return Ok(()) at the end
}
