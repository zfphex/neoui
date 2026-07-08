use neoui::*;
fn main() {
    let mut ui = ui("Test", 1000, 700);
    while ui.window.open() {
        ui.frame(|ui| {
            ui.begin_layout(Flow::Down, None);
            let _s = ui.text(
                "A line of text\nAnother line of text.",
                style()
                    .fill_width()
                    .fill_height()
                    .radius(100)
                    .align(Alignment::Center)
                    .bg(rgb(34, 46, 155))
                    .fg(rgb(224, 203, 13))
                    .border(red())
                    .border_side(border::TOP | border::BOTTOM)
                    .font_size(32),
            );
            ui.end_layout();

            if ui.window.pressed(Key::Escape) {
                ui.window.close();
            }
        });
    }
}
