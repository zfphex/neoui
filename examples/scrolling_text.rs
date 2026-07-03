use neoui::*;

fn main() {
    let mut ui = ui("Test", 1000, 700);

    let mut p: f32 = 0.0;
    let mut scroll_y = 18;

    while ui.window.open() {
        ui.frame(|ui| {
            let bounds = ui.current_frame().bounds;
            ui.scroll_view(bounds, &mut scroll_y, |ui| {
                ui.text("test", style());
                ui.text("test", style());
                ui.text("test", style());
                ui.text("test", style());
                ui.text("test", style());
                ui.text("test", style());
                ui.text("test", style());
            });

            if ui.window.pressed(Key::Escape) {
                ui.window.close();
            }
        });

        scroll_y = (p * 10.0).round() as usize;
        p = (p + 0.01) % std::f32::consts::TAU;
    }
}
