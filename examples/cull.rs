use neoui::*;
use std::time::Instant;

const CARDS: usize = 2000;
const TRACKS: usize = 8;
const CARD_GAP: i32 = 16;
const COVER: i32 = 120;

fn tracks(index: usize) -> usize {
    TRACKS + index % 5
}

fn card_height(index: usize) -> i32 {
    COVER.max(34 + tracks(index) as i32 * 22)
}

fn main() {
    let mut ui = ui("cull", 1000, 820);
    ui.default_font_size = 13;
    ui.vsync = false;

    let mut scroll = Scroll::new();
    let mut cull = true;
    let mut built = 0usize;
    let mut build_ms = 0.0f64;
    let mut frame_ms = 0.0f64;
    let mut max_scroll = 0;
    let mut clicked = String::from("nothing yet");

    let bg = rgb(18, 18, 20);
    let panel = rgb(26, 26, 30);
    let line = rgb(48, 48, 54);
    let dim = rgb(150, 150, 160);
    let accent = rgb(120, 180, 255);

    while ui.window.open() {
        let frame_start = Instant::now();
        ui.frame(|ui| {
            ui.clear_color = bg;
            if ui.window.pressed(Key::Char('c')) {
                cull = !cull;
            }

            if ui.window.pressed(Key::Escape) {
                ui.window.close();
            }

            let (bar, body) = ui.split_v(76);
            let build_start = Instant::now();
            built = 0;

            max_scroll = ui
                .scroll(bounds(body).bg(panel).elastic(true), &mut scroll, |ui| {
                    for index in 0..CARDS {
                        let height = card_height(index);
                        let card = style().padlr(16).padtb(8).fill_width().height(height);
                        let body = style().fill_width().height(height);
                        let (card, body) = if cull {
                            (card, body)
                        } else {
                            (card.skip_cull(), body.skip_cull())
                        };

                        ui.flow_right(card, |ui| {
                            built += 1;
                            let layout = ui.walk_layout(COVER, COVER, 0);
                            let cover = Rect::new(layout.paint_x, layout.paint_y, COVER, COVER);
                            ui.paint_rect(cover, style().bg(rgb(58, 58, 66)).radius(8));
                            if ui.clicked(cover) {
                                clicked = format!("cover of album {index}");
                            }
                            ui.gap(16);

                            ui.flow_down(body, |ui| {
                                let title = ui.fmt(format_args!("Album {index}"));
                                ui.text(title, style().fg(white()).font_size(18).padb(6).fill_width());
                                for track in 0..tracks(index) {
                                    let label = ui.fmt(format_args!("{}.  Track title {index}-{track}", track + 1));
                                    let row = ui.item(
                                        label,
                                        style()
                                            .fg(dim)
                                            .padlr(6)
                                            .padtb(3)
                                            .radius(4)
                                            .fill_width()
                                            .hover(rgb(44, 44, 52)),
                                    );
                                    if row.clicked {
                                        clicked = format!("track {track} of album {index}");
                                    }
                                }
                            });
                        });
                        ui.gap(CARD_GAP);
                    }
                })
                .max_scroll;
            build_ms = build_start.elapsed().as_secs_f64() * 1000.0;

            ui.place_down(bounds(bar).bg(panel), |ui| {
                ui.text(
                    "c: toggle culling — click a cover or a track, then try clicking this bar",
                    style().fg(dim).padl(16).padt(10).padb(4).fill_width(),
                );
                ui.rect(style().fill_width().height(1).bg(line));
                ui.place_right(style().padl(16).padt(8).fill_width().height(28), |ui| {
                    let stats = ui.fmt(format_args!(
                        "cull {} · built {built}/{CARDS} in {build_ms:.2}ms · max_scroll {max_scroll} · frame {frame_ms:.2}ms",
                        if cull { "on" } else { "off" }
                    ));
                    ui.text(stats, style().fg(white()).padr(24));
                    let last = ui.fmt(format_args!("last click: {clicked}"));
                    ui.text(last, style().fg(accent));
                });
            });
        });
        frame_ms = frame_start.elapsed().as_secs_f64() * 1000.0;
    }
}
