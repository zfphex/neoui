use neoui::*;

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
    let mut handled = false;

    if ui.dragged(hitbox) {
        let click_y = ui.mouse_position().y.saturating_sub(viewport.y) as f32;

        // This maps the mouse position to a 0.0 - 1.0 ratio of the entire track,
        // preventing the thumb from jumping to the cursor center.
        let ratio = (click_y / viewport.height as f32).clamp(0.0, 1.0);
        *scroll_y = (ratio * max_scroll as f32).round() as usize;

        handled = true;
    }

    if !handled && scroll_direction != 0 && ui.mouse_position().intersects(viewport) {
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

fn main() {
    defer_results!();

    let mut ui = ui("Basic", 1000, 700);
    ui.default_font_size = 13;

    let mut track_scroll_y = 0;
    let mut total_track_content_height: usize = 0;
    let mut selected_song = 0;

    loop {
        let mut scroll_direction = 0;
        match ui.window.event() {
            Some(event) => match event {
                Event::Quit => return,
                Event::Input(Key::Escape, _) => return,
                Event::Input(Key::ScrollDown, _) => scroll_direction = 1,
                Event::Input(Key::ScrollUp, _) => scroll_direction = -1,
                _ => {}
            },
            None => {}
        }

        let _ui_width = ui.width();
        let _ui_height = ui.height();

        ui.start_frame(black());

        let row_style = style()
            .pad(8)
            .padl(12)
            .hover(rgb(35, 35, 35))
            .fillw()
            .hover_border(rgb(90, 90, 90))
            .selected(rgb(82, 82, 82))
            .selected_border(rgb(170, 170, 170));

        //Yeah so the items were just not hiting y = 0 so it was a non issue before 🤣
        let (_, bounds) = ui.split_v(40);
        total_track_content_height = ui.scroll(Some(bounds), track_scroll_y, |ui| {
            let frame = ui.layout_stack.last().unwrap();
            scrollbar(
                ui,
                frame.bounds,
                total_track_content_height,
                &mut track_scroll_y,
                scroll_direction,
            );

            let tracklist: Vec<String> = (0..100).into_iter().map(|i| format!("track {i}")).collect();
            let row_style = row_style
                .align(Alignment::Left { pad: 12 })
                .width(ui.resolve_size(Size::FillMinus(20), Flow::Right));

            for (idx, track) in tracklist.into_iter().enumerate() {
                if ui.item(track, idx == selected_song, row_style).clicked {
                    selected_song = idx;
                    println!("Clicked item {idx}");
                }
            }
        });

        ui.draw_frame();
    }
}
