use neoui::*;

fn main() {
    let mut ui = ui("Font", 1000, 700);

    loop {
        if ui.exit() {
            break;
        }

        ui.begin_frame(black());

        //TODO: This should be aligned on the horizon line, currently it's not?
        ui.text("Example", style().bg(gray()));

        ui.draw_frame();
    }
}
