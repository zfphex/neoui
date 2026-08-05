use neoui::*;

fn main() {
    let mut ui = ui("Alpha", 900, 640);
    ui.default_font_size = 15;

    while ui.window.open() {
        ui.frame(|ui| {
            if ui.window.pressed(Key::Escape) {
                ui.window.close();
            }

            let (w, h) = ui.window.size();
            let (w, h) = (w as i32, h as i32);

            ui.rect(style().x(0).y(0).width(w).height(h).bg(rgb(24, 24, 30)));
            ui.rect(style().x(w / 2).y(0).width(w - w / 2).height(h).bg(rgb(228, 228, 235)));

            let steps = 8;
            let sw = (w - 80) / steps;
            for i in 0..steps {
                let a = (32 + i * 32).min(255) as u8;
                ui.rect(
                    style()
                        .x(40 + i * sw)
                        .y(48)
                        .width(sw - 8)
                        .height(120)
                        .radius(10)
                        .bg(rgba(60, 150, 255, a)),
                );
            }

            let cx = w / 2;
            let size = 200;
            let a = 130;
            ui.rect(overlay(cx - 150, 240, size, rgba(255, 60, 60, a)));
            ui.rect(overlay(cx - 40, 240, size, rgba(60, 220, 90, a)));
            ui.rect(overlay(cx - 95, 330, size, rgba(70, 120, 255, a)));

            ui.rect(
                style()
                    .x(40)
                    .y(560)
                    .width(w - 80)
                    .height(56)
                    .radius(12)
                    .bg(rgb(255, 190, 40)),
            );
            ui.text("opaque white", style().x(70).y(576).fg(white()).font_size(22));
            ui.text(
                "40% white",
                style().x(w / 2 + 30).y(576).fg(with_alpha(white(), 102)).font_size(22),
            );
        });
    }
}

fn overlay(x: i32, y: i32, size: i32, color: u32) -> Style {
    style()
        .x(x)
        .y(y)
        .width(size)
        .height(size)
        .radius(size as usize / 2)
        .bg(color)
}
