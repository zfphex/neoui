use neoui::*;

const BODY: &str = "Text runs with a fixed width breaks at that width, otherwise text won't break";
const HARD: &str = "A newline in the string still breaks the line.\nThis line started after one.";
const LONG: &str = "Supercalifragilisticexpialidocious breaks mid-word when no space fits.";

fn main() {
    let mut ui = ui("Wrap", 900, 700);
    ui.default_font_size = 15;

    let bg = rgb(18, 18, 18);
    let panel = rgb(28, 28, 28);
    let dim = rgb(150, 150, 150);
    let accent = rgb(0, 122, 204);

    let mut width = 320i32;

    while ui.window.open() {
        ui.frame(|ui| {
            if ui.window.pressed(Key::Escape) {
                ui.window.close();
            }
            if ui.window.is_down(Key::ArrowLeft) {
                width -= 4;
            }
            if ui.window.is_down(Key::ArrowRight) {
                width += 4;
            }
            width = width.clamp(60, 800);

            ui.flow_down(flow().fillw().fillh().bg(bg).pad(20).gap(20), |ui| {
                ui.text("Left and right arrows resize the column", text().fg(dim).font_size(13));

                ui.flow_down(flow().width(width + 24).bg(panel).pad(12).gap(8), |ui| {
                    ui.text(BODY, text().w(width).wrap(true).content_top_left());
                    ui.text(HARD, text().w(width).wrap(true).fg(dim).content_top_left());
                    ui.text(LONG, text().w(width).wrap(true).fg(accent).content_top_left());
                });

                ui.flow_down(flow().bg(panel).pad(12).gap(8), |ui| {
                    ui.text("No width, so this run is as wide as its text.", text().fg(dim));
                });

                ui.flow_right(flow().fillw().gap(12), |ui| {
                    for (label, style) in [
                        ("left", text().content_top_left()),
                        ("center", text().content_top_center()),
                        ("right", text().content_top_right()),
                    ] {
                        ui.flow_down(flow().w(0.33).bg(panel).pad(12).gap(6), |ui| {
                            ui.text(label, text().fg(dim).font_size(13));
                            ui.text(BODY, style.fillw().wrap(true).font_size(13));
                        });
                    }
                });
            });
        });
    }
}
