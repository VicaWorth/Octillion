#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui::{self, Color32, Ui, ColorImage, TextureHandle};
use egui::vec2;

use egui_extras::{TableBuilder, Column};

use futures::executor::block_on;

use egui::{Align, Layout, TextFormat, TextStyle, Widget, WidgetText}; // Import Align and Layout for cell alignment
use egui::text::LayoutJob; // For potential

use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;
use dotenvy::dotenv;
use std::default::Default;
use std::env;
use std::collections::{HashMap};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Cursor;

use tokio::sync::mpsc;

use reqwest; // Keep reqwest import
use path_clean::PathClean;
use url::Url;

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
    image_url: Option<String>,
    price: Option<f64>,
    base_price: Option<f64>,
    base_price_per: Option<f64>,
    price_per: Option<f64>,
    unit_of_measure: Option<String>,
}

#[derive(Debug, Clone)]
struct UIRow {
    item: String,
    price: f32,
    quantity: i32,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum SortColumn {
    Name,
    Inventory,
    Price,
    BasePrice,
    PricePer,
    BasePricePer,
    Unit,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum SortDirection {
    Ascending,
    Descending,
}

// Helper function for comparing Option<T> where T: PartialOrd
// This matches MySQL's NULL sorting (NULLs first in ASC) and Option::cmp for Ord types.
fn compare_options<T: PartialOrd>(a: &Option<T>, b: &Option<T>, none_is_less: bool) -> std::cmp::Ordering {
    match (a, b) {
        (Some(val_a), Some(val_b)) => val_a.partial_cmp(val_b).unwrap_or(std::cmp::Ordering::Equal),
        (Some(_), None) => if none_is_less { std::cmp::Ordering::Greater } else { std::cmp::Ordering::Less }, // If None is less, Some is Greater
        (None, Some(_)) => if none_is_less { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater },    // If None is less, None is Less
        (None, None) => std::cmp::Ordering::Equal,
    }
}

// Enum for Image Status
// Cannot derive Debug because TextureHandle doesn't implement it.
#[derive(Clone)]
enum ImageStatus {
    // Idle,
    Downloading,
    Loaded(TextureHandle),
    Error(String),
}

struct ImageMessage {
    url: String,
    result: Result<(PathBuf, ColorImage), String>,
}

// In struct MyApp:
struct MyApp {
    budget_header_text: String,
    rows: Vec<UIRow>,
    product_rows: Vec<Product>,
    show_popup: bool,
    loading_products: bool,
    search: String,
    error_message: String,
    selected_product_index: Option<usize>,
    image_cache: HashMap<String, ImageStatus>,
    image_tx: mpsc::Sender<ImageMessage>,
    image_rx: mpsc::Receiver<ImageMessage>,
    // Add these fields for sorting state
    active_sort_column: Option<SortColumn>,
    sort_direction: SortDirection,
}

impl Default for MyApp {
    fn default() -> Self {
        let (image_tx, image_rx) = mpsc::channel(100);
        Self {
            budget_header_text: "Budget".to_string(),
            rows: vec![],
            product_rows: vec![],
            show_popup: false,
            loading_products: false,
            search: String::new(),
            error_message: String::new(),
            selected_product_index: None,
            image_cache: HashMap::new(),
            image_tx,
            image_rx,
            // Initialize sorting state
            active_sort_column: None, // No column sorted by default
            sort_direction: SortDirection::Ascending,
        }
    }
}

impl MyApp {
    fn add_product_to_budget(&mut self) {
        if let Some(index) = self.selected_product_index {
            if let Some(product) = self.product_rows.get(index) {
                self.rows.push(UIRow {
                    item: product.name.clone().unwrap_or_else(|| "N/A".to_string()),
                    price: product.price.unwrap_or(0.0) as f32,
                    quantity: 1,
                });
                self.selected_product_index = None;
            } else {
                eprintln!("Error: Selected index {} is out of bounds.", index);
                self.selected_product_index = None;
            }
        }
    }

    fn ui_budget_table(&mut self, ui: &mut Ui) {
        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
        let row_height = text_height * 1.8;

        TableBuilder::new(ui)
        .striped(true)
        .resizable(false) 
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::remainder().at_least(150.0)) // Item
        .column(Column::initial(100.0).at_least(80.0).at_most(200.0)) // Price
        .column(Column::initial(100.0).at_least(80.0).at_most(150.0))// Quantity:
        .column(Column::initial(40.0).at_least(30.0).at_most(60.0))// Delete
        .header(text_height * 1.2, |mut header| {
            header.col(|ui| { ui.strong("Item"); });
            header.col(|ui| { ui.strong("Price"); });
            header.col(|ui| { ui.strong("Quantity"); });
            // Add padding or adjust alignment if the header text itself feels cramped
            header.col(|ui| { ui.add_space(5.0); ui.strong("Delete"); }); // Example padding
        })
            .body(|mut body| {
                let mut delete_index = None;
                for (i, row_data) in self.rows.iter_mut().enumerate() {
                    body.row(row_height, |mut row| {
                        row.col(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut row_data.item).desired_width(f32::INFINITY));
                        });
                        row.col(|ui| {
                            ui.add(egui::DragValue::new(&mut row_data.price).speed(0.1).prefix("$").max_decimals(2).min_decimals(2));
                        });
                        row.col(|ui| {
                             ui.add(egui::DragValue::new(&mut row_data.quantity).speed(0.1).clamp_range(0..=999));
                        });
                        row.col(|ui| {
                            if ui.button("X").clicked() {
                                delete_index = Some(i);
                            }
                        });
                    });
                }
                 // Perform deletion outside the loop to avoid borrow issues
                 if let Some(index) = delete_index {
                     self.rows.remove(index);
                 }
            });
    }

    fn sort_product_rows(&mut self) {
        if let Some(sort_column) = self.active_sort_column {
            let direction = self.sort_direction;
            // `none_is_less = true` means None values come before Some values in ascending sort.
            // This is consistent with Option::cmp for types that implement Ord,
            // and matches typical SQL NULL sorting (NULLs first in ASC).
            let none_is_less = true;

            self.product_rows.sort_unstable_by(|a, b| {
                let ordering = match sort_column {
                    SortColumn::Name => a.name.cmp(&b.name),
                    SortColumn::Inventory => a.inventory_available.cmp(&b.inventory_available),
                    SortColumn::Price => compare_options(&a.price, &b.price, none_is_less),
                    SortColumn::BasePrice => compare_options(&a.base_price, &b.base_price, none_is_less),
                    SortColumn::PricePer => compare_options(&a.price_per, &b.price_per, none_is_less),
                    SortColumn::BasePricePer => compare_options(&a.base_price_per, &b.base_price_per, none_is_less),
                    SortColumn::Unit => a.unit_of_measure.cmp(&b.unit_of_measure),
                };

                match direction {
                    SortDirection::Ascending => ordering,
                    SortDirection::Descending => ordering.reverse(),
                }
            });
            // Important: Clear selection as row indices have changed
            self.selected_product_index = None;
        }
    }

    fn ui_summary(&self, ui: &mut Ui) {
        let total_price: f32 = self.rows.iter().map(|row| row.price * row.quantity as f32).sum();
        egui::Grid::new("summary_table").num_columns(2).show(ui, |ui| {
            ui.label("Total:");
            ui.label(format!("${:.2}", total_price));
            ui.end_row();
        });
    }

    fn ui_product_popup(&mut self, ctx: &egui::Context, content_width: f32) {
        if !self.show_popup { return; }
        let mut open = self.show_popup;
        egui::Window::new("Safeway Products")
            .open(&mut open)
            .resizable(true)
            .default_width(content_width * 1.2)
            .default_height(400.0)
            .min_height(250.0)
            .show(ctx, |ui| {
                egui::TopBottomPanel::top("popup_search_panel")
                    .frame(egui::Frame::default().inner_margin(egui::Margin::same(5.0)))
                    .show_inside(ui, |ui| {
                        self.ui_popup_search_bar(ui);
                        ui.separator();
                    });
                egui::TopBottomPanel::bottom("popup_buttons_panel")
                    .frame(egui::Frame::default().inner_margin(egui::Margin::symmetric(10.0, 5.0)))
                    .show_inside(ui, |ui| {
                        ui.separator();
                        self.ui_popup_bottom_bar(ui);
                    });
                egui::CentralPanel::default()
                    .frame(egui::Frame::default().inner_margin(egui::Margin::same(5.0)))
                    .show_inside(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                self.ui_popup_results_area(ui, ctx);
                            });
                    });
            });
        self.show_popup = open;
        if !self.show_popup { self.selected_product_index = None; }
    }

    fn ui_popup_search_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Search:");
            let search_response = ui.add(egui::TextEdit::singleline(&mut self.search).desired_width(ui.available_width() * 0.5));
            let enter_pressed = search_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let search_clicked = ui.button("Search").clicked();
            let load_all_clicked = ui.button("Load All").clicked();

            if search_clicked || enter_pressed {
                if !self.search.trim().is_empty() {
                    self.fetch_products();
                } else {
                    self.error_message = "Please enter a search term.".to_string();
                    self.product_rows.clear();
                    // Keep already loaded images in cache when clearing search
                    self.image_cache.retain(|_, status| matches!(status, ImageStatus::Loaded(_)));
                    self.selected_product_index = None;
                }
            } else if load_all_clicked {
                 self.search.clear(); // Ensure search term is cleared for load all
                 self.fetch_products();
            }

            // Optional clear button
            if !self.search.is_empty() && ui.button("Clear").clicked() {
                self.search.clear();
                self.product_rows.clear();
                self.error_message.clear();
                self.selected_product_index = None;
                self.image_cache.retain(|_, status| matches!(status, ImageStatus::Loaded(_)));
            }
        });
    }

     fn ui_popup_results_area(&mut self, ui: &mut Ui, ctx: &egui::Context) {
         if !self.error_message.is_empty() {
             ui.colored_label(Color32::RED, &self.error_message);
             ui.separator();
         }
         if self.loading_products {
             ui.horizontal(|ui| {
                 ui.spinner();
                 ui.label("Loading product data...");
             });
         } else {
             self.ui_product_display_table(ui, ctx);
         }
     }

     fn ui_popup_bottom_bar(&mut self, ui: &mut Ui) {
         ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                 if ui.button("Add New Blank Row").clicked() {
                     self.rows.push(UIRow { item: "New Item".to_string(), price: 0.0, quantity: 1 });
                 }
                 ui.add_space(20.0); // Space between buttons
                 let is_product_selected = self.selected_product_index.is_some();
                 let add_button_resp = ui.add_enabled(is_product_selected, egui::Button::new("Add Selected to Budget"));
                 if add_button_resp.clicked() { self.add_product_to_budget(); }
                 // Display selected item name for user feedback
                 if let Some(index) = self.selected_product_index {
                     if let Some(product) = self.product_rows.get(index) {
                         let name = product.name.as_deref().unwrap_or("N/A");
                         let max_len = 30; // Limit display length
                         let truncated_name = if name.len() > max_len { format!("{}...", &name[..max_len]) } else { name.to_string() };
                         ui.label(format!("Selected: {}", truncated_name)).on_hover_text(name); // Show full name on hover
                     }
                 }
            });
         });
     }

    /// Renders the table displaying the fetched products, now with selection and images.
    /// Renders the table displaying the fetched products, now with selection, images, and sorting.
    fn ui_product_display_table(&mut self, ui: &mut Ui, ctx: &egui::Context) { // Changed _ctx to ctx
        let image_size = egui::vec2(80.0, 80.0);
        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
        let row_height = image_size.y.max(text_height * 1.5) + ui.style().spacing.item_spacing.y; // Dynamic row height

        // Temporary variables to store interaction results from headers
        let mut sort_criteria_changed_flag = false;
        let mut temp_active_sort_col = self.active_sort_column;
        let mut temp_sort_dir = self.sort_direction;

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::exact(image_size.x + 10.0).at_least(image_size.x)) // Image column + padding
            .column(Column::remainder().at_least(80.0)) // Item Name (Resizable)
            .column(Column::initial(90.0).at_least(70.0)) // Inventory
            .column(Column::initial(80.0).at_least(60.0)) // Price
            .column(Column::initial(80.0).at_least(60.0)) // Base Price
            .column(Column::initial(80.0).at_least(60.0)) // Price/Per
            .column(Column::initial(90.0).at_least(70.0)) // Base Price/Per
            .column(Column::initial(60.0).at_least(40.0)) // Unit
            .min_scrolled_height(200.0) // Ensure table area has min height
            .header(20.0, |mut header| {
                // Helper closure to create sortable header cells
                // This closure captures temp_active_sort_col, temp_sort_dir, and sort_criteria_changed_flag mutably.
                let mut sort_header_button =
                    |ui: &mut egui::Ui, label: &str, column_type: SortColumn| {
                    let mut button_text = label.to_string();
                    if temp_active_sort_col == Some(column_type) {
                        // Corrected part: use push_str for string slices
                        let suffix = match temp_sort_dir {
                            SortDirection::Ascending => " ^", // Now a string slice
                            SortDirection::Descending => " v", // Now a string slice
                        };
                        button_text.push_str(suffix); // Use push_str
                    }
                    // Use a button for the header. Styling could make it look more like a typical table header.
                    if ui.button(button_text).clicked() {
                        if temp_active_sort_col == Some(column_type) {
                            // Clicked on the same column, toggle direction
                            temp_sort_dir = match temp_sort_dir {
                                SortDirection::Ascending => SortDirection::Descending,
                                SortDirection::Descending => SortDirection::Ascending,
                            };
                        } else {
                            // Clicked on a new column, sort ascending by default
                            temp_active_sort_col = Some(column_type);
                            temp_sort_dir = SortDirection::Ascending;
                        }
                        sort_criteria_changed_flag = true; // Signal that sort criteria have changed
                    }
                };

                header.col(|ui| { ui.strong("Image"); }); // Image column not sortable
                header.col(|ui| sort_header_button(ui, "Item", SortColumn::Name));
                header.col(|ui| sort_header_button(ui, "Inventory", SortColumn::Inventory));
                header.col(|ui| sort_header_button(ui, "Price", SortColumn::Price));
                header.col(|ui| sort_header_button(ui, "Base Price", SortColumn::BasePrice));
                header.col(|ui| sort_header_button(ui, "Price / Unit", SortColumn::PricePer));
                header.col(|ui| sort_header_button(ui, "Base / Unit", SortColumn::BasePricePer));
                header.col(|ui| sort_header_button(ui, "Unit", SortColumn::Unit));
            })
            .body(|mut body| {
                // Handle empty state display within the table body
                if self.product_rows.is_empty() {
                    if !self.loading_products && self.error_message.is_empty() {
                         body.row(row_height, |mut row| {
                            row.col(|ui| { ui.label(" "); }); // Empty cell for image
                            // Span label across multiple columns visually if needed, or just put in main column
                            row.col(|ui| { ui.label("No products found matching your search criteria."); });
                            for _ in 0..6 { row.col(|_ui| {}); } // Fill remaining columns
                        });
                    } else if self.error_message.is_empty() { // Still loading
                         body.row(row_height, |mut row| {
                             // Indicate loading within the table area
                             row.col(|ui| { ui.spinner(); });
                             row.col(|ui| { ui.label("Loading..."); });
                             for _ in 0..6 { row.col(|_ui| {}); }
                         });
                    }
                    // If error_message is set, it's handled above the table
                } else {
                    // Clone data needed inside the loop to avoid borrow issues if not sorting live
                    // Since sorting happens before this, direct iteration is fine.
                    let products_view = &self.product_rows; // Iterate over the (potentially sorted) rows
                    let image_cache_clone = self.image_cache.clone();

                    for (index, product) in products_view.iter().enumerate() {
                        body.row(row_height, |mut row| {
                            row.col(|ui| { // Image Column
                                if let Some(url) = &product.image_url {
                                    if !url.is_empty() {
                                        match image_cache_clone.get(url) {
                                            Some(ImageStatus::Loaded(texture)) => {
                                                let img_src = (texture.id(), image_size);
                                                let img = egui::Image::new(img_src);
                                                ui.add(img);
                                            }
                                            Some(ImageStatus::Downloading) => {
                                                ui.add_sized(image_size, egui::Spinner::new());
                                            }
                                            Some(ImageStatus::Error(err_msg)) => {
                                                let response = ui.add_sized(image_size, egui::Label::new("⚠️").wrap(false));
                                                response.on_hover_text(err_msg);
                                            }
                                            _ => { // None (download not started) or Idle
                                                ui.add_sized(image_size, egui::Label::new("-").wrap(false));
                                            }
                                        }
                                    } else { // URL string is empty
                                        ui.add_sized(image_size, egui::Label::new("N/A").wrap(false));
                                    }
                                } else { // No URL field in product data
                                     ui.add_sized(image_size, egui::Label::new("N/A").wrap(false));
                                }
                            });

                            // --- Item Name Column (Selectable + Truncate) ---
                            row.col(|ui| {
                                let item_label_str = product.name.as_deref().unwrap_or("N/A");
                                let is_selected = self.selected_product_index == Some(index);

                                let (rect, response) = ui.allocate_at_least(
                                    ui.available_size_before_wrap(),
                                    egui::Sense::click()
                                );

                                if response.clicked() {
                                    // Toggle selection on click
                                    if self.selected_product_index == Some(index) {
                                        // self.selected_product_index = None; // Optional: deselect on click
                                    } else {
                                        self.selected_product_index = Some(index);
                                    }
                                    // No need to request repaint here for selection, as sort might do it.
                                    // ui.ctx().request_repaint(); // If only selection changes
                                }

                                if is_selected {
                                    ui.painter().rect_filled(
                                        rect,
                                        egui::Rounding::none(),
                                        ui.visuals().selection.bg_fill,
                                    );
                                }
                                let mut cell_ui = ui.child_ui(rect, *ui.layout());
                                let label_response = cell_ui.add(
                                    egui::Label::new(item_label_str)
                                        .truncate(true) // Enable truncation "..."
                                );

                                // Add hover text to show the full name if truncated.
                                label_response.on_hover_text(item_label_str);
                            });

                            // Other Columns
                             row.col(|ui| {
                                ui.label(format!("{}", product.inventory_available.map_or_else(
                                    || "N/A".to_string(),
                                    // Use shorter labels for inventory status
                                    |v| match v { 0 => "Out".to_string(), 1 => "In".to_string(), _ => "N/A".to_string() }
                                )));
                             });
                             row.col(|ui| { ui.label(format!("${:.2}", product.price.unwrap_or(0.0))); });
                             row.col(|ui| { ui.label(format!("${:.2}", product.base_price.unwrap_or(0.0))); });
                             row.col(|ui| { ui.label(format!("${:.2}", product.price_per.unwrap_or(0.0))); });
                             row.col(|ui| { ui.label(format!("${:.2}", product.base_price_per.unwrap_or(0.0))); });
                             row.col(|ui| { ui.label(product.unit_of_measure.as_deref().unwrap_or("N/A")); });

                        }); // End Row
                    } // End loop
                } // End else (product_rows not empty)
            }); // End body

        // After the table is built, if sort criteria changed, update self and re-sort.
        if sort_criteria_changed_flag {
            self.active_sort_column = temp_active_sort_col;
            self.sort_direction = temp_sort_dir;
            self.sort_product_rows(); // This will also clear selected_product_index
            ctx.request_repaint(); // Request repaint as data order has changed
        }
    }

    /// Fetches product metadata synchronously, then spawns async tasks for image downloads.
    fn fetch_products(&mut self) {
        self.loading_products = true; // Indicate loading started
        self.product_rows.clear();
        self.error_message.clear();
        self.selected_product_index = None;

        // HACK: Create temporary context to request repaint during potentially long fetch.
        // Ideally, context would be passed or accessed more directly if needed mid-operation.
        let ctx = egui::Context::default(); // Keep this for repaint requests if needed
        ctx.request_repaint(); // Request redraw to show loading state

        // Block on DB operations (simplifies state management compared to fully async fetch)
        match block_on(init_sql_connection()) {
             Ok(pool) => {
                 let search_term = if self.search.is_empty() { "%".to_string() } else { format!("%{}%", self.search) };
                 // Fetch product data from DB
                 match block_on(sqlx::query_as::<_, Product>(
                     "SELECT name, inventoryAvailable, imageUrl, price, basePrice, pricePer, basePricePer, unitOfMeasure FROM products WHERE name LIKE ? ORDER BY basePricePer",
                 ).bind(search_term).fetch_all(&pool)) {
                     Ok(products) => {
                         self.product_rows = products; // Store fetched products
                         // Iterate through products and spawn download tasks for missing images
                         for product in &self.product_rows {
                             if let Some(url) = &product.image_url {
                                 if !url.trim().is_empty() {
                                     // Check cache: Only spawn download if not already loaded, error, or downloading
                                     match self.image_cache.get(url) {
                                         Some(ImageStatus::Loaded(_)) | Some(ImageStatus::Error(_)) | Some(ImageStatus::Downloading) => {
                                             // Already handled or in progress, skip
                                         }
                                         _ => { // Status is Idle or image is not in cache (None)
                                             // Mark as Downloading immediately (in UI thread)
                                             self.image_cache.insert(url.clone(), ImageStatus::Downloading);
                                             // Clone necessary variables for the async task
                                             let tx_clone = self.image_tx.clone();
                                             let url_clone = url.clone();
                                             // Spawn the asynchronous download task
                                             tokio::spawn(async move {
                                                 download_and_process_image(url_clone, tx_clone).await;
                                             });
                                         }
                                     }
                                 }
                             }
                         }
                         // Optional: Set message if search returned no results (handled in table display)
                         // if self.product_rows.is_empty() {
                         //      self.error_message = "No products found matching your search.".to_string();
                         // }
                     }
                     Err(e) => { self.error_message = format!("Error fetching products: {}", e); }
                 }
             }
             Err(e) => { self.error_message = format!("Database connection error: {}", e); }
         }

        // Re-apply current sort if one is active
        if self.active_sort_column.is_some() {
            self.sort_product_rows();
        }

        self.loading_products = false; // Indicate loading finished
        ctx.request_repaint(); // Request redraw to show results/error
    }
    
    fn apply_styling(&self, ctx: &egui::Context) {
         ctx.style_mut(|style| {
            style.spacing.item_spacing = egui::vec2(8.0, 4.0);
            style.spacing.button_padding = egui::vec2(12.0, 6.0);
            style.spacing.window_margin = egui::Margin::symmetric(25.0, 30.0);

            // Define text styles
            style.text_styles = [
                (egui::TextStyle::Heading, egui::FontId::new(32.0, egui::FontFamily::Proportional)),
                (egui::TextStyle::Body, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
                (egui::TextStyle::Button, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
            ]
            .into();

            // Set rounding for inactive widgets (adjust other states if needed)
            style.visuals.widgets.inactive.rounding = egui::Rounding::from(4.0);
            style.visuals.window_rounding = egui::Rounding::from(6.0);
        });
    }
} // End impl MyApp

impl eframe::App for MyApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            // Process incoming image download results from the channel
            while let Ok(msg) = self.image_rx.try_recv() {
                 match msg.result {
                     Ok((_path, color_image)) => {
                         // Load texture into egui (handles deduplication internally)
                         let texture_options = egui::TextureOptions {
                             magnification: egui::TextureFilter::Linear,
                             minification: egui::TextureFilter::Linear,
                         };
                         let texture: TextureHandle = ctx.load_texture(
                             &msg.url,           // Name/ID for the texture manager
                             color_image,        // Image data
                             texture_options     // Texture options
                         );
                         eprintln!("Loaded/updated texture handle for: {}", msg.url);
                         // Update cache with the loaded texture handle
                         // Cloning msg.url is necessary as msg is consumed by the loop
                         self.image_cache.insert(msg.url.clone(), ImageStatus::Loaded(texture));
                     }
                     Err(e) => {
                         eprintln!("Received error for image {}: {}", msg.url, e);
                         // Update cache with error status, using entry API for safety
                         self.image_cache.entry(msg.url.clone()) // Clone msg.url for cache key
                             .and_modify(|status| {
                                 // Only update if not already successfully loaded
                                 if !matches!(status, ImageStatus::Loaded(_)) {
                                     *status = ImageStatus::Error(e.clone());
                                 }
                                 })
                             // If entry doesn't exist, insert the error status
                             .or_insert(ImageStatus::Error(e));
                     }
                 }
                 // Request repaint after processing a message to update UI
                 ctx.request_repaint();
             }

            self.apply_styling(ctx); // Apply custom visual styles

            // Main application layout
            egui::CentralPanel::default().show(ctx, |ui| {
                let panel_width = ui.available_width();
                // Calculate content width with a minimum size
                let content_width = (panel_width * 0.6).max(400.0);
                // Center the main budget section vertically
                ui.vertical_centered(|ui| {
                    egui::Frame::default() // Frame for the budget section
                        .inner_margin(egui::Margin::symmetric(20.0, 15.0))
                        .show(ui, |ui| {
                            ui.set_width(content_width); // Constrain width
                            ui.heading(&self.budget_header_text);
                            self.ui_budget_table(ui); // Display the budget items table
                            ui.separator();
                            self.ui_summary(ui); // Display the total summary
                            ui.separator();

                            // Button to open the product search popup
                            if ui.button("Add Product from DB").clicked() {
                                self.show_popup = true;
                                self.fetch_products(); // Fetch data when button is clicked
                            }
                            ui.add_space(10.0);
                        });
                });
                // Display the product popup window if show_popup is true
                let popup_width = (panel_width * 0.8).max(600.0); // Calculate popup width
                self.ui_product_popup(ctx, popup_width); // Pass context to popup
            });
        } // End update
} // End impl eframe::App

/// Generates a safe local file path within a base directory for a given URL.
fn get_local_image_path(absolute_base_dir: &Path, url_str: &str) -> Result<PathBuf, String> {
    let url = Url::parse(url_str).map_err(|e| format!("Invalid URL '{}': {}", url_str, e))?;
    // Extract filename from URL path segments
    let filename = url.path_segments()
        .and_then(|s| s.last()) // Get last segment
        .filter(|n| !n.is_empty()) // Ensure it's not empty
        .unwrap_or("unknown_image.bin"); // Provide a fallback

    // Sanitize filename to allowed characters
    let safe_filename: String = filename.chars()
        .filter(|c| c.is_alphanumeric() || matches!(*c, '.' | '-' | '_'))
        .collect();

    // Prevent potentially problematic filenames
    if safe_filename.is_empty() || safe_filename == "." || safe_filename == ".." {
        return Err(format!("Could not generate safe filename for {}", url_str));
    }

    // Combine base directory and safe filename
    let combined = absolute_base_dir.join(&safe_filename);
    // Clean the path (e.g., resolve ., .., normalize separators)
    let cleaned_path = combined.clean();

    // Security check: Ensure the cleaned path is still within the intended base directory
    if !cleaned_path.starts_with(absolute_base_dir) {
         return Err(format!(
             "Potential path traversal detected after cleaning. Base: '{}', Cleaned: '{}'",
             absolute_base_dir.display(),
             cleaned_path.display()
         ));
    }
    Ok(cleaned_path)
}

/// Decodes image data from byte slice into egui::ColorImage.
fn load_image_from_bytes(bytes: &[u8]) -> Result<ColorImage, String> {
    // Use image crate with explicit namespace if needed
    use image::io::Reader as ImageReader;
    let img = ImageReader::new(Cursor::new(bytes)) // Read from memory buffer
               .with_guessed_format() // Try to auto-detect format (png, jpg, etc.)
               .map_err(|e| format!("Guess format failed: {}", e))?
               .decode() // Decode the image data
               .map_err(|e| format!("Decode failed: {}", e))?;

    // Convert the decoded image to RGBA8 format for egui
    let size = [img.width() as _, img.height() as _];
    let rgba_image = img.to_rgba8();
    // Get raw pixel data
    let pixels = rgba_image.as_flat_samples();
    // Create egui ColorImage (ensure correct color format, e.g., unmultiplied alpha)
    Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}

/// Reads an image file from disk and decodes it into egui::ColorImage.
fn load_image_from_disk(path: &Path) -> Result<ColorImage, String> {
    eprintln!("Attempting to load image from disk: {}", path.display());
    // Read file bytes
    let image_bytes = fs::read(path)
        .map_err(|e| format!("Read failed '{}': {}", path.display(), e))?;
    // Check if file is empty
    if image_bytes.is_empty() {
        return Err(format!("Image file '{}' is empty.", path.display()));
    }
    eprintln!("Read {} bytes from disk for: {}", image_bytes.len(), path.display());
    // Decode bytes using the helper function
    load_image_from_bytes(&image_bytes)
        .map_err(|e| format!("Processing failed '{}': {}", path.display(), e))
}

/// Asynchronously downloads an image, saves it locally, decodes it,
/// and sends the result back via an MPSC channel.
async fn download_and_process_image(
    url: String,
    tx: mpsc::Sender<ImageMessage>,
) {
    // Calculate absolute path for the image cache directory
    let absolute_image_dir = match std::env::current_dir() {
         Ok(cwd) => cwd.join("product_images"),
         Err(e) => {
             let err_msg = format!("Failed to get current working directory: {}", e);
             eprintln!("{}", err_msg);
             // Send error back if CWD fails
             let _ = tx.send(ImageMessage { url, result: Err(err_msg) }).await;
             return;
         }
    };

    // Ensure the image directory exists
    if let Err(e) = fs::create_dir_all(&absolute_image_dir) {
        eprintln!("Failed to create image directory: {}", e);
        let _ = tx.send(ImageMessage { url, result: Err(format!("Failed to create image directory '{}': {}", absolute_image_dir.display(), e)) }).await;
        return;
    }

    // Get the safe local path for the image file
    let local_path = match get_local_image_path(&absolute_image_dir, &url) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Failed to get local path for {}: {}", url, e);
            let _ = tx.send(ImageMessage { url, result: Err(e) }).await;
            return;
        }
    };

    // Check if image already exists locally and try to load it
    if local_path.exists() {
        match load_image_from_disk(&local_path) {
            Ok(color_image) => {
                eprintln!("Loaded existing image from disk: {}", local_path.display());
                // Send success message with existing image data
                let _ = tx.send(ImageMessage { url, result: Ok((local_path, color_image)) }).await;
                return; // Don't proceed to download if loaded successfully
            }
            Err(e) => {
                // Log error loading existing file, but proceed to download (file might be corrupt)
                eprintln!("Failed loading existing image '{}': {}. Attempting download.", local_path.display(), e);
            }
        }
    }

    // If not found locally or loading failed, proceed to download, save, and decode
    let download_result: Result<(PathBuf, ColorImage), String> = async {
        eprintln!("Downloading image from: {}", url);
        let client = reqwest::Client::new(); // Create HTTP client
        // Send GET request
        let response = client.get(&url).send().await.map_err(|e| e.to_string())?;

        // Check for HTTP errors
        if !response.status().is_success() {
            return Err(format!("HTTP error {} fetching {}", response.status(), url));
        }
        // Read response body as bytes
        let image_bytes = response.bytes().await.map_err(|e| e.to_string())?;
        eprintln!("Downloaded {} bytes for: {}", image_bytes.len(), url);

        // Save bytes to local file
        fs::write(&local_path, &image_bytes).map_err(|e| e.to_string())?;
        eprintln!("Saved image to: {}", local_path.display());

        // Decode the downloaded bytes
        let color_image = load_image_from_bytes(&image_bytes)?;
        eprintln!("Decoded image successfully: {}", url);

        // Return the path and decoded image data
        Ok((local_path, color_image))

    }.await; // Execute the async block

    // Send the final result (Ok or Err) back to the UI thread via the channel
    match download_result {
        Ok(data) => { // data is Ok((local_path, color_image))
            let _ = tx.send(ImageMessage { url, result: Ok(data) }).await;
        }
        Err(e) => { // An error occurred during download/save/decode
             let error_string = format!("Failed download/process {}: {}", url, e);
             eprintln!("{}", error_string);
             // Optionally attempt cleanup of failed download (path ownership makes this tricky)
             // fs::remove_file(&local_path).ok();
             let _ = tx.send(ImageMessage { url, result: Err(error_string) }).await;
        }
    }
}

// Main application entry point
#[tokio::main] // Use tokio runtime
async fn main() -> Result<(), eframe::Error> {
    // Optional: Setup logging (e.g., env_logger::init(); )

    // Configure native window options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0]) // Initial size
            .with_min_inner_size([800.0, 600.0]), // Minimum size
        ..Default::default()
    };

    let app = MyApp::default(); // Create the application state

    // Run the eframe application
    eframe::run_native(
        "Octillion Desktop Budget", // Window title
        options,
        // Closure to create the app instance
        Box::new(|cc| {
            // Install image loaders (useful for other potential image formats/sources)
            egui_extras::install_image_loaders(&cc.egui_ctx);
            // Return the created app state wrapped in a Box
            Box::new(app)
        }),
    )
}