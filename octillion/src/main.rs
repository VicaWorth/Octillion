#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui::{self, Color32, Ui}; // Added Ui
use egui::vec2;

use futures::executor::block_on;

use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;
// use sqlx::Row; // Import Row - Not strictly needed with FromRow
use dotenvy::dotenv;
use std::default::Default;
use std::env; // Import Default

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
    name: Option<String>,
    inventory_available: Option<i32>,
    image_url: Option<String>, // Keep if needed later, otherwise remove
    price: Option<f64>,
    base_price: Option<f64>,
    base_price_per: Option<f64>,
    price_per: Option<f64>,
    unit_of_measure: Option<String>,
}

// --- No changes to DbMessage, AppColors, UIRow needed ---
enum DbMessage {
    Products(Result<Vec<Product>, sqlx::Error>),
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

// --- MyApp struct remains the same ---
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
            loading_products: false, // Start as false, trigger loading on search/popup open
            search: String::new(),
            error_message: String::new(),
        }
    }
}

// --- Helper methods for UI components ---
impl MyApp {
    /// Renders the main budget items table.
    fn ui_budget_table(&mut self, ui: &mut Ui, content_width: f32) {
        egui::Grid::new("budget_table")
            .striped(true)
            .min_col_width(content_width / 3.0)
            .spacing(egui::Vec2::new(8.0, 4.0))
            .show(ui, |ui| {
                ui.label("Item");
                ui.label("Price");
                ui.label("Quantity");
                ui.end_row();

                // Use retain_mut for potential future deletion logic if needed
                // For now, simple iteration is fine.
                for row in &mut self.rows {
                    ui.vertical_centered_justified(|ui| {
                         // Use available_width for text edit to fill space potentially
                        ui.add_sized(ui.available_size(), egui::TextEdit::singleline(&mut row.item));
                    });
                    ui.vertical_centered_justified(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut row.price)
                                .speed(0.1)
                                .update_while_editing(true)
                                .prefix("$"), // Add prefix for clarity
                        );
                    });
                    ui.vertical_centered_justified(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut row.quantity)
                                .speed(1)
                                .update_while_editing(true)
                                .clamp_range(0..=999), // Add a clamp range
                        );
                    });
                    ui.end_row();
                }
            });
    }

    /// Renders the summary row with the total price.
    fn ui_summary(&self, ui: &mut Ui) {
        let total_price: f32 = self
            .rows
            .iter()
            .map(|row| row.price * row.quantity as f32)
            .sum();

        egui::Grid::new("summary_table").show(ui, |ui| {
            ui.label("Total:");
            ui.label(format!("${:.2}", total_price));
            ui.end_row();
        });
    }

    /// Handles the display and logic of the product search popup window.
    fn ui_product_popup(&mut self, ctx: &egui::Context, content_width: f32) {
        // Only proceed if the popup should be shown
        if !self.show_popup {
            return;
        }

        let mut open = self.show_popup; // Use a variable to control window visibility

        egui::Window::new("Safeway Products")
            .open(&mut open) // Control window state
            .resizable(true)
            .default_width(content_width * 1.5) // Make popup wider
            .show(ctx, |ui| {
                self.ui_popup_content(ui, content_width);
            });

        // Update show_popup state based on whether the window was closed
        self.show_popup = open;
        if !self.show_popup {
            // Optional: Reset state when popup closes
            self.search.clear();
            self.product_rows.clear();
            self.error_message.clear();
            self.loading_products = false;
        }
    }

    /// Renders the content *inside* the product popup window.
    fn ui_popup_content(&mut self, ui: &mut Ui, _content_width: f32) { // content_width might be needed for inner layout later
        // --- Top Search Bar ---
        ui.horizontal(|ui| {
            ui.label("Search:");
            let search_response = ui.text_edit_singleline(&mut self.search);
            if ui.button("Search").clicked() || (search_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                if !self.search.trim().is_empty() { // Only search if not empty
                    self.loading_products = true;
                    self.error_message.clear(); // Clear previous errors
                    self.product_rows.clear(); // Clear previous results
                    self.fetch_products(); // Trigger fetch
                } else {
                     self.error_message = "Please enter a search term.".to_string();
                     self.product_rows.clear();
                }
            }
        });
        ui.separator();

        // --- Product Table ---
        egui::ScrollArea::vertical() // Usually only vertical scroll is needed for tables
            .auto_shrink([false, false]) // Prevent shrinkage
            .show(ui, |ui|
        {
            if !self.error_message.is_empty() {
                ui.colored_label(Color32::RED, &self.error_message); // Show errors prominently
            }

            if self.loading_products {
                ui.label("Loading products...");
                // Fetching is now triggered by the search button/enter key
            } else {
                 self.ui_product_display_table(ui); // Display products
            }
        });

        ui.separator();

        // --- Bottom Buttons ---
        ui.horizontal(|ui| {
            // Example: Add selected product (needs selection logic)
             // This button is currently non-functional without selection
             // ui.add_enabled(!self.product_rows.is_empty(), egui::Button::new("Add Selected"));

            // Simple button to add a default row (like original)
             if ui.button("Add New Blank Row").clicked() {
                self.rows.push(UIRow {
                    item: "New Item".to_string(),
                    price: 0.0,
                    quantity: 1,
                });
                 // Maybe close popup after adding?
                 // self.show_popup = false;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Close").clicked() {
                    self.show_popup = false; // Request closing the window
                }
            });
        });
    }

     /// Renders the table displaying the fetched products.
    fn ui_product_display_table(&self, ui: &mut Ui) {
        egui::Grid::new("product_table")
            .striped(true)
            // .min_col_width(content_width / 6.0) // Adjust as needed, maybe pass width in
            .num_columns(7) // Explicitly set column count
            .spacing(egui::Vec2::new(8.0, 4.0)) // Increased spacing
            .show(ui, |ui| {
                // Header row
                ui.label("Item").highlight();
                ui.label("Inventory").highlight();
                ui.label("Price").highlight();
                ui.label("Base Price").highlight();
                ui.label("Price/Per").highlight();
                ui.label("Base Price/Per").highlight();
                ui.label("Unit").highlight();
                ui.end_row();

                if self.product_rows.is_empty() && self.error_message.is_empty() {
                     ui.label("No products found matching your search.");
                     ui.end_row();
                 } else {
                    for product in &self.product_rows {
                        ui.label(product.name.as_deref().unwrap_or("N/A"));
                        ui.label(format!("{}", product.inventory_available.map_or("N/A".to_string(), |v| v.to_string())));
                        ui.label(format!("${:.2}", product.price.unwrap_or(0.0)));
                        ui.label(format!("${:.2}", product.base_price.unwrap_or(0.0)));
                        ui.label(format!("${:.2}", product.price_per.unwrap_or(0.0)));
                        ui.label(format!("${:.2}", product.base_price_per.unwrap_or(0.0)));
                        ui.label(product.unit_of_measure.as_deref().unwrap_or("N/A"));
                        ui.end_row();
                    }
                }
            });
    }

    /// Fetches products from the database based on the current search term.
    /// NOTE: This still uses block_on and will freeze the UI.
    /// For a real application, this should be done asynchronously.
    fn fetch_products(&mut self) {
        // Reset state before fetching
        self.product_rows.clear();
        self.error_message.clear();

        let pool_result = block_on(init_sql_connection());
        let pool = match pool_result {
            Ok(pool_val) => pool_val,
            Err(e) => {
                self.error_message = format!("Database connection error: {}", e);
                self.loading_products = false;
                return;
            }
        };

        // Use a clone of search for the query binding
        let search_term = format!("%{}%", self.search);

        // Execute the query (synchronously)
        let products_result = block_on(
            sqlx::query_as::<_, Product>(
                "SELECT name, inventoryAvailable, imageUrl, price, basePrice, pricePer, basePricePer, unitOfMeasure FROM products WHERE name LIKE ? ORDER BY basePricePer",
            )
            .bind(search_term) // Bind the cloned value
            .fetch_all(&pool),
        );

        match products_result {
            Ok(products) => {
                self.product_rows = products;
            }
            Err(e) => {
                self.error_message = format!("Error fetching products: {}", e);
            }
        }
        // Always set loading to false after attempt, regardless of success/error
        self.loading_products = false;
    }

    fn apply_styling(&self, ctx: &egui::Context) {
         ctx.style_mut(|style| {
            // Keep your styling customizations
            style.spacing.item_spacing = egui::Vec2::new(8.0, 4.0);
            style.spacing.button_padding = egui::Vec2::new(12.0, 6.0);
            style.spacing.window_margin = egui::Margin::symmetric(25.0, 30.0);
            // style.visuals.override_text_color = Some(self.app_colors.primary);
            // style.visuals.window_fill = self.app_colors.background; // Example using colors

            style.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::new(32.0, egui::FontFamily::Proportional),
            );
             style.text_styles.insert(
                egui::TextStyle::Button, // Style buttons
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            );

            // Example: Slightly rounder widgets
            // style.visuals.widgets.all.rounding = egui::Rounding::from(4.0);
            // style.visuals.window_rounding = egui::Rounding::from(6.0);
        });
    }
}


impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply custom styling first
        self.apply_styling(ctx);

        // --- Central Panel ---
        egui::CentralPanel::default().show(ctx, |ui| {
            let panel_width = ui.available_width();
            // Let's make the content width a bit smaller, maybe 60% or 50% as originally
            let content_width = panel_width * 0.6; // Adjust as needed

            // Use vertical_centered to stack elements vertically and center them horizontally
            ui.vertical_centered(|ui| {
                // Create the main content Frame INSIDE vertical_centered
                egui::Frame::default()
                    .inner_margin(egui::Margin::symmetric(20.0, 15.0)) // Adjust padding
                    // .fill(self.app_colors.background) // Optional: Set frame background
                    .show(ui, |ui| {
                        // Set the width *of the frame's content area*
                        ui.set_width(content_width);
                        ui.style_mut().spacing.item_spacing = vec2(10.0, 10.0); // Spacing inside frame

                        // --- Heading ---
                        ui.heading("Budget App");
                        ui.add_space(15.0); // Add space after heading

                        // --- Budget Table ---
                        // Pass the calculated content_width to the table function
                        self.ui_budget_table(ui, content_width);
                        ui.add_space(10.0); // Add space
                        ui.separator();
                        ui.add_space(10.0); // Add space

                        // --- Summary ---
                        self.ui_summary(ui);
                        ui.add_space(10.0); // Add space
                        ui.separator();
                        ui.add_space(15.0); // Add space

                        // --- Add Row Button ---
                        // Centering within the frame is implicit due to vertical_centered parent
                        // Use horizontal layout if you need multiple buttons side-by-side later
                        if ui.button("Add Product from DB").clicked() {
                            self.show_popup = true;
                             // Trigger initial load ONLY if the popup is opened AND there's no data/error yet
                             if self.product_rows.is_empty() && self.error_message.is_empty() && !self.search.is_empty() {
                                 self.loading_products = true;
                                 self.fetch_products();
                             } else if self.product_rows.is_empty() && self.error_message.is_empty() && self.search.is_empty() {
                                 // Maybe show a prompt to search if search is empty?
                                 // Or fetch everything? Be careful with fetching everything.
                                 // For now, let's only fetch if search term exists.
                             }
                        }
                        ui.add_space(10.0); // Add space at the bottom of the frame
                    }); // End Frame::show
            }); // End vertical_centered

            // --- Product Popup Window ---
            // This remains outside the vertical_centered block,
            // as it's a separate window managed by egui's layering.
            // Pass the *panel* width or a desired popup width, not necessarily content_width
            let popup_width = panel_width * 0.8; // Example: make popup wider
            self.ui_product_popup(ctx, popup_width);

        }); // End CentralPanel::show
    }
}

// ... (Keep the main function and other parts unchanged) ...

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Octillion Desktop",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::<MyApp>::default()
        }),
    )
}