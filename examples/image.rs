use neoui::*;

fn main() {
    let wide = Image::open("target/img/wide.jpg").unwrap();
    let tall = Image::open("target/img/tall.jpg").unwrap();
    let small = Image::open("target/img/small.jpg").unwrap();
    let logo = Image::open("target/img/logo.png").unwrap();
    let thumb = wide.thumbnail(96);

    let fits = [
        ("Stretch", ImageFit::Stretch),
        ("Contain", ImageFit::Contain),
        ("Cover", ImageFit::Cover),
        ("Fixed", ImageFit::Fixed),
    ];
    let radii = [0, 12, 32, 75];
    let opacities = [48u8, 112, 176, 255];

    let mut app = ui("Image", 714, 940);

    while app.window.open() {
        app.frame(|ui| {
            if ui.window.pressed(Key::Escape) {
                ui.window.close();
            }

            let cell = style().width(150).height(150).gap(22);
            let caption = style().fg(gray()).font_size(14).width(150).gap(22);
            let heading = style().font_size(17).padt(16).padb(8);

            ui.flow_down(style().pad(20), |ui| {
                ui.text("ImageFit", heading);
                ui.flow_right(style().height(150), |ui| {
                    for (_, fit) in fits {
                        ui.image(&wide, cell.image_fit(fit).bg(rgb(26, 26, 30)));
                    }
                });
                ui.flow_right(style().height(20), |ui| {
                    for (name, _) in fits {
                        ui.text(name, caption);
                    }
                });

                ui.text("radius", heading);
                ui.flow_right(style().height(150), |ui| {
                    for radius in radii {
                        ui.image(&wide, cell.image_fit(ImageFit::Cover).radius(radius));
                    }
                });
                ui.flow_right(style().height(20), |ui| {
                    for radius in radii {
                        ui.text(radius.to_string(), caption);
                    }
                });

                ui.text("opacity over background", heading);
                ui.flow_right(style().height(150), |ui| {
                    for opacity in opacities {
                        ui.image(
                            &tall,
                            cell.image_fit(ImageFit::Cover)
                                .radius(8)
                                .opacity(opacity)
                                .bg(rgb(190, 80, 30)),
                        );
                    }
                });
                ui.flow_right(style().height(20), |ui| {
                    for opacity in opacities {
                        ui.text(opacity.to_string(), caption);
                    }
                });

                ui.text("alpha, thumbnail, upscale", heading);
                ui.flow_right(style().height(150), |ui| {
                    let panel = cell.bg(rgb(40, 70, 110));
                    ui.image(&logo, panel.image_fit(ImageFit::Contain));
                    ui.image(&thumb, panel.image_fit(ImageFit::Fixed));
                    ui.image(&small, panel.image_fit(ImageFit::Cover));
                    ui.image(&small, panel.image_fit(ImageFit::Fixed));
                });
                ui.flow_right(style().height(20), |ui| {
                    for name in ["logo.png alpha", "thumbnail(96)", "48px upscaled", "48px at 1:1"] {
                        ui.text(name, caption);
                    }
                });
            });
        });
    }
}
