use neoui::*;

fn main() {
    let mut ui = ui_hidden(600, 400);
    ui.default_font_size = 24;

    //Render 3 frames, only write out the last one.
    for frame in 0..3 {
        let render = |ui: &mut FrameContext| {
            ui.flow_down(flow().fillw().fillh().bg(rgb(24, 24, 30)).pad(40).gap(12), |ui| {
                ui.text("headless render", text().fg(rgb(240, 240, 240)));
                ui.rect(rect().width(220).height(80).radius(12).bg(rgb(0, 122, 204)));
            });
        };

        if frame == 2 {
            ui.frame_hidden("target/headless.png", render).unwrap();
        } else {
            ui.frame(render);
        }
    }
}
