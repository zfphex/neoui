use neoui::*;

fn main() {
    defer_results!();
    let mut ui = ui("Test", 1000, 700);

    while ui.window.open() {
        ui.frame(|ui| {
            if ui.window.pressed(Key::Escape) {
                ui.window.close();
            }

            for _ in 0..6 {
                ui.flow_right(flow(), |ui| {
                    ui.rect(rect().width(200).height(200).radius(30).bg(hex("#8aa6d4")));
                    ui.rect(rect().width(200).height(200).radius(30).bg(hex("#8aa6d4")));
                    ui.rect(rect().width(200).height(200).radius(30).bg(hex("#8aa6d4")));
                    ui.rect(rect().width(200).height(200).radius(30).bg(hex("#8aa6d4")));
                    ui.rect(rect().width(200).height(200).radius(30).bg(hex("#8aa6d4")));
                });
            }
        });
    }
}
