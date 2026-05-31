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
    ctx.rect(rect, 0, rgb(25, 25, 25));
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
        0,
        hex("#000000"),
    );

    let thumb_w = 12;
    let thumb_h = 18;

    let available_width = width.saturating_sub(thumb_w);
    let thumb_x = rect.x + (*volume * available_width as f32).round() as usize;
    let thumb_y = rect.y + (height.saturating_sub(thumb_h)) / 2;

    let thumb_color = hex("#0078D7");
    ctx.rect(Rect::new(thumb_x, thumb_y, thumb_w, thumb_h), 0, thumb_color);
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
    const SCROLL_SPEED: usize = 50;
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
        .selection(rgb(82, 82, 82))
        .selection_border(rgb(170, 170, 170));

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

            let window = ctx.window.as_mut().unwrap();
            let max_scroll = total_track_content_height.saturating_sub(scroll_viewport.height);

            let scrollbar_w = 6;
            let scrollbar_x = scroll_viewport.x + scroll_viewport.width - scrollbar_w - 4;
            let scrollbar_rect = Rect::new(scrollbar_x.saturating_sub(4), scroll_viewport.y, scrollbar_w + 8, scroll_viewport.height);

            let mut dragging_scrollbar = false;
            if window.left_mouse.pressed {
                if let Some(initial) = window.left_mouse.inital_position {
                    if initial.intersects(scrollbar_rect) {
                        dragging_scrollbar = true;
                    }
                }
            }

            if dragging_scrollbar {
                let click_y = window.mouse_position.y.saturating_sub(scroll_viewport.y);
                let visible_ratio = scroll_viewport.height as f32 / total_track_content_height.max(1) as f32;
                let thumb_h = (scroll_viewport.height as f32 * visible_ratio).max(20.0) as usize;
                
                let track_h = scroll_viewport.height.saturating_sub(thumb_h);
                let thumb_y = click_y.saturating_sub(thumb_h / 2);
                let scroll_ratio = if track_h > 0 { thumb_y as f32 / track_h as f32 } else { 0.0 };
                
                track_scroll_y = (scroll_ratio * max_scroll as f32).round() as usize;
                track_scroll_y = track_scroll_y.clamp(0, max_scroll);
            } else if window.mouse_position.intersects(right_panel_rect) && scroll_direction != 0 {
                if scroll_direction > 0 {
                    track_scroll_y = (track_scroll_y + SCROLL_SPEED).min(max_scroll);
                } else {
                    track_scroll_y = track_scroll_y.saturating_sub(SCROLL_SPEED);
                }
            }

            //Draw the panel background
            ctx.rect(right_panel_rect, 0, panel_bg);

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

            if total_track_content_height > scroll_viewport.height {
                let scrollbar_w = 6;
                let scrollbar_x = scroll_viewport.x + scroll_viewport.width - scrollbar_w - 4;
                let visible_ratio = scroll_viewport.height as f32 / total_track_content_height as f32;
                let thumb_h = (scroll_viewport.height as f32 * visible_ratio).max(20.0) as usize;
                
                let max_scroll = total_track_content_height.saturating_sub(scroll_viewport.height);
                let scroll_ratio = if max_scroll > 0 { track_scroll_y as f32 / max_scroll as f32 } else { 0.0 };
                let thumb_y = scroll_viewport.y + (scroll_ratio * (scroll_viewport.height.saturating_sub(thumb_h)) as f32) as usize;

                ctx.rect(Rect::new(scrollbar_x, thumb_y, scrollbar_w, thumb_h), 1, rgb(80, 80, 80));
            }

            ctx.end_layout();

            ctx.begin_layout_with_bounds(Flow::Down, album_header_rect);

            ctx.button(
                "beabadoobee - Fake It Flowers (2020)",
                style().fg(accent_blue).font_size(14).padl(8).padb(4),
            );

            ctx.end_layout();

            ctx.rect(right_panel_rect.width(1), 0, border_color);
            ctx.end_layout();
        }

        //Sidebar
        {
            ctx.begin_layout_with_bounds(Flow::Down, sidebar_rect);
            ctx.rect(sidebar_rect, 0, panel_bg);

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
