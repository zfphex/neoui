use neoui::*;

fn main() {
    let mut ui = ui("Icons", 1000, 700);

    loop {
        if let Some(event) = ui.poll_event() {
            match event {
                Event::Quit | Event::Input(Key::Escape, _) => break,
                _ => {}
            }
        }

        ui.start_frame(black());

        let clip = ui.layout_stack.last().expect("No active frame").clip;
        // draw_text("", font, x, y, font_size, display_scale, window_width, buffer, color, cache, clip)
        // ui.paint_icon();

        ui.draw_frame();
    }
}
