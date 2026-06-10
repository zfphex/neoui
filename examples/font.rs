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

        ui.flow_right_styled(style().bounds(top_nav).bg(gray()), |ui| {
            let s = style().gap(12);
            ui.text("text", s);
            ui.text("text", s);
            ui.text("text", s);
            ui.text("text", s);
            ui.text("text", s);
            ui.text("text", s);
            ui.text("text", s);
            ui.text("text", s);
            ui.text("text", s);
        });

        ui.flow_down_styled(style().bounds(sidebar).bg(rgb(30, 40, 80)), |_| {});

        ui.flow_down_styled(style().bounds(tracks).bg(rgb(90, 40, 50)), |_| {});

        // ui.flow_down_styled(style().width(0.5).bg(rgb(30, 30, 30)), |ui| {
        //     ui.text("Okay", style());
        // });

        // ui.flow_down_styled(style().width(0.5).bg(rgb(80, 90, 80)), |ui| {
        //     ui.text("Okay", style());
        // });

        // ui.flow_down_styled(style().width(0.5).bg(rgb(90, 30, 20)), |ui| {
        //     ui.text("Okay", style());
        // });

        // ui.flow_right_styled(style().height(300).bg(rgb(20, 90, 20)), |ui| {
        //     for _ in 0..10 {
        //         ui.text("Okay", style());
        //     }
        // });

        ui.draw_frame();
    }
}
