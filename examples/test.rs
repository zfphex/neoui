use neoui::*;
fn main() {
    let mut ui = ui("Test", 1000, 700);
    loop {
        if ui.exit() {
            break;
        }
        ui.begin_frame(black());

        ui.begin_layout(Flow::Down, None);
        let s = ui.text(
            "A line of text\nAnother line of text.",
            style()
                // .bg(red())
                .bg(gray())
                //TODO: Borders 
                // .border(red())
                .font_size(32),
        );
        ui.end_layout();

        ui.draw_frame();
    }
}
