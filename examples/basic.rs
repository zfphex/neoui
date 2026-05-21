use neoui::*;

fn main() {
    unsafe {
        CTX.font = Some(fontdue::Font::from_bytes(FONT, fontdue::FontSettings::default()).unwrap());
        CTX.window = Some(create_window(
            "Basic",
            0,
            0,
            1000,
            750,
            WindowStyle::DEFAULT,
        ));
    }

    let ctx = unsafe { &mut *(&raw mut CTX) };

    loop {
        if exit() {
            break;
        }

        let width = ctx.width();
        let _height = ctx.height();

        begin_ui(black());

        {
            begin_layout_with_bounds(Flow::Right, Rect::new(0, 0, width, 30));

            let menu_style = style()
                .font_size(13)
                .height(30)
                .pad(13)
                .bg(rgb(25, 25, 25))
                .hover(rgb(45, 45, 45));

            ctx.rect(Rect::new(0, 0, width, 30), rgb(25, 25, 25));
            ctx.button("File", menu_style);
            ctx.button("Edit", menu_style);
            ctx.button("View", menu_style);
            ctx.button("Playback", menu_style);
            ctx.button("Library", menu_style);
            ctx.button("Help", menu_style);

            end_layout();
        }

        draw_cmd();
        ctx.draw();
    }
}
