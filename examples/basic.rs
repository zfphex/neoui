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

fn volume_slider(ui: &mut Context, volume: &mut f32, width: usize, height: usize) {
    let rect = ui.walk_layout(width, height);
    ui.paint_rect(rect, bg(rgb(25, 25, 25)));
    let window = ui.window.as_ref().unwrap();
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

    ui.paint_triangle(
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
    ui.paint_rect(Rect::new(thumb_x, thumb_y, thumb_w, thumb_h), bg(thumb_color));
}

pub fn scrollbar(ui: &mut Context, viewport: Rect, content_height: usize, scroll_y: &mut usize, scroll_direction: i32) {
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

    let window = ui.window.as_mut().unwrap();
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

    ui.paint_rect(Rect::new(x, thumb_y, w, thumb_h), bg(rgb(80, 80, 80)));
}

const fn dropdown_items(menu: Menu) -> &'static [&'static str] {
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

    let mut ui = ui("Basic", 1000, 700);
    ui.default_font_size = 13;

    let mut current_menu: Option<(Menu, Rect)> = None;
    let mut volume = 0.5;

    let mut track_scroll_y = 0;
    let mut total_track_content_height: usize = 0;

    let panel_bg = rgb(10, 10, 10);
    let border_color = rgb(45, 45, 45);
    let accent_blue = rgb(0, 102, 204);
    let text_dim = rgb(170, 170, 170);
    let menu_bg = rgb(25, 25, 25);
    let menu_hover = rgb(45, 45, 45);
    let items = [
        ("File", Menu::File),
        ("Edit", Menu::Edit),
        ("View", Menu::View),
        ("Playback", Menu::Playback),
        ("Library", Menu::Library),
        ("Help", Menu::Help),
    ];

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

    loop {
        let mut scroll_direction = 0;
        match ui.window.as_mut().unwrap().event() {
            Some(event) => match event {
                Event::Quit => return,
                Event::Input(Key::Escape, _) => return,
                Event::Input(Key::ScrollDown, _) => scroll_direction = 1,
                Event::Input(Key::ScrollUp, _) => scroll_direction = -1,
                _ => {}
            },
            None => {}
        }

        let ui_width = ui.width();
        let _ui_height = ui.height();

        ui.begin_frame(black());

        let (top_nav_rect, body) = ui.split_v(30);
        let (sidebar_rect, track_rect) = body.split_h(260);

        if let Some((menu, rect)) = current_menu {
            let item_style = style()
                .width(180)
                .padlr(12)
                .padtb(8)
                .bg(rgb(35, 35, 35))
                .hover(rgb(60, 60, 60))
                .depth(1);

            ui.begin_layout(Flow::Down, None);

            //Update the postion and height. TODO: Should make this easier to do.
            let last = ui.layout_stack.last_mut().unwrap();
            last.cursor_x = rect.x;
            last.cursor_y = top_nav_rect.height;

            for &item in dropdown_items(menu) {
                if ui.list_item(item, false, item_style).clicked {
                    println!("{}", item);
                    current_menu = None;
                }
            }
            ui.end_layout_absolute();

            let window = ui.window.as_mut().unwrap();
            let left = &mut window.left_mouse;
            if let Some(inital) = left.inital_position
                && let Some(release) = left.release_position
            {
                if left.released && !inital.intersects(rect) && !release.intersects(rect) {
                    left.inital_position = None;
                    left.released = false;
                    current_menu = None;
                }
            }
        }

        let menu_style = style()
            .height(top_nav_rect.height)
            .padl(14)
            .padr(14)
            .bg(menu_bg)
            .hover(menu_hover);

        ui.flow_right(top_nav_rect, |ui| {
            for (label, menu) in items {
                let state = ui.text(label, menu_style);
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
            let gap = bar_style.bg(menu_bg).width(120);
            ui.rect(bar_style);
            ui.rect(gap);
            ui.rect(bar_style);
            ui.rect(gap);
            ui.rect(bar_style);
            let frame = ui.layout_stack.last().unwrap();

            ui.rect(gap.width(ui_width - frame.cursor_x - 200 - 14));
            volume_slider(ui, &mut volume, 200, top_nav_rect.height);
            ui.spacer(bg(menu_bg));
        });

        let player_row_style = style()
            .pad(8)
            .padl(12)
            .bg(panel_bg)
            .hover(rgb(35, 35, 35))
            .hover_border(rgb(90, 90, 90))
            .selected(rgb(82, 82, 82))
            .selected_border(rgb(170, 170, 170));

        ui.flow_down(sidebar_rect, |ui| {
            ui.paint_rect(sidebar_rect, bg(panel_bg));

            ui.text("All Music", style().fg(text_dim).pad(6));

            for artist in artists {
                ui.list_item(artist, false, player_row_style);
            }
        });

        ui.flow_down(track_rect, |ui| {
            //TODO: I want to create this header which is based on the font size and not some fixed pixel width.
            let (album_header_rect, scroll_viewport) = ui.split_v(24);

            ui.flow_down(album_header_rect, |ui| {
                ui.paint_rect(album_header_rect, bg(panel_bg));
                ui.text(
                    "beabadoobee - Fake It Flowers (2020)",
                    style().fg(accent_blue).font_size(14).padl(8).padb(4),
                );
            });

            total_track_content_height = ui.scroll(scroll_viewport, Flow::Down, |ui| {
                scrollbar(
                    ui,
                    scroll_viewport,
                    total_track_content_height,
                    &mut track_scroll_y,
                    scroll_direction,
                );

                ui.begin_scroll_view(scroll_viewport, track_scroll_y);
                let tracklist: Vec<String> = (0..100).into_iter().map(|i| format!("track {i}")).collect();
                let player_row_style = player_row_style.width(ui.resolve_size(Size::RemainingMinus(20), true));
                for (idx, track) in tracklist.into_iter().enumerate() {
                    if ui.list_item(track, false, player_row_style).clicked {
                        println!("Clicked item {idx}")
                    }
                }
            });
        });

        ui.paint_rect(track_rect.width(1), bg(border_color));

        ui.draw_frame();
    }
}
