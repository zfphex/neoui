use neoui::*;
fn main() {
    let mut ui = ui("Test", 1000, 700);
    loop {
        if ui.exit() {
            break;
        }
        ui.begin_frame(black());

        ui.begin_layout(Flow::Down, None);
        let state = ui.text("A", style().font_size(32).border(gray()).bg(rgb(80, 80, 80)));
        ui.end_layout();

        ui.draw_frame();
    }
}
