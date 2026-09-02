use neoui::*;

#[rustfmt::skip]
const LETTERS: [&str; 26] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
    "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
];

fn main() {
    let mut ui = ui("A-Z Rail", 420, 620);
    ui.default_font_size = 12;
    ui.clear_color = hex("#0b0b0c");

    let stocked = "ABCDEFGHIJKLMNOPRSTVWY";
    let mut active = "D";

    while ui.window.open() {
        ui.frame(|ui| {
            if ui.window.pressed(Key::Escape) {
                ui.window.close();
            }

            let (w, h) = ui.window.size();
            let (w, h) = (w as i32, h as i32);
            let rail = Rect::new(w - 30, 0, 30, h);
            let inside = ui.hovered(rail);
            let fade = ui.animate_f32(if inside { 1.0 } else { 0.0 }, 0.15, Ease::InOutSine);
            let my = (ui.mouse_position().y - rail.y) as f32;
            let at = |offset: f32| (my + offset) / rail.height as f32;
            let glow = |a: f32| rgba(155, 132, 217, (a * fade * 255.0) as u8);

            ui.rect(rect().x(0).y(0).width(w).height(h).bg(hex("#101011")));
            ui.rect(
                rect()
                    .x(rail.x)
                    .y(0)
                    .width(rail.width)
                    .height(h)
                    .bg(rgba(236, 233, 228, 13)),
            );

            if fade > 0.0 {
                ui.gradient(gradient().x(rail.x).y(0).width(rail.width).height(h), 180.0)
                    .stop(at(-55.0), glow(0.0))
                    .stop(at(-32.0), glow(0.11))
                    .stop(at(0.0), glow(0.30))
                    .stop(at(32.0), glow(0.11))
                    .stop(at(55.0), glow(0.0));

                ui.gradient(gradient().x(rail.x).y(0).width(1).height(h), 180.0)
                    .stop(at(-70.0), rgba(155, 132, 217, 0))
                    .stop(at(0.0), rgba(199, 183, 240, (0.75 * fade * 255.0) as u8))
                    .stop(at(70.0), rgba(155, 132, 217, 0));
            }

            let top = (h - 26 * 15) / 2;
            for (i, ch) in LETTERS.iter().enumerate() {
                let has = stocked.contains(ch);
                let is_active = *ch == active;
                let state = ui.text(
                    *ch,
                    text()
                        .x(rail.x + 6)
                        .y(top + i as i32 * 15)
                        .width(18)
                        .height(13)
                        .radius(3)
                        .content(Alignment::Center)
                        .font_size(11)
                        .fg(match (is_active, has) {
                            (true, _) => hex("#c7b7f0"),
                            (_, true) => rgba(236, 233, 228, 153),
                            _ => rgba(236, 233, 228, 56),
                        })
                        .bg(if is_active { rgba(155, 132, 217, 41) } else { rgba(0, 0, 0, 0) }),
                );

                if state.clicked && has {
                    active = ch;
                }
            }
        });
    }
}
