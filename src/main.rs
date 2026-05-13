#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod data;
mod theme;

fn main() -> eframe::Result<()> {
    let icon = make_icon();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Tra cứu Biểu thuế Xuất Nhập khẩu 2026")
            .with_inner_size([1360.0, 840.0])
            .with_min_inner_size([900.0, 600.0])
            .with_icon(std::sync::Arc::new(icon)),
        ..Default::default()
    };

    eframe::run_native(
        "Biểu Thuế XNK 2026",
        native_options,
        Box::new(|cc| {
            theme::apply_theme(&cc.egui_ctx);
            Ok(Box::new(app::BtApp::new(cc)))
        }),
    )
}

fn make_icon() -> egui::IconData {
    const S: usize = 32;
    let mut rgba = vec![0u8; S * S * 4];
    let cx = S as f32 / 2.0;
    let cy = S as f32 / 2.0;
    for y in 0..S {
        for x in 0..S {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let i = (y * S + x) * 4;
            if dx * dx + dy * dy <= 14.5 * 14.5 {
                rgba[i]     = 203; // Mauve #cba6f7
                rgba[i + 1] = 166;
                rgba[i + 2] = 247;
                rgba[i + 3] = 255;
            }
        }
    }
    egui::IconData { rgba, width: S as u32, height: S as u32 }
}
