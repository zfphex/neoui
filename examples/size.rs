use neoui::*;

fn main() {
    let mut ctx = ctx("Size", 1000, 700);
    let mut p: f32 = 0.0;

    loop {
        if ctx.exit() {
            break;
        }

        ctx.begin_frame(black());

        // Creates two relatively sized rectangles.
        let (t, b) = ctx.split_v(p.sin().abs());

        ctx.paint_rect(t, bg(red()));
        ctx.paint_rect(b, bg(green()));

        ctx.flow_down(t, |ctx| {
            ctx.rect(style().width(100).height(100).bg(rgb(202, 202, 202)));
            ctx.rect(style().width(100).height(100).bg(rgb(119, 119, 119)));
            ctx.rect(style().width(100).height(100).bg(rgb(88, 88, 88)));
        });

        p = (p + 0.001) % std::f32::consts::TAU;

        ctx.draw_frame();
    }
}
