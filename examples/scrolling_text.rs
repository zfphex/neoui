use neoui::*;

fn main() {
    let mut ui = ui("Test", 1000, 700);

    let mut p: f32 = 0.0;
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
            ui.text("test", style());
            ui.text("test", style());
            ui.text("test", style());
            ui.text("test", style());
            ui.text("test", style());
            ui.text("test", style());
            ui.text("test", style());
        });

        scroll_y = (p * 10.0).round() as usize;
        p = (p + 0.01) % std::f32::consts::TAU;

        ui.draw_frame();
    }
}
