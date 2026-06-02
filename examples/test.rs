use neoui::*;
fn main() {
    let mut ctx = ctx("Test", 1000, 700);
    loop {
        if ctx.exit() {
            break;
        }
        ctx.begin_frame(black());

        ctx.begin_layout(Flow::Down, None);
        ctx.button("A", style());
        ctx.button("B", style());
        ctx.end_layout();

        ctx.begin_layout(Flow::Right, None);
        ctx.button("Left", style());
        ctx.button("Right", style());
        ctx.button("Left", style());
        ctx.button("Right", style());
        ctx.button("Left", style());
        ctx.button("Right", style());
        ctx.end_layout();

        ctx.begin_layout(Flow::Down, None);
        ctx.button("A", style());
        ctx.button("B", style());
        ctx.end_layout();

        ctx.draw_frame();
    }
}
