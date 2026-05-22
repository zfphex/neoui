use neoui::*;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Menu {
    File,
    Edit,
    View,
    Playback,
    Library,
    Help,
}

fn volume_slider(ctx: &mut Context, volume: &mut f32, width: usize, height: usize) {
    let rect = ctx.walk_layout(width, height);
    ctx.rect(rect, rgb(25, 25, 25));
    let window = ctx.window.as_ref().unwrap();
    if window.mouse_position.intersects(rect) && window.left_mouse.pressed {
        let click_x = window.mouse_position.x.saturating_sub(rect.x);
        *volume = (click_x as f32 / width as f32).clamp(0.0, 1.0);
    }

    let max_track_h = 6;
    let cy = rect.y + height / 2;

    ctx.triangle(
        (rect.x, cy + max_track_h),
        (rect.x + width, cy + max_track_h),
        (rect.x + width, cy - max_track_h),
        hex("#000000"),
    );

    let thumb_w = 12;
    let thumb_h = 18;

    let available_width = width.saturating_sub(thumb_w);
    let thumb_x = rect.x + (*volume * available_width as f32).round() as usize;
    let thumb_y = rect.y + (height.saturating_sub(thumb_h)) / 2;

    let thumb_color = hex("#0078D7");
    ctx.rect(Rect::new(thumb_x, thumb_y, thumb_w, thumb_h), thumb_color);
}

fn main() {
    let ctx = create_ctx("Basic", 1000, 700);
    let mut current_menu: Option<(Menu, Rect)> = None;
    let mut volume = 0.5;

    loop {
        if exit() {
            break;
        }

        let ctx_width = ctx.width();
        let ctx_height = ctx.height();

        begin_ui(black());

        {
            let height = 30;
            let menu_style = style()
                .font_size(13)
                .height(height)
                .padl(14)
                .padr(14)
                .bg(rgb(25, 25, 25))
                .hover(rgb(45, 45, 45));

            begin_layout_with_bounds(Flow::Right, Rect::new(0, 0, ctx_width, height));

            let items = [
                ("File", Menu::File),
                ("Edit", Menu::Edit),
                ("View", Menu::View),
                ("Playback", Menu::Playback),
                ("Library", Menu::Library),
                ("Help", Menu::Help),
            ];

            for (label, menu) in items {
                let state = ctx.button(label, menu_style);
                if state.clicked {
                    current_menu = Some((menu, state.rect));
                }
            }

            let bar_style = style().width(1).height(height).bg(hex("#424242"));
            let gap = bar_style.bg(rgb(25, 25, 25)).width(120);
            ctx.spacer(bar_style);
            ctx.spacer(gap);
            ctx.spacer(bar_style);
            ctx.spacer(gap);
            ctx.spacer(bar_style);
            let frame = ctx.layout_stack.last().unwrap();
            ctx.spacer(gap.width(ctx_width - frame.cursor_x - 200 - 14));
            volume_slider(ctx, &mut volume, 200, height);

            ctx.spacer(menu_style);

            end_layout();
        }

        if let Some((menu, rect)) = current_menu {
            let dropdown_width = 180;
            let item_style = style()
                .font_size(13)
                .padlr(12)
                .padtb(8)
                .bg(rgb(35, 35, 35))
                .hover(rgb(60, 60, 60));

            begin_layout_with_bounds(Flow::Down, Rect::new(rect.x, 30, dropdown_width, 400));

            let drop_down: &[&str] = match menu {
                Menu::File => &["New Project", "Open File...", "Save"],
                Menu::Edit => &["Undo", "Redo", "Cut"],
                Menu::View => &["Toggle Sidebar", "Zoom In"],
                Menu::Playback => &["Play / Pause", "Stop"],
                Menu::Library => &["Scan Folders..."],
                Menu::Help => &["Documentation", "About"],
            };

            for &item in drop_down {
                if ctx.list_item(item, false, dropdown_width, item_style).clicked {
                    println!("{}", item);
                    current_menu = None;
                }
            }

            end_layout();

            if ctx.clicked(Rect::new(0, 0, ctx_width, ctx_height)) {
                current_menu = None;
            }
        }

        draw_cmd();
        ctx.draw();
    }
}
