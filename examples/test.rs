use neoui::*;
fn main() {
    let mut ui = ui("Test", 1000, 700);
    while ui.window.open() {
        ui.frame(|ui| {
            ui.begin_layout(Flow::Down, None);
            let _s = ui.text(
                "A line of text\nAnother line of text.",
                style()
                    // .bg(red())
                    .bg(gray())
                    //TODO: Borders
                    // .border(red())
                    .font_size(32),
            );
            ui.end_layout();

            if ui.window.pressed(Key::Escape) {
                ui.window.close();
            }
        });
    }
}
