use neoui::*;

fn main() {
    defer_results!();

    let mut ui = ui("Basic", 1000, 700);
    ui.default_font_size = 13;

    let mut scroll_y = 0;
    let mut selected_song = 0;
    let tracklist: Vec<String> = (0..100).into_iter().map(|i| format!("track {i}")).collect();

    loop {
        if let Some(event) = ui.poll_event() {
            match event {
                Event::Quit | Event::Input(Key::Escape, _) => break,
                _ => {}
            }
        }

        let _ui_width = ui.width();
        let _ui_height = ui.height();

        ui.start_frame(black());

        let row_style = style()
            .pad(8)
            .padl(12)
            .hover(rgb(35, 35, 35))
            .fill_width()
            .hover_border(rgb(90, 90, 90))
            .selected(rgb(82, 82, 82))
            .selected_border(rgb(170, 170, 170));

        let (body, scrollbar) = ui.split_h(Size::FillMinus(20));
        let state = ui.scroll_view(bounds(body), &mut scroll_y, |ui| {
            let row_style = row_style.align(Alignment::Left).padl(12).width(body.width);

            for (idx, track) in tracklist.iter().enumerate() {
                if ui.item(track, idx == selected_song, row_style).clicked {
                    selected_song = idx;
                    println!("Clicked item {idx}");
                }
            }
        });

        {
            let scrollbar = scrollbar.inner(4, 0);
            let bar_height = 80;
            let mid_bar = bar_height as f32 / 2.0;
            let mut ratio = scroll_y as f32 / state.max_scroll as f32;

            // TODO: The bar cannot be dragged to the absolute top or bottom (cutoff just before).
            if ui.dragged(scrollbar) {
                ratio = ((ui.mouse_position().y as f32 + mid_bar) / scrollbar.height as f32).clamp(0.0, 1.0);
                scroll_y = (((ratio * state.max_scroll as f32) - mid_bar) as usize).clamp(0, state.max_scroll);
            }

            let y = scrollbar.y + (ratio * scrollbar.height as f32 - 84.0) as usize;
            let bar = Rect::new(scrollbar.x, y, scrollbar.width, bar_height);
            ui.paint_rect(bar, bg(rgb(80, 80, 80)));
        }

        ui.draw_frame();
    }
}
