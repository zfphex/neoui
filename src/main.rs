use neoui::*;
use window::*;

pub const FONT: &[u8] = include_bytes!("../fonts/Aptos.ttf");

fn main() {
    let mut window = create_window("Title", 0, 0, 800, 600, WindowStyle::DEFAULT);
    let font = fontdue::Font::from_bytes(FONT, fontdue::FontSettings::default()).unwrap();

    loop {
        if let Some(event) = window.event() {
            match event {
                Event::Quit | Event::Input(Key::Escape, _) => break,
                _ => {}
            }
        }

        draw_text(
            "This is some text",
            &font,
            0,
            0,
            32,
            1.0,
            window.width(),
            &mut window.buffer,
            white(),
            false,
        );

        draw_window(&mut window, black());
    }
}
