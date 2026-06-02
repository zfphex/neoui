use neoui::*;
fn main() {
    let mut ctx = ctx("Test", 1000, 700);
    loop {
        if ctx.exit() {
            break;
        }
        ctx.begin_frame(black());

        ctx.begin_layout(Flow::Down, None);
        ctx.text("A", style());
        ctx.text("B", style());
        ctx.end_layout();

        ctx.begin_layout(Flow::Right, None);
        ctx.text("Left", style());
        ctx.text("Right", style());
        ctx.text("Left", style());
        ctx.text("Right", style());
        ctx.text("Left", style());
        ctx.text("Right", style());
        ctx.end_layout();

        ctx.begin_layout(Flow::Down, None);
        ctx.text("A", style());
        ctx.text("B", style());
        ctx.end_layout();

        ctx.draw_frame();
    }
}
