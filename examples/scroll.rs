use neoui::*;

fn main() {
    let mut ui = ui("Test", 1000, 700);

    let scroll_y = 18;
    loop {
        if ui.exit() {
            break;
        }

        ui.start_frame(black());

        let bounds = ui.layout_stack.last().unwrap().bounds;
        ui.begin_scroll_view(bounds, scroll_y);
        ui.text("test", style());
        ui.text("test", style());
        ui.text("test", style());
        ui.text("test", style());
        ui.text("test", style());
        ui.text("test", style());
        ui.text("test", style());

        ui.end_scroll_view();

        ui.draw_frame();
    }
}
