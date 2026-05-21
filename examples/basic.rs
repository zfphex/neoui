use neoui::*;

fn main() {
    let ctx = create_ctx("Basic", 1000, 700, WindowStyle::DEFAULT);

    loop {
        if exit() {
            break;
        }

        begin_ui(black());

        {
            begin_layout_with_bounds(Flow::Right, Rect::new(0, 0, ctx.width(), 30));

            let menu_style = style()
                .font_size(13)
                .height(30)
                .pad(13)
                .bg(rgb(25, 25, 25))
                .hover(rgb(45, 45, 45));

            ctx.button("File", menu_style);
            ctx.button("Edit", menu_style);
            ctx.button("View", menu_style);
            ctx.button("Playback", menu_style);
            ctx.button("Library", menu_style);
            ctx.button("Help", menu_style);
            ctx.spacer(menu_style);

            end_layout();
        }

        draw_cmd();
        ctx.draw();
    }
}
