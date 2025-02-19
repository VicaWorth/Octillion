// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui::{self, Color32, Visuals, Sense, Vec2};

struct AppColors {
    primary: Color32,
    secondary: Color32,
    background: Color32,
    accent: Color32,
    secondary_accent: Color32
}
// struct AppStyles {
    
// }

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

struct MyApp {
    name: String,
    age: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let app_colors = AppColors {
            primary: Color32::from_rgb(142, 202, 230),
            secondary: Color32::from_rgb(33, 158, 188),
            background: Color32::from_rgb(2, 48, 71),
            accent: Color32::from_rgb(255, 183, 3),
            secondary_accent: Color32::from_rgb(251, 133, 0)
        };

        let mut visuals = Visuals::dark();
        visuals.widgets.inactive.bg_fill = app_colors.primary;
        visuals.widgets.hovered.bg_fill = app_colors.secondary_accent;
        visuals.override_text_color = Some(app_colors.accent);
        ctx.set_visuals(visuals);
        egui::CentralPanel::default().frame(egui::Frame {
            fill: app_colors.background,
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Increment").clicked() {
                self.age += 1;
            }

            ui.label(format!("Hello '{}', age {}", self.name, self.age));

            // ui.image(egui::include_image!(
            //    "../../../crates/egui/assets/ferris.png"
            // ));

        });
    }
}