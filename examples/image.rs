use neoui::*;

fn main() {
    defer_results!();

    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!("usage: cargo run --example image --features image -- <cover.jpg|cover.png>");
        return;
    };

    let cover = Image::open(path).unwrap();
    let mut app = ui("Image", 640, 480);

    while app.window.open() {
        app.frame(|ui| {
            let (width, height) = ui.window.content_size();
            let side = (width.min(height) as i32 - 48).max(1);

            ui.paint_image(
                Rect::new(24, 24, side, side),
                &cover,
                style().fit(ImageFit::Fixed).radius(16),
            );

            if ui.window.pressed(Key::Escape) {
                ui.window.close();
            }
        });
    }
}
