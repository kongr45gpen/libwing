use eframe::egui;
use libwing::{WingConsole, WingResponse, Meter};
use std::time::Duration;

fn main() -> Result<(), libwing::Error> {
    let consoles = WingConsole::scan(true)?;
    if consoles.is_empty() {
        eprintln!("No Wing consoles found");
        std::process::exit(1);
    }

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 400.0)),
        ..Default::default()
    };

    let mut wing = WingConsole::connect(&consoles[0].ip)?;
    
    // Request meters for first 32 channels
    let meters: Vec<Meter> = (1..=32).map(|i| Meter {
        id: i,              // Channel number
        port: 0,           // Will be set by request_meter
        interval: 50,      // Update every 50ms
    }).collect();

    let port = wing.request_meter(&meters)?;

    // Store meter data in app state
    let app = WingMetersApp {
        wing,
        port,
        meter_values: vec![0.0; 32],
    };

    eframe::run_native(
        "Wing Meters",
        options,
        Box::new(|_cc| Box::new(app)),
    ).unwrap();

    Ok(())
}

struct WingMetersApp {
    wing: WingConsole,
    port: u16,
    meter_values: Vec<f32>,
}

impl eframe::App for WingMetersApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Read meter values
        if let Ok((_, values)) = self.wing.read_meters() {
            for (i, &value) in values.iter().enumerate() {
                if i < self.meter_values.len() {
                    // Convert from dB to normalized value (0.0 to 1.0)
                    let db = value as f32 / 256.0;
                    self.meter_values[i] = (db + 60.0) / 60.0; // Assuming -60dB to 0dB range
                    self.meter_values[i] = self.meter_values[i].clamp(0.0, 1.0);
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Channel Meters");
            
            for (i, &value) in self.meter_values.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("Ch {}: ", i + 1));
                    let meter_height = 20.0;
                    let meter_width = ui.available_width() - 50.0;
                    
                    let rect = ui.allocate_space(egui::vec2(meter_width, meter_height));
                    let meter_rect = egui::Rect::from_min_size(
                        rect.min,
                        egui::vec2(meter_width * value, meter_height),
                    );
                    
                    // Draw background
                    ui.painter().rect_filled(
                        rect.rect,
                        0.0,
                        egui::Color32::from_gray(64),
                    );
                    
                    // Draw meter value
                    let color = if value > 0.9 {
                        egui::Color32::RED
                    } else if value > 0.7 {
                        egui::Color32::YELLOW
                    } else {
                        egui::Color32::GREEN
                    };
                    
                    ui.painter().rect_filled(
                        meter_rect,
                        0.0,
                        color,
                    );
                });
            }
        });

        // Request continuous updates
        ctx.request_repaint_after(Duration::from_millis(50));
    }
}
