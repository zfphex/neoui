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
    ctx.paint_rect(rect, bg(rgb(25, 25, 25)));
    let window = ctx.window.as_ref().unwrap();
    if let Some(inital) = window.left_mouse.inital_position
        && window.left_mouse.pressed
    {
        if inital.intersects(rect) {
            let click_x = window.mouse_position.x.saturating_sub(rect.x);
            *volume = (click_x as f32 / width as f32).clamp(0.0, 1.0);
        }
    }

    let max_track_h = 6;
    let cy = rect.y + height / 2;

    ctx.paint_triangle(
        (rect.x, cy + max_track_h),
        (rect.x + width, cy + max_track_h),
        (rect.x + width, cy - max_track_h),
        bg(hex("#000000")),
    );

    let thumb_w = 12;
    let thumb_h = 18;

    let available_width = width.saturating_sub(thumb_w);
    let thumb_x = rect.x + (*volume * available_width as f32).round() as usize;
    let thumb_y = rect.y + (height.saturating_sub(thumb_h)) / 2;

    let thumb_color = hex("#0078D7");
    ctx.paint_rect(Rect::new(thumb_x, thumb_y, thumb_w, thumb_h), bg(thumb_color));
}

pub fn scrollbar(
    ctx: &mut Context,
    viewport: Rect,
    content_height: usize,
    scroll_y: &mut usize,
    scroll_direction: i32,
) {
    if content_height <= viewport.height {
        return;
    }

    let max_scroll = content_height.saturating_sub(viewport.height);
    let visible_ratio = viewport.height as f32 / content_height as f32;
    let min_thumb_h = 20.0;

    let thumb_h = (viewport.height as f32 * visible_ratio).max(min_thumb_h) as usize;
    let track_h = viewport.height.saturating_sub(thumb_h);

    let w = 8;
    let pad = 4;
    let x = viewport.x + viewport.width - w - pad;
    let hitbox = Rect::new(x.saturating_sub(pad), viewport.y, w + pad * 2, viewport.height);

    let window = ctx.window.as_mut().unwrap();
    let mut handled = false;

    if window.left_mouse.pressed && window.left_mouse.inital_position.is_some_and(|p| p.intersects(hitbox)) {
        let click_y = window.mouse_position.y.saturating_sub(viewport.y) as f32;

        // This maps the mouse position to a 0.0 - 1.0 ratio of the entire track,
        // preventing the thumb from jumping to the cursor center.
        let ratio = (click_y / viewport.height as f32).clamp(0.0, 1.0);
        *scroll_y = (ratio * max_scroll as f32).round() as usize;

        handled = true;
    }

    if !handled && scroll_direction != 0 && window.mouse_position.intersects(viewport) {
        if scroll_direction > 0 {
            *scroll_y = scroll_y.saturating_add(50);
        } else {
            *scroll_y = scroll_y.saturating_sub(50);
        }
    }

    *scroll_y = (*scroll_y).clamp(0, max_scroll);

    let ratio = if max_scroll > 0 {
        *scroll_y as f32 / max_scroll as f32
    } else {
        0.0
    };
    let thumb_y = viewport.y + (ratio * track_h as f32) as usize;

    ctx.paint_rect(Rect::new(x, thumb_y, w, thumb_h), bg(rgb(80, 80, 80)));
}

fn dropdown_items(menu: Menu) -> &'static [&'static str] {
    match menu {
        Menu::File => &["New Project", "Open File...", "Save"],
        Menu::Edit => &["Undo", "Redo", "Cut"],
        Menu::View => &["Toggle Sidebar", "Zoom In"],
        Menu::Playback => &["Play / Pause", "Stop"],
        Menu::Library => &["Scan Folders..."],
        Menu::Help => &["Documentation", "About"],
    }
}

fn main() {
    defer_results!();
    let mut ctx = ctx("Basic", 1000, 700);
    ctx.default_font_size = 13;
    let mut current_menu: Option<(Menu, Rect)> = None;
    let mut volume = 0.5;

    let dropdown_width = 180;
    let dropdown_item_padtb = 8;
    let dropdown_item_height = ctx.default_font_size + dropdown_item_padtb * 2;

    let mut track_scroll_y = 0;

    let mut total_track_content_height: usize = 0;

    // let dark_bg = rgb(15, 15, 15);
    let panel_bg = rgb(10, 10, 10);
    let border_color = rgb(45, 45, 45);
    let accent_blue = rgb(0, 102, 204);
    let text_dim = rgb(170, 170, 170);
    let player_row_style = style()
        .pad(8)
        .padl(12)
        .bg(panel_bg)
        .hover(rgb(35, 35, 35))
        .hover_border(rgb(90, 90, 90))
        .selected(rgb(82, 82, 82))
        .selected_border(rgb(170, 170, 170));

    loop {
        let mut scroll_direction = 0;
        match ctx.window.as_mut().unwrap().event() {
            Some(event) => match event {
                Event::Quit => return,
                Event::Input(Key::Escape, _) => return,
                Event::Input(Key::ScrollDown, _) => scroll_direction = 1,
                Event::Input(Key::ScrollUp, _) => scroll_direction = -1,
                _ => {}
            },
            None => {}
        }

        let ctx_width = ctx.width();
        #[allow(unused)]
        let ctx_height = ctx.height();

        ctx.begin_ui(black());

        let (top_nav_rect, bottom_rect) = ctx.split_v(30);

        let sidebar_rect = Rect::new(bottom_rect.x, bottom_rect.y, 260, bottom_rect.height);
        let right_panel_rect = Rect::new(
            bottom_rect.x + 260,
            bottom_rect.y,
            bottom_rect.width.saturating_sub(260),
            bottom_rect.height,
        );

        if let Some((menu, rect)) = current_menu {
            let item_style = style()
                .padlr(12)
                .padtb(dropdown_item_padtb)
                .bg(rgb(35, 35, 35))
                .hover(rgb(60, 60, 60))
                .depth(1);

            let total_dropdown_height = dropdown_items(menu).len() * dropdown_item_height;
            let dropdown = Rect::new(rect.x, top_nav_rect.height, dropdown_width, total_dropdown_height);

            ctx.begin_layout_with_bounds(Flow::Down, dropdown);

            for &item in dropdown_items(menu) {
                if ctx.list_item(item, false, dropdown_width, item_style).clicked {
                    println!("{}", item);
                    current_menu = None;
                }
            }

            ctx.end_layout_absolute();

            let window = ctx.window.as_mut().unwrap();
            let left = &mut window.left_mouse;
            if let Some(inital) = left.inital_position
                && let Some(release) = left.release_position
            {
                if left.released && !inital.intersects(rect) && !release.intersects(rect) {
                    left.position = None;
                    left.released = false;
                    current_menu = None;
                }
            }
        }

        {
            let menu_style = style()
                .height(top_nav_rect.height)
                .padl(14)
                .padr(14)
                .bg(rgb(25, 25, 25))
                .hover(rgb(45, 45, 45));

            let items = [
                ("File", Menu::File),
                ("Edit", Menu::Edit),
                ("View", Menu::View),
                ("Playback", Menu::Playback),
                ("Library", Menu::Library),
                ("Help", Menu::Help),
            ];

            ctx.begin_layout_with_bounds(Flow::Right, top_nav_rect);

            for (label, menu) in items {
                let state = ctx.button(label, menu_style);
                if state.clicked {
                    if let Some((cm, _)) = current_menu
                        && cm == menu
                    {
                        current_menu = None;
                    } else {
                        current_menu = Some((menu, state.rect));
                    }
                }
            }

            let bar_style = style().width(1).height(top_nav_rect.height).bg(hex("#424242"));

            let gap = bar_style.bg(rgb(25, 25, 25)).width(120);
            ctx.spacer(bar_style);
            ctx.spacer(gap);
            ctx.spacer(bar_style);
            ctx.spacer(gap);
            ctx.spacer(bar_style);
            let frame = ctx.layout_stack.last().unwrap();
            ctx.spacer(gap.width(ctx_width - frame.cursor_x - 200 - 14));
            volume_slider(&mut ctx, &mut volume, 200, top_nav_rect.height);
            ctx.spacer(menu_style);

            ctx.end_layout();
        }

        {
            ctx.begin_layout_with_bounds(Flow::Down, right_panel_rect);

            let (album_header_rect, scroll_viewport) = ctx.split_v(35);

            ctx.begin_layout_with_bounds(Flow::Down, scroll_viewport);

            //Draw the panel background
            ctx.paint_rect(right_panel_rect, bg(panel_bg));

            scrollbar(
                &mut ctx,
                scroll_viewport,
                total_track_content_height,
                &mut track_scroll_y,
                scroll_direction,
            );

            ctx.begin_scroll_view(scroll_viewport, track_scroll_y);
            let tracklist: Vec<String> = (0..100).into_iter().map(|i| format!("track {i}")).collect();
            for (idx, track) in tracklist.into_iter().enumerate() {
                if ctx
                    .list_item(track, false, scroll_viewport.width - 20, player_row_style)
                    .clicked
                {
                    println!("Clicked item {idx}")
                }
            }

            total_track_content_height = ctx.end_scroll_view();

            ctx.end_layout();

            ctx.begin_layout_with_bounds(Flow::Down, album_header_rect);

            ctx.paint_rect(album_header_rect, bg(panel_bg));
            ctx.button(
                "beabadoobee - Fake It Flowers (2020)",
                style().fg(accent_blue).font_size(14).padl(8).padb(4),
            );

            ctx.end_layout();

            ctx.paint_rect(right_panel_rect.width(1), bg(border_color));
            ctx.end_layout();
        }

        //Sidebar
        {
            ctx.begin_layout_with_bounds(Flow::Down, sidebar_rect);
            ctx.paint_rect(sidebar_rect, bg(panel_bg));

            ctx.button("All Music", style().fg(text_dim).pad(6));

            let artists = [
                "Arca",
                "BADBADNOTGOOD",
                "beabadoobee",
                "Björk",
                "black midi",
                "Bonobo",
                "C418",
                "Daft Punk",
                "Death Grips ",
                "Duster ",
                "Flume",
            ];

            for artist in artists {
                ctx.list_item(artist, false, sidebar_rect.width - 10, player_row_style);
            }

            ctx.end_layout();
        }

        ctx.draw();
    }
}
