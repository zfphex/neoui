use neoui::*;

fn main() {
    let mut ui = ui("Center", 900, 640);
    ui.default_font_size = 15;

    let mut fitted = Rect::default();

    while ui.window.open() {
        ui.frame(|ui| {
            if ui.window.pressed(Key::Escape) {
                ui.window.close();
            }

            // Horizontal parent + stated height - vertical centering.
            // Vertical child + stated width     - horizontal centering.
            ui.flow_right(flow().fillw().height(0.5).bg(rgb(30, 30, 30)).children_center(), |ui| {
                ui.flow_down(flow().fillw().height(64).children_center(), |ui| {
                    ui.flow_down(flow().width(160).height(64).bg(red()).pad(8), |ui| {
                        ui.text("known size", text());
                    });
                });
            });

            //Auto sized but one frame behind (typical approach).
            let (w, h) = (ui.window.size().0 as i32, ui.window.size().1 as i32);
            let state = ui.place_down(
                flow()
                    .x((w - fitted.width) / 2)
                    .y(h / 2 + (h / 4 - fitted.height / 2))
                    .bg(blue())
                    .pad(12)
                    .gap(6),
                |ui| {
                    ui.text("auto sized", text());
                    ui.text("centered on both axes", text());
                },
            );
            fitted = state.bounds;
        });
    }
}
