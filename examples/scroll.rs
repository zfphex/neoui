use neoui::*;

fn main() {
    let mut ui = ui("Test", 1000, 700);

    let mut scroll_y = 18;

    while ui.window.open() {
        ui.frame(|ui| {
            let bounds = ui.current_frame().bounds;
            ui.scroll_view(bounds, &mut scroll_y, |ui| {
                for _ in 0..100 {
                    ui.text("test", style());
                }
            });

            if ui.window.pressed(Key::Escape) {
                ui.window.close();
            }
        });
    }
}
