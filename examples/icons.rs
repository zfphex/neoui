use neoui::*;

fn main() {
    let icon_font = fontdue::Font::from_bytes(
        include_bytes!("../fonts/MaterialIcons-Regular.ttf") as &[u8],
        fontdue::FontSettings::default(),
    )
    .unwrap();

    let mut ui = ui("Icons", 1000, 700);
    ui.default_font_size = 16;
    let icon_id = ui.add_font(icon_font);

    let icons = [
        ("\u{e88a}", "home"),
        ("\u{e8b6}", "search"),
        ("\u{e87d}", "favorite"),
        ("\u{e8b8}", "settings"),
        ("\u{e037}", "play"),
        ("\u{e034}", "pause"),
        ("\u{e047}", "stop"),
    ];

    while ui.window.open() {
        ui.frame(|ui| {
            ui.flow_down(style(), |ui| {
                for (icon, label) in icons.iter() {
                    ui.flow_right(style(), |ui| {
                        ui.text(*icon, style().font(icon_id).font_size(32));
                        ui.text(*label, style().font_size(32));
                    });
                }
            });

            ui.gap(270);

            ui.flow_right(style(), |ui| {
                ui.gap(20);

                for (icon, label) in icons {
                    ui.flow_down(style().width(96), |ui| {
                        ui.text(
                            icon,
                            style()
                                .width(96)
                                .height(96)
                                .bg(rgb(24, 24, 24))
                                .border(rgb(55, 55, 55))
                                .font(icon_id)
                                .font_size(64),
                        );

                        ui.gap(8);

                        ui.text(
                            label,
                            style()
                                .fg(gray())
                                .font_size(16)
                                .fill_width()
                                .padl(2)
                                .align(Alignment::Left),
                        );
                    });

                    ui.gap(45);
                }
            });

            for (i, (icon, label)) in icons.iter().enumerate() {
                let x = 20 + i * 140;
                let y = 260;
                let icon_rect = Rect::new(x, y + 80, 96, 96);
                let label_rect = Rect::new(x, y + 190, 96, 28);

                ui.paint_rect(icon_rect, style().bg(rgb(24, 24, 24)).border(rgb(55, 55, 55)));
                ui.paint_text(
                    *icon,
                    icon_rect.x as i32,
                    icon_rect.y as i32,
                    icon_rect.width,
                    icon_rect.height,
                    white(),
                    icon_id,
                    64,
                    Alignment::Center,
                    Padding::default(),
                    0,
                );
                ui.flow_skip(bounds(label_rect), Flow::Down, |ui| {
                    ui.text(*label, style().fg(gray()).font_size(16));
                });
            }

            if ui.window.pressed(Key::Escape) {
                ui.window.close();
            }
        });
    }
}
