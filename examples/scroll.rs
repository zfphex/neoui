use neoui::*;

fn main() {
    let mut ui = ui("Test", 1000, 700);

    let mut scroll_y = 18;

    loop {
        ui.start_frame(black());

        if let Some(event) = ui.poll_event() {
            match event {
                Event::Quit | Event::Input(Key::Escape, _) => break,
                _ => {}
            }
        }

        let bounds = ui.layout_stack.last().unwrap().bounds;
        ui.scroll_view(bounds, &mut scroll_y, |ui| {
            for _ in 0..100 {
                ui.text("test", style());
            }
        });

        ui.draw_frame();
    }
}
