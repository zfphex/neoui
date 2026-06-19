use neoui::*;

fn main() {
    let icon_font = fontdue::Font::from_bytes(
        include_bytes!("../fonts/MaterialIcons-Regular.ttf") as &[u8],
        fontdue::FontSettings::default(),
    )
    .unwrap();

    let mut ui = ui("Icons", 1000, 700);
    ui.default_font_size = 16;

    let icons = [
        ('\u{e88a}', "home"),
        ('\u{e8b6}', "search"),
        ('\u{e87d}', "favorite"),
        ('\u{e8b8}', "settings"),
        ('\u{e037}', "play"),
        ('\u{e034}', "pause"),
        ('\u{e047}', "stop"),
    ];

    loop {
        if let Some(event) = ui.poll_event() {
            match event {
                Event::Quit | Event::Input(Key::Escape, _) => break,
                _ => {}
            }
        }

        ui.start_frame(black());

        for (i, (icon, label)) in icons.iter().enumerate() {
            let x = 20 + i * 140;
            let icon_rect = Rect::new(x, 80, 96, 96);
            let label_rect = Rect::new(x, 190, 96, 28);

            ui.paint_rect(
                icon_rect,
                style().bg(rgb(24, 24, 24)).border(rgb(55, 55, 55)).radius(12),
            );
            ui.paint_icon(*icon, icon_rect, &icon_font, style().fg(white()).font_size(64));
            ui.flow_once(bounds(label_rect), Flow::Down, |ui| {
                ui.text(*label, style().fg(gray()).font_size(16));
            });
        }

        ui.draw_frame();
    }
}
