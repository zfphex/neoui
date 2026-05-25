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
    let ctx = create_ctx("Basic", 1000, 700);
    let mut current_menu: Option<(Menu, Rect)> = None;
    let mut volume = 0.5;

    let dropdown_width = 180;
    let dropdown_item_font_size = 13;
    let dropdown_item_padtb = 8;
    let dropdown_item_height = dropdown_item_font_size + dropdown_item_padtb * 2;

    loop {
        if exit() {
            break;
        }

        let ctx_width = ctx.width();
        let ctx_height = ctx.height();

        begin_ui(black());

        let (top_nav_rect, _) = ctx.split_v(30);

        let dropdown_rect = current_menu.map(|(menu, button_rect)| {
            let h = dropdown_items(menu).len() * dropdown_item_height;
            Rect::new(button_rect.x, top_nav_rect.height, dropdown_width, h)
        });

        if let Some(rect) = dropdown_rect {
            ctx.block_input(rect);
        }

        //Menu Items
        {
            let menu_style = style()
                .font_size(13)
                .height(top_nav_rect.height)
                .padl(14)
                .padr(14)
                .bg(rgb(25, 25, 25))
                .hover(rgb(45, 45, 45));

            begin_layout_with_bounds(Flow::Right, top_nav_rect);

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
            volume_slider(ctx, &mut volume, 200, top_nav_rect.height);
            ctx.spacer(menu_style);

            end_layout();
        }

        let dark_bg = rgb(15, 15, 15);
        let panel_bg = rgb(10, 10, 10);
        let border_color = rgb(45, 45, 45);
        let accent_blue = rgb(0, 102, 204);
        let text_dim = rgb(170, 170, 170);
        let player_row_style = style()
            .font_size(13)
            .pad(8)
            .padl(12)
            .bg(panel_bg)
            .hover(rgb(35, 35, 35))
            .hover_border(rgb(90, 90, 90))
            .selection(rgb(82, 82, 82))
            .selection_border(rgb(170, 170, 170));

        let (sidebar_rect, right_panel_rect) = ctx.split_h(260);

        //Sidebar
        {
            begin_layout_with_bounds(Flow::Down, sidebar_rect);
            ctx.rect(sidebar_rect, panel_bg);

            ctx.button("All Music", style().fg(text_dim).font_size(13).pad(6));

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

            end_layout();
        }

        //Main panel
        {
            begin_layout_with_bounds(Flow::Down, right_panel_rect);

            ctx.rect(right_panel_rect, panel_bg);

            ctx.button(
                "beabadoobee - Fake It Flowers (2020)",
                style().fg(accent_blue).font_size(14).padl(8).padb(4),
            );

            let tracklist = [
                "1.01   Care",
                "1.02   Worth It",
                "1.04   Back To Mars",
                "1.05   Charlie Brown",
                "1.06   Emo Song",
                "1.07   Sorry",
                "1.08   Further Away",
            ];

            for track in tracklist {
                ctx.list_item(track, false, right_panel_rect.width - 20, player_row_style);
            }

            ctx.rect(right_panel_rect.width(1), border_color);

            end_layout();
        }

        //Drop down menu
        if let (Some((menu, _)), Some(rect)) = (current_menu, dropdown_rect) {
            let item_style = style()
                .font_size(dropdown_item_font_size)
                .padlr(12)
                .padtb(dropdown_item_padtb)
                .bg(rgb(35, 35, 35))
                .hover(rgb(60, 60, 60));

            begin_overlay(Flow::Down, rect);

            for &item in dropdown_items(menu) {
                if ctx.list_item(item, false, dropdown_width, item_style).clicked {
                    println!("{}", item);
                    current_menu = None;
                }
            }

            end_overlay();

            if ctx.clicked(Rect::new(0, 0, ctx_width, ctx_height)) {
                current_menu = None;
            }
        }

        draw_cmd();
        ctx.draw();
    }
}
