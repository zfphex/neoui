use neoui::*;

fn main() {
    let mut ui = ui("Font", 1000, 700);

    loop {
        if ui.exit() {
            break;
        }

        ui.start_frame(black());

        let (top_nav, rem) = ui.split_v(30);
        let (sidebar, tracks) = rem.split_h(260);

        // ui.flow_right_styled(style().bounds(top_nav).bg(gray()), |ui| {
        //     let s = style().gap(12);
        //     ui.text("text", s);
        //     ui.text("text", s);
        //     ui.text("text", s);
        //     ui.text("text", s);
        //     ui.text("text", s);
        //     ui.text("text", s);
        //     ui.text("text", s);
        //     ui.text("text", s);
        //     ui.text("text", s);
        // });

        // ui.flow_down_styled(style().bounds(sidebar).bg(rgb(30, 40, 80)), |_| {});

        // ui.flow_down_styled(style().bounds(tracks).bg(rgb(90, 40, 50)), |_| {});

        let sides = border::LEFT | border::RIGHT | border::TOP;

        ui.paint_rect(
            rem,
            style().bg(blue()).radius(40).border_color(red()).border_sides(sides),
        );
        // ui.paint_rect(
        //     top_nav,
        //     style().bg(green()).border_thickness(1).border_sides(border::ALL),
        // );

        ui.draw_frame();
    }
}
