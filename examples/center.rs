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
            ui.flow_right(
                flow().fillw().height(1.0 / 3.0).bg(rgb(30, 30, 30)).children_center(),
                |ui| {
                    ui.flow_down(flow().fillw().height(64).children_center(), |ui| {
                        ui.flow_down(flow().width(160).height(64).bg(red()).pad(8), |ui| {
                            ui.text("known size", text());
                        });
                    });
                },
            );

            // Auto sized but one frame behind.
            let (w, h) = (ui.window.size().0 as i32, ui.window.size().1 as i32);
            let state = ui.place_down(
                flow()
                    .x((w - fitted.width) / 2)
                    .y(h / 3 + (h / 6 - fitted.height / 2))
                    .bg(blue())
                    .pad(12)
                    .gap(6),
                |ui| {
                    ui.text("auto sized (1 frame behind)", text());
                    ui.text("centered on both axes", text());
                },
            );
            fitted = state.bounds;

            // Auto sized in the same frame using hide + ui.center().
            ui.place_down(flow().y(h * 2 / 3).fillw().height(h / 3), |ui| {
                let element = |ui: &mut FrameContext<'_, '_>, hide: bool, c: Rect| {
                    let style = flow().x(c.x).y(c.y).bg(hex("#238636")).pad(12).gap(6);
                    ui.place_down(if hide { style.hide() } else { style }, |ui| {
                        ui.text("auto sized (same frame)", text());
                        ui.text("measured hidden, then centered", text());
                    })
                };
                let measured = element(ui, true, Rect::new(0, 0, 0, 0));
                let c = ui.center(measured.bounds);
                element(ui, false, c);
            });
        });
    }
}
