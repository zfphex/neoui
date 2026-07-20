use neoui::*;

fn fade(ui: &mut FrameContext, target: u32) -> u32 {
    ui.animate_color(target, 8.0)
}

fn main() {
    defer_results!();
    let mut ui = ui("Anim", 700, 520);
    ui.default_font_size = 14;

    let mut open = false;
    let mut selected = 0usize;
    let mut hover_a = false;
    let mut hover_b = false;
    let mut next_id = 4u64;
    let mut items = vec![(0u64, true), (1, false), (2, true), (3, false)];

    while ui.window.open() {
        ui.frame(|ui| {
            if ui.window.pressed(Key::Escape) {
                ui.window.close();
            }

            ui.flow_down(style().pad(12).gap(16), |ui| {
                ui.text("OK — distinct slots in a scope", fg(white()));
                let w = ui.animate_f32(if open { 240.0 } else { 80.0 }, 0.35, Ease::OutCubic);
                let bar = ui.text(
                    if open { "close" } else { "open" },
                    style().width(w as i32).height(36).bg(hex("#3b82f6")).fg(white()),
                );
                if bar.clicked {
                    open = !open;
                }

                ui.text("OK — helper calls (scope slots)", fg(hex("#4ade80")));
                ui.flow_right(style().gap(8), |ui| {
                    let ca = fade(ui, if hover_a { hex("#3b82f6") } else { hex("#52525b") });
                    let cb = fade(ui, if hover_b { hex("#ef4444") } else { hex("#52525b") });
                    hover_a = ui.text("A", style().width(80).height(36).bg(ca).fg(white())).hovered;
                    hover_b = ui.text("B", style().width(80).height(36).bg(cb).fg(white())).hovered;
                });

                ui.text("OK — one animate per nested scope", fg(hex("#4ade80")));
                for i in 0..3 {
                    ui.flow_down(style(), |ui| {
                        let h = ui.animate_f32(if selected == i { 48.0 } else { 28.0 }, 0.25, Ease::OutCubic);
                        let row = ui.text(
                            format!("row {i}"),
                            style()
                                .width(200)
                                .height(h as i32)
                                .bg(if selected == i { hex("#22c55e") } else { hex("#3f3f46") })
                                .fg(white()),
                        );
                        if row.clicked {
                            selected = i;
                        }
                    });
                }

                #[rustfmt::skip] 
                // Stable keys: insert/reorder keeps each item's animation state.
                ui.text("OK — with_id (stable keys)", fg(hex("#4ade80")));
                ui.flow_right(style().gap(8), |ui| {
                    if ui.text("+", style().width(28).height(28).bg(hex("#3f3f46")).fg(white())).clicked {
                        items.insert(0, (next_id, false));
                        next_id += 1;
                    }
                    if ui.text("rot", style().width(40).height(28).bg(hex("#3f3f46")).fg(white())).clicked {
                        items.rotate_left(1);
                    }
                    for (id, on) in items.iter_mut() {
                        ui.with_id(*id, |ui| {
                            let c = fade(ui, if *on { hex("#3b82f6") } else { hex("#52525b") });
                            if ui.text(format!("{id}"), style().width(48).height(32).bg(c).fg(white())).clicked {
                                *on = !*on;
                            }
                        });
                    }
                });
            });
        });
    }
}
