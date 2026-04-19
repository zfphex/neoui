use neoui::*;
use window::*;

fn main() {
    unsafe {
        CTX.font = Some(fontdue::Font::from_bytes(FONT, fontdue::FontSettings::default()).unwrap());
        CTX.window = Some(create_window("Title", 0, 0, 800, 600, WindowStyle::DEFAULT));
    }

    loop {
        if exit() {
            break;
        }

        if button("example label", bg(black())) {
            println!("Clicked")
        }

        let ctx = unsafe { &mut *(&raw mut CTX) };
        let window = ctx.window.as_mut().unwrap();

        window.buffer.fill(black());

        draw_cmd();

        window.draw();
        window.vsync();
    }
}
