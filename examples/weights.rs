use neoui::*;

const FACES: [(Weight, bool, &[u8]); 11] = [
    (Weight::Light, false, include_bytes!("../fonts/Aptos-Light.ttf")),
    (Weight::SemiBold, false, include_bytes!("../fonts/Aptos-SemiBold.ttf")),
    (Weight::Bold, false, include_bytes!("../fonts/Aptos-Bold.ttf")),
    (Weight::ExtraBold, false, include_bytes!("../fonts/Aptos-ExtraBold.ttf")),
    (Weight::Black, false, include_bytes!("../fonts/Aptos-Black.ttf")),
    (Weight::Light, true, include_bytes!("../fonts/Aptos-Light-Italic.ttf")),
    (Weight::Regular, true, include_bytes!("../fonts/Aptos-Italic.ttf")),
    (Weight::SemiBold, true, include_bytes!("../fonts/Aptos-SemiBold-Italic.ttf")),
    (Weight::Bold, true, include_bytes!("../fonts/Aptos-Bold-Italic.ttf")),
    (Weight::ExtraBold, true, include_bytes!("../fonts/Aptos-ExtraBold-Italic.ttf")),
    (Weight::Black, true, include_bytes!("../fonts/Aptos-Black-Italic.ttf")),
];

const WEIGHTS: [Weight; 6] = [
    Weight::Light,
    Weight::Regular,
    Weight::SemiBold,
    Weight::Bold,
    Weight::ExtraBold,
    Weight::Black,
];

fn main() {
    let mut ui = ui("Weights", 1100, 620);
    ui.clear_color = white();

    for (weight, italic, bytes) in FACES {
        let face = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).unwrap();
        ui.add_face(Font::default().id, weight, italic, face);
    }

    while ui.window.open() {
        ui.frame(|ui| {
            ui.flow_right(style().fill().pad(16), |ui| {
                for italic in [false, true] {
                    ui.flow_down(style().w(550).fill_height(), |ui| {
                        for weight in WEIGHTS {
                            let mut s = style().weight(weight).font_size(28).fg(black()).align_left().w(550).h(96);
                            if italic {
                                s = s.italic();
                            }
                            ui.text(format!("{weight:?} Hamburgefonstiv 123"), s);
                        }
                    });
                }
            });
        });

        if ui.window.pressed(Key::Escape) {
            ui.window.close();
        }
    }
}
