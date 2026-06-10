use neoui::*;

fn main() {
    let mut ui = ui("Size", 1000, 700);
    let mut p: f32 = 0.0;

    loop {
        if ui.exit() {
            break;
        }

        ui.start_frame(black());

        // Creates two relatively sized rectangles.
        let (t, b) = ui.split_v(p.sin().abs());

        ui.paint_rect(t, bg(red()));
        ui.paint_rect(b, bg(green()));

        ui.flow_down(t, |ui| {
            ui.rect(style().width(100).height(100).bg(rgb(202, 202, 202)));
            ui.rect(style().width(100).height(100).bg(rgb(119, 119, 119)));
            ui.rect(style().width(100).height(100).bg(rgb(88, 88, 88)));
        });

        p = (p + 0.001) % std::f32::consts::TAU;

        ui.draw_frame();
    }
}
