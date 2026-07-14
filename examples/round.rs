use neoui::*;

fn main() {
    let mut ui = ui("Test", 1000, 700);

    while ui.window.open() {
        ui.frame(|ui| {
            if ui.window.pressed(Key::Escape) {
                ui.window.close();
            }

            ui.rect(
                style()
                    .width(200)
                    .height(200)
                    .radius(20)
                    .border(white())
                    .border_thickness(1),
            );
        });
    }
}
