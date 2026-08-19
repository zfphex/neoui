use neoui::*;

fn main() {
    let (wide_pixels, ww, wh) = decode(&std::fs::read("target/img/photo.jpg").unwrap()).unwrap();
    let wide = Image::new(ww, wh, &wide_pixels);

    let (tall_pixels, tw, th) = decode(&std::fs::read("target/img/gradient.png").unwrap()).unwrap();
    let tall = Image::new(tw, th, &tall_pixels);

    let (logo_pixels, lw, lh) = decode(&std::fs::read("target/img/rings.png").unwrap()).unwrap();
    let logo = Image::new(lw, lh, &logo_pixels);

    let radii = [0, 12, 32, 75];
    let opacities = [48u8, 112, 176, 255];
    let sizes = [48, 96, 150, 220];

    let mut app = ui("Image", 800, 800);

    while app.window.open() {
        app.frame(|ui| {
            if ui.window.pressed(Key::Escape) {
                ui.window.close();
            }

            let cell = image().width(150).height(150).gap(22);
            let caption = text().fg(gray()).font_size(14).width(150).gap(22);
            let heading = text().font_size(17).padt(16).padb(8);

            ui.flow_down(flow().pad(20), |ui| {
                ui.text("radius", heading);
                ui.flow_right(flow().height(150), |ui| {
                    for radius in radii {
                        ui.image(wide, cell.radius(radius));
                    }
                });
                ui.flow_right(flow().height(20), |ui| {
                    for radius in radii {
                        ui.text(radius.to_string(), caption);
                    }
                });

                ui.text("opacity over background", heading);
                ui.flow_right(flow().height(150), |ui| {
                    for opacity in opacities {
                        ui.image(tall, cell.radius(8).opacity(opacity).bg(rgb(190, 80, 30)));
                    }
                });
                ui.flow_right(flow().height(20), |ui| {
                    for opacity in opacities {
                        ui.text(opacity.to_string(), caption);
                    }
                });

                ui.text("scaling, alpha", heading);
                ui.flow_right(flow().height(220), |ui| {
                    for size in sizes {
                        ui.image(logo, image().width(size).height(size).gap(22).bg(rgb(40, 70, 110)));
                    }
                });
                ui.flow_right(flow().height(20), |ui| {
                    for size in sizes {
                        ui.text(format!("{size}px"), caption);
                    }
                });
            });
        });
    }
}
