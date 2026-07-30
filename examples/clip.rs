use neoui::*;

fn main() {
    let mut ui = ui("clip and cross_align", 900, 600);
    ui.default_font_size = 14;

    while ui.window.open() {
        if ui.window.pressed(Key::Escape) {
            ui.window.close();
        }

        ui.frame(|ui| {
            let panel = style()
                .width(260)
                .height(130)
                .bg(rgb(24, 24, 26))
                .border(rgb(80, 80, 86))
                .radius(6);
            let label = style().fg(rgb(150, 150, 155)).font_size(12).padb(6);
            let big = style().fg(rgb(120, 190, 255)).font_size(26).padlr(10);
            let row = style().fg(white()).padlr(10).padtb(2);

            ui.flow_right(style().pad(28).gap(60), |ui| {
                ui.flow_down(style().gap(28), |ui| {
                    ui.text("clip(true)", label);
                    ui.flow_down(panel.clip(true), |ui| {
                        for _ in 0..5 {
                            ui.text("overflowing content", big);
                        }
                    });

                    ui.text("clip(false) — default", label);
                    ui.flow_down(panel, |ui| {
                        for _ in 0..5 {
                            ui.text("overflowing content", big);
                        }
                    });
                });

                ui.flow_down(style().gap(20), |ui| {
                    for (name, align) in [
                        ("CrossAlign::Start", CrossAlign::Start),
                        ("CrossAlign::Center", CrossAlign::Center),
                        ("CrossAlign::End", CrossAlign::End),
                    ] {
                        ui.text(name, label);
                        ui.flow_down(panel.cross_align(align).gap(4).padtb(10), |ui| {
                            ui.text("short", row);
                            ui.text("a little longer", row);
                            ui.text("the longest row of the three", row);
                        });
                    }
                });
            });
        });
    }
}
