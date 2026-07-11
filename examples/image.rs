use neoui::*;

fn main() -> Result<(), String> {
    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!("usage: cargo run --example image --features image -- <cover.jpg|cover.png>");
        return Ok(());
    };
    let cover = Image::open(path)?;
    let mut app = ui("NeoUI image", 640, 480);
    while app.window.open() {
        app.frame(|ui| {
            let (width, height) = ui.window.content_size();
            let side = (width.min(height) as i32 - 48).max(1);
            ui.paint_image(
                Rect::new(24, 24, side, side),
                &cover,
                ImageStyle {
                    fit: ImageFit::Cover,
                    radius: 16,
                    ..Default::default()
                },
            );
            if ui.window.pressed(Key::Escape) {
                ui.window.close();
            }
        });
    }
    Ok(())
}
