use neoui::*;
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Menu {
    File,
    Edit,
    View,
    Playback,
    Library,
    Help,
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
    let mut selected_song = 0;
    let mut volume = 0.5;
    let mut scroll_y = 0;

    let panel_bg = rgb(10, 10, 10);
    let border_color = rgb(45, 45, 45);
    let bar_color = rgb(66, 66, 66);
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

    let tracklist: Vec<String> = (0..100).into_iter().map(|i| format!("track {i}")).collect();
    let icons = fontdue::Font::from_bytes(
        include_bytes!("../fonts/MaterialIcons-Regular.ttf") as &[u8],
        fontdue::FontSettings::default(),
    )
    .unwrap();
    let icon_id = ui.add_font(icons);

    let play = "\u{e037}";
    let pause = "\u{e034}";
    let stop = "\u{e047}";

    #[cfg(feature = "profile")]
    let now = std::time::Instant::now();

    while ui.window.open() {
        #[cfg(feature = "profile")]
        if now.elapsed() > std::time::Duration::from_secs(2) {
            ui.window.close();
        }

        ui.frame(|ui| {
            let (top_nav_rect, body) = ui.split_v(30);
            let (sidebar_rect, track_rect) = ui.split_rect_h(body, 260);

            ui.flow_right(bounds(top_nav_rect).bg(menu_bg), |ui| {
                for (label, menu) in items {
                    let state = ui.text(
                        label,
                        style()
                            .height(top_nav_rect.height)
                            .padl(14)
                            .padr(14)
                            .bg(menu_bg)
                            .hover(menu_hover),
                    );

                    if state.clicked {
                        if current_menu.is_some_and(|(cm, _)| cm == menu) {
                            current_menu = None;
                        } else {
                            current_menu = Some((menu, state.rect));
                        }
                    }
                }

                let bar = style().width(1).height(top_nav_rect.height).bg(bar_color);
                let icon = style().font(icon_id).font_size(24).fill_height();
                if ui.text(stop, icon).clicked {
                    println!("Stop");
                }

                if ui.text(play, icon).clicked {
                    println!("Play")
                }

                if ui.text(pause, icon).clicked {
                    println!("Pause")
                }

                ui.gap(10);

                ui.rect(bar);
                ui.gap(120);

                ui.rect(bar);
                ui.gap(120);

                ui.gap(-214);

                //Volume slider.
                {
                    let width = 200;
                    let height = top_nav_rect.height;
                    let rect = ui.walk_layout(width, height, 0).size;

                    ui.paint_rect(rect, bg(rgb(25, 25, 25)));

                    if let Some(percent) = ui.drag_percentage_x(rect) {
                        volume = percent;
                    }

                    let track_height = 6;
                    let cy = rect.y + height / 2;

                    ui.paint_triangle(
                        (rect.x, cy + track_height),
                        (rect.x + width, cy + track_height),
                        (rect.x + width, cy.saturating_sub(track_height)),
                        bg(black()),
                    );

                    let thumb_w = 12;
                    let thumb_h = 18;
                    let available_width = width.saturating_sub(thumb_w);
                    let thumb_x = rect.x + (volume * available_width as f32).round() as usize;
                    let thumb_y = rect.y + (height.saturating_sub(thumb_h)) / 2;
                    let thumb_color = rgb(0, 102, 204);

                    ui.paint_rect(Rect::new(thumb_x, thumb_y, thumb_w, thumb_h), bg(thumb_color));
                }
            });

            //Should go after the hit testing so there is not a one frame delay.
            if let Some((menu, rect)) = current_menu {
                let item_style = style()
                    .width(180)
                    .padlr(12)
                    .padtb(8)
                    .bg(rgb(35, 35, 35))
                    .hover(rgb(60, 60, 60))
                    .align(Alignment::Left)
                    .depth(1);

                ui.flow_skip(style().x(rect.x).y(top_nav_rect.height), Flow::Down, |ui| {
                    for &item in dropdown_items(menu) {
                        if ui.item(item, false, item_style).clicked {
                            println!("{}", item);
                            current_menu = None;
                        }
                    }
                });

                if ui.lost_focus(rect) {
                    current_menu = None;
                }
            }

            let row_style = style()
                .pad(8)
                .padl(12)
                .hover(rgb(35, 35, 35))
                .fill_width()
                .hover_border(rgb(90, 90, 90))
                .selected(rgb(82, 82, 82))
                .align(Alignment::Left)
                .selected_border(rgb(170, 170, 170));

            ui.flow_down(bounds(sidebar_rect).bg(panel_bg), |ui| {
                ui.text("All Music", style().fg(text_dim).pad(6));

                for artist in artists {
                    ui.item(artist, false, row_style);
                }

                ui.paint_rect(sidebar_rect, style().border(border_color).border_side(RIGHT));
            });

            //This is kinda cursed.
            let (track_rect, scrollbar) = ui.split_rect_h(track_rect, Size::FillMinus(20));

            let state = ui.scroll_view(track_rect, &mut scroll_y, |ui| {
                ui.text(
                    "beabadoobee - Fake It Flowers (2020)",
                    style().fg(accent_blue).font_size(14).padl(8).padb(4).height(24),
                );

                let row_style = row_style.align(Alignment::Left).padl(12);

                for (idx, track) in tracklist.iter().enumerate() {
                    if ui.item(Cow::from(track), idx == selected_song, row_style).clicked {
                        selected_song = idx;
                        println!("Clicked item {idx}");
                    }
                }
            });

            {
                let s = scrollbar.inner(4, 0);
                let (y, h) = (s.y as f32, s.height as f32);
                let thumb_h = 80.0;
                // Calculate the exact space the bar can move within.
                let available_height = (h - thumb_h).max(0.0);
                let mut ratio = (scroll_y as f32 / state.max_scroll as f32).clamp(0.0, 1.0);

                if ui.dragged(scrollbar) {
                    // Offset the mouse position by half the bar height so the drag centers on the thumb.
                    let mousey = ui.mouse_position().y as f32 - y - (thumb_h / 2.0);
                    ratio = (mousey / available_height).clamp(0.0, 1.0);
                    scroll_y = (ratio * state.max_scroll as f32).round() as usize;
                }

                let y = s.y + (ratio * available_height).round() as usize;
                let thumb = Rect::new(s.x, y, s.width, thumb_h as usize);
                ui.paint_rect(thumb, bg(rgb(80, 80, 80)));
            }
        });

        if ui.window.pressed(Key::Escape) {
            ui.window.close();
        }
    }
}
