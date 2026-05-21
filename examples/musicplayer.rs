use neoui::*;

struct PlayerState {
    selected_artist_idx: usize,
    selected_track_idx: usize,
    volume: f32,
    playback_pos: f32,
}

fn slider(
    ctx: &mut Context,
    value: f32,
    min: f32,
    max: f32,
    width: usize,
    height: usize,
    style: Style,
) -> f32 {
    let window = ctx.window.as_mut().unwrap();

    let thumb_size = (height / 2).max(10).min(16);
    let track_height = (height / 6).max(2).min(4);
    let track_y_offset = (height - track_height) / 2;
    let thumb_y_offset = (height - thumb_size) / 2;

    let frame = ctx.layout_stack.last_mut().expect("No active layout frame");
    let x = frame.cursor_x;
    let y = frame.cursor_y;

    match frame.flow {
        Flow::Down => {
            frame.cursor_y += height;
            frame.max_child_width = frame.max_child_width.max(width);
            frame.max_child_height += height;
        }
        Flow::Right => {
            frame.cursor_x += width;
            frame.max_child_width += width;
            frame.max_child_height = frame.max_child_height.max(height);
        }
    }

    let hit_rect = Rect::new(x, y, width, height);
    let mut new_value = value;
    if window.left_mouse.pressed && window.mouse_position.intersects(hit_rect) {
        let mouse_x = window.mouse_position.x.clamp(x, x + width);
        let ratio = (mouse_x - x) as f32 / width as f32;
        new_value = (min + ratio * (max - min)).clamp(min, max);
    }

    let t = ((new_value - min) / (max - min)).clamp(0.0, 1.0);
    let thumb_x = x + (t * (width - thumb_size) as f32) as usize;
    let filled_w = thumb_x.saturating_sub(x);
    let track_y = y + track_y_offset;

    let track_bg = style.bg.unwrap_or_else(gray);
    let track_fill = style.hover.unwrap_or_else(|| rgb(0, 102, 204));
    let thumb_color = style.fg.unwrap_or_else(white);

    ctx.commands.push(Command::Rect {
        rect: Rect::new(x, track_y, width, track_height),
        color: track_bg,
    });
    ctx.commands.push(Command::Rect {
        rect: Rect::new(x, track_y, filled_w, track_height),
        color: track_fill,
    });
    ctx.commands.push(Command::Rect {
        rect: Rect::new(thumb_x, y + thumb_y_offset, thumb_size, thumb_size),
        color: thumb_color,
    });

    new_value
}

fn main() {
    let ctx = create_ctx("Music Player", 1000, 700, WindowStyle::DEFAULT);

    let mut state = PlayerState {
        selected_artist_idx: 2,
        selected_track_idx: 1,
        volume: 0.8,
        playback_pos: 0.71,
    };

    let dark_bg = rgb(15, 15, 15);
    let panel_bg = rgb(10, 10, 10);
    let border_color = rgb(45, 45, 45);
    let accent_blue = rgb(0, 102, 204);
    let text_dim = rgb(170, 170, 170);

    let player_row_style = style()
        .font_size(13)
        .pad(8)
        .bg(panel_bg)
        .hover(rgb(35, 35, 35))
        .hover_border(rgb(90, 90, 90))
        .selection(rgb(82, 82, 82))
        .selection_border(rgb(170, 170, 170));

    loop {
        if exit() {
            break;
        }

        let width = ctx.width();
        let height = ctx.height();

        begin_ui(dark_bg);

        begin_layout_with_bounds(Flow::Right, Rect::new(0, 0, width, 30));

        let menu_style = style()
            .font_size(13)
            .pad(6)
            .bg(dark_bg)
            .hover(rgb(25, 25, 25))
            .hover_border(rgb(170, 170, 170));

        ctx.button("File", menu_style);
        ctx.button("Edit", menu_style);
        ctx.button("View", menu_style);
        ctx.button("Playback", menu_style);
        ctx.button("Library", menu_style);
        ctx.button("Help", menu_style);
        end_layout();

        begin_layout_with_bounds(Flow::Right, Rect::new(0, 30, width, 40));
        let btn_style = style().font_size(12).pad(4).bg(rgb(30, 30, 30));
        ctx.button(" ⏹ ", btn_style);
        ctx.button(" ▶ ", btn_style);
        ctx.button(" ⏸ ", btn_style);
        ctx.button("  ⏮ ", btn_style);
        ctx.button(" ⏭ ", btn_style);

        ctx.button(" 🔀 ", btn_style);

        let slider_style = style().bg(rgb(45, 45, 45)).hover(accent_blue).fg(white());
        state.volume = slider(ctx, state.volume, 0.0, 1.0, 120, 20, slider_style);

        end_layout();

        ctx.rect(Rect::new(0, 70, width, 1), border_color);

        let content_y = 72;
        let content_h = height.saturating_sub(content_y);
        let sidebar_w = 260;
        let right_panel_w = width.saturating_sub(sidebar_w);

        begin_layout_with_bounds(Flow::Down, Rect::new(0, content_y, sidebar_w, content_h));
        ctx.rect(Rect::new(0, content_y, sidebar_w, content_h), panel_bg);

        ctx.button("All Music (89)", style().fg(text_dim).font_size(13));

        let artists = [
            "  Arca (1)",
            "  BADBADNOTGOOD (3)",
            "👉 beabadoobee (6)",
            "  Björk (7)",
            "  black midi (4)",
            "  Bonobo (1)",
            "  C418 (3)",
            "  Daft Punk (1)",
            "  Death Grips (11)",
            "  Duster (10)",
            "  Flume (6)",
        ];

        for (idx, artist) in artists.iter().enumerate() {
            if ctx.list_item(
                *artist,
                idx == state.selected_artist_idx,
                sidebar_w - 10,
                player_row_style,
            ) {
                state.selected_artist_idx = idx;
            }
        }
        end_layout();

        ctx.rect(Rect::new(sidebar_w, content_y, 2, content_h), border_color);

        let right_x = sidebar_w + 2;
        let meta_panel_h = 200;

        begin_layout_with_bounds(
            Flow::Down,
            Rect::new(right_x, content_y, right_panel_w, meta_panel_h),
        );

        ctx.rect(
            Rect::new(right_x, content_y, right_panel_w, meta_panel_h),
            panel_bg,
        );

        ctx.button(
            "Metadata Info Tracker",
            style().fg(accent_blue).font_size(13),
        );

        let metadata = [
            ("Artist Name", "beabadoobee"),
            ("Track Title", "Worth It"),
            ("Album Title", "Fake It Flowers"),
            ("Date", "2020"),
            ("Track Number", "2"),
            ("Disc Number", "1"),
        ];

        let row_height = 22;
        let grid_y = content_y + 25;
        let grid_w = right_panel_w - 20;
        let grid_h = metadata.len() * row_height;
        let meta_grid_bounds = Rect::new(right_x + 10, grid_y, grid_w, grid_h);

        let col_0_width = 120;
        let col_1_width = grid_w.saturating_sub(col_0_width);

        for (row_idx, (prop, val)) in metadata.iter().enumerate() {
            begin_grid_cell(
                0,
                row_idx,
                col_0_width,
                row_height,
                meta_grid_bounds,
                Flow::Right,
            );
            ctx.button(*prop, style().fg(text_dim).font_size(13).pad(4));
            end_layout();

            begin_grid_cell(
                1,
                row_idx,
                col_1_width,
                row_height,
                meta_grid_bounds,
                Flow::Right,
            );
            let display_val = format!("{}", val);
            ctx.button(display_val, style().fg(white()).font_size(13).pad(2));
            end_layout();
        }

        end_layout();

        let track_y = content_y + meta_panel_h;
        let track_panel_h = content_h.saturating_sub(meta_panel_h);
        ctx.rect(Rect::new(right_x, track_y, right_panel_w, 2), border_color);

        begin_layout_with_bounds(
            Flow::Down,
            Rect::new(right_x, track_y + 4, right_panel_w, track_panel_h),
        );
        ctx.rect(
            Rect::new(right_x, track_y + 4, right_panel_w, track_panel_h),
            panel_bg,
        );

        ctx.button(
            "beabadoobee - [2020] Fake It Flowers",
            style().fg(accent_blue).font_size(14),
        );

        let tracklist = [
            "1.01  Care",
            "1.02  Worth It",
            "1.04  Back To Mars",
            "1.05  Charlie Brown",
            "1.06  Emo Song",
            "1.07  Sorry",
            "1.08  Further Away",
        ];

        for (idx, track) in tracklist.iter().enumerate() {
            if ctx.list_item(
                *track,
                idx == state.selected_track_idx,
                right_panel_w - 20,
                player_row_style,
            ) {
                state.selected_track_idx = idx;
            }
        }
        end_layout();

        draw_cmd();
        ctx.draw();
    }
}
