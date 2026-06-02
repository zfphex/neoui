use neoui::*;
fn main() {
    let mut ui = ui("Test", 1000, 700);
    loop {
        if ui.exit() {
            break;
        }
        ui.begin_frame(black());

        ui.begin_layout(Flow::Down, None);
        ui.text("A", style());
        ui.text("B", style());
        ui.end_layout();

        ui.begin_layout(Flow::Right, None);
        ui.text("Left", style());
        ui.text("Right", style());
        ui.text("Left", style());
        ui.text("Right", style());
        ui.text("Left", style());
        ui.text("Right", style());
        ui.end_layout();

        ui.begin_layout(Flow::Down, None);
        ui.text("A", style());
        ui.text("B", style());
        ui.end_layout();

        ui.draw_frame();
    }
}
