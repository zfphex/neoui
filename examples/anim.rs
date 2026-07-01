use neoui::*;

fn main() {
    let mut ui = ui("Test", 1000, 700);

    let mut sidebar_open = false;
    let mut button_hovered = false;

    loop {
        ui.start_frame(black());

        if let Some(event) = ui.poll_event() {
            match event {
                Event::Quit | Event::Input(Key::Escape, _) => break,
                _ => {}
            }
        }

        let target_width = if sidebar_open { 300.0 } else { 60.0 };
        let current_width = ui.animate_f32(target_width, 0.2, Ease::OutCubic) as usize;

        let target_btn_color = if button_hovered{ hex("#3b82f6") } else { hex("#3f3f46") };
        let current_btn_color = ui.animate_color(target_btn_color, 10.0);

        ui.flow_right(style(), |ui| {
            // Sidebar
            ui.flow_down(style().width(current_width).bg(hex("#27272a")), |ui| {
                let btn_text = if current_width > 180 && sidebar_open {
                    "Collapse"
                } else {
                    ">>"
                };

                let toggle_btn = ui.text(
                    btn_text,
                    style().width(Size::Fill).height(50).bg(current_btn_color).fg(white()),
                );

                button_hovered = toggle_btn.hovered;

                if toggle_btn.clicked {
                    sidebar_open = !sidebar_open;
                }
            });

            // Body
            // TODO: Pad does not work here.
            ui.flow_down(style().pad(80), |ui| {
                ui.text("Main Content Area", style().fg(white()));
            });
        });

        ui.draw_frame();
    }
}
