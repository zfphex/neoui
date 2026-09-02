use neoui::*;

fn main() {
    defer_results!();

    let mut ui = ui("SSPA Accessibility Demo", 980, 680);
    ui.default_font_size = 13;

    let bold_bytes = include_bytes!("../fonts/Aptos-Bold.ttf");
    let bold_font = fontdue::Font::from_bytes(bold_bytes as &[u8], fontdue::FontSettings::default()).unwrap();
    ui.add_face(Font::default().id, Weight::Bold, false, bold_font);

    let grid_items = [
        "Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot", "Golf", "Hotel", "India",
    ];

    let mut task_counter = 4usize;
    let mut tasks = vec![
        "Task 1: Spatial geometric focus".to_string(),
        "Task 2: Zero cross-frame IDs".to_string(),
        "Task 3: Weak signature shift snap".to_string(),
    ];

    let mut event_logs: Vec<String> = vec!["Ready. Use Tab or Arrow keys to navigate.".to_string()];

    while ui.window.open() {
        ui.frame(|ui| {
            if ui.window.pressed(Key::Escape) {
                ui.window.close();
            }

            let bg_main = hex("#0f172a");
            let panel_bg = hex("#1e293b");
            let border_col = hex("#334155");
            let accent_blue = hex("#3b82f6");
            let text_dim = hex("#94a3b8");
            let text_bright = hex("#f8fafc");

            let window_bounds = ui.current_frame_bounds();
            ui.paint_rect(window_bounds, rect().bg(bg_main));

            let (header_rect, content_rect) = ui.split_v(44);
            let (left_pane, right_pane) = ui.split_rect_h(content_rect, 580);

            ui.flow_right(flow().bounds(header_rect).padlr(16).bg(hex("#090d16")), |ui| {
                ui.text(
                    "Spatial-Stream Point Anchor (SSPA) Accessibility Demo",
                    text().fg(text_bright).font_size(15).bold().padtb(12),
                );
                ui.text(
                    "— ID-Free Immediate Mode Focus Engine",
                    text().fg(text_dim).font_size(13).padtb(12).padl(8),
                );
            });

            ui.flow_down(flow().bounds(left_pane).pad(16).gap(16), |ui| {
                ui.text(
                    "1. 2D SPATIAL & TAB NAVIGATION (3x3 GRID)",
                    text().fg(hex("#38bdf8")).bold(),
                );
                ui.text(
                    "Use Arrow Keys for 2D directional cone navigation, or Tab/Shift-Tab for sequential cycling.",
                    text().fg(text_dim),
                );

                for row in 0..3 {
                    ui.flow_right(flow().gap(12), |ui| {
                        for col in 0..3 {
                            let idx = row * 3 + col;
                            let label = grid_items[idx];

                            let btn_style = text()
                                .width(160)
                                .height(38)
                                .pad(8)
                                .bg(hex("#1e293b"))
                                .hover(hex("#334155"))
                                .fg(text_bright)
                                .content(Alignment::Center)
                                .border(border_col)
                                .border_thickness(1)
                                .radius(4);

                            let state = ui.text(label, btn_style);

                            // Highlight focus ring if SSPA active
                            if state.focused {
                                ui.paint_rect(state.bounds, rect().border(accent_blue).border_thickness(2).radius(4));
                            }

                            if state.activated {
                                let trigger = if state.clicked { "Mouse Click" } else { "Keyboard Enter/Space" };
                                if event_logs.len() >= 8 {
                                    event_logs.remove(0);
                                }
                                event_logs.push(format!("[ACTIVATED] Grid \"{label}\" via {trigger}"));
                            }
                        }
                    });
                }

                ui.gap(8);

                ui.text(
                    "2. DYNAMIC MUTATION STABILITY (ZERO ID KEYS)",
                    text().fg(hex("#4ade80")).bold(),
                );
                ui.text(
                    "Test list reflows. SSPA retains focus across mutations without manual push_id() scopes.",
                    text().fg(text_dim),
                );

                ui.flow_right(flow().gap(8), |ui| {
                    let tool_btn = text()
                        .padtb(6)
                        .padlr(12)
                        .bg(hex("#334155"))
                        .hover(hex("#475569"))
                        .fg(text_bright)
                        .radius(4)
                        .content(Alignment::Center);

                    let prepend_state = ui.text("+ Prepend Item (Tier 2 Shift)", tool_btn);
                    if prepend_state.focused {
                        ui.paint_rect(
                            prepend_state.bounds,
                            rect().border(accent_blue).border_thickness(2).radius(4),
                        );
                    }
                    if prepend_state.activated {
                        tasks.insert(0, format!("Task {task_counter}: Dynamically prepended"));
                        task_counter += 1;
                        if event_logs.len() >= 8 {
                            event_logs.remove(0);
                        }
                        event_logs.push(
                            "[MUTATION] Prepended item -> Focus automatically tracks shifted elements".to_string(),
                        );
                    }

                    let rename_state = ui.text("Rename Label (Tier 1b)", tool_btn);
                    if rename_state.focused {
                        ui.paint_rect(
                            rename_state.bounds,
                            rect().border(accent_blue).border_thickness(2).radius(4),
                        );
                    }
                    if rename_state.activated {
                        if let Some(cursor) = ui.focus_cursor() {
                            let (cx, cy) = cursor.point;
                            for task in &mut tasks {
                                if task.contains("Task") {
                                    if task.ends_with(" [EDITED]") {
                                        *task = task.replace(" [EDITED]", "");
                                    } else {
                                        task.push_str(" [EDITED]");
                                    }
                                    break;
                                }
                            }
                            if event_logs.len() >= 8 {
                                event_logs.remove(0);
                            }
                            event_logs.push(format!(
                                "[MUTATION] Renamed label at ({cx:.0}, {cy:.0}) -> Tier 1b retains focus"
                            ));
                        }
                    }

                    let delete_state = ui.text(
                        "- Delete Selected (Tier 3)",
                        tool_btn.bg(hex("#7f1d1d")).hover(hex("#991b1b")),
                    );
                    if delete_state.focused {
                        ui.paint_rect(
                            delete_state.bounds,
                            rect().border(hex("#ef4444")).border_thickness(2).radius(4),
                        );
                    }
                    if delete_state.activated {
                        if !tasks.is_empty() {
                            let removed = tasks.remove(0);
                            if event_logs.len() >= 8 {
                                event_logs.remove(0);
                            }
                            event_logs.push(format!(
                                "[MUTATION] Deleted \"{removed}\" -> Tier 3 snapped to neighbor"
                            ));
                        }
                    }
                });

                ui.flow_down(flow().fillw().gap(6).padtb(4), |ui| {
                    for (i, task) in tasks.iter().enumerate() {
                        let row_style = text()
                            .padtb(8)
                            .padlr(12)
                            .fillw()
                            .bg(hex("#1e293b"))
                            .hover(hex("#273549"))
                            .fg(text_bright)
                            .radius(4)
                            .border(border_col)
                            .border_thickness(1)
                            .content(Alignment::Left);

                        let task_label = ui.fmt(format_args!("{task}"));
                        let state = ui.text(task_label, row_style);

                        if state.focused {
                            ui.paint_rect(
                                state.bounds,
                                rect().border(hex("#60a5fa")).border_thickness(2).radius(4),
                            );
                        }

                        if state.activated {
                            let trigger = if state.clicked { "Mouse" } else { "Keyboard" };
                            if event_logs.len() >= 8 {
                                event_logs.remove(0);
                            }
                            event_logs.push(format!("[ACTIVATED] Item #{i} \"{task}\" via {trigger}"));
                        }
                    }
                });
            });

            ui.flow_down(flow().bounds(right_pane).pad(16).gap(14).bg(panel_bg), |ui| {
                ui.text("SSPA LIVE TELEMETRY", text().fg(hex("#a78bfa")).bold());

                let cursor_opt = ui.focus_cursor();
                let total_nodes = ui.state.accessability_state.prev_nodes.len();

                let (px, py, stream_idx, role_bits, text_sig, depth) = if let Some(cursor) = cursor_opt {
                    (
                        format!("{:.1}", cursor.point.0),
                        format!("{:.1}", cursor.point.1),
                        format!("{} / {}", cursor.stream_index, total_nodes),
                        format!("0x{:04X}", cursor.role.bits()),
                        format!("0x{:08X}", cursor.text_signature),
                        cursor.depth.to_string(),
                    )
                } else {
                    (
                        "None".into(),
                        "None".into(),
                        format!("0 / {}", total_nodes),
                        "None".into(),
                        "None".into(),
                        "0".into(),
                    )
                };

                let px_str = ui.fmt(format_args!("• Subpixel Centroid: ({px}, {py})"));
                let stream_str = ui.fmt(format_args!("• Stream Index:      {stream_idx}"));
                let sig_str = ui.fmt(format_args!("• Text Signature:    {text_sig}"));
                let role_str = ui.fmt(format_args!("• Role Bitflags:     {role_bits}"));
                let depth_str = ui.fmt(format_args!("• Layer Depth:       {depth}"));

                ui.flow_down(
                    flow()
                        .fillw()
                        .gap(6)
                        .pad(8)
                        .bg(hex("#0f172a"))
                        .radius(4)
                        .border(border_col),
                    |ui| {
                        ui.text(px_str, text().fg(hex("#38bdf8")));
                        ui.text(stream_str, text().fg(hex("#4ade80")));
                        ui.text(sig_str, text().fg(hex("#facc15")));
                        ui.text(role_str, text().fg(hex("#f472b6")));
                        ui.text(depth_str, text().fg(text_dim));
                    },
                );

                ui.gap(6);

                ui.text("EVENT LOG", text().fg(hex("#fbbf24")).bold());
                ui.flow_down(
                    flow()
                        .fillw()
                        .gap(4)
                        .pad(8)
                        .bg(hex("#0f172a"))
                        .radius(4)
                        .border(border_col),
                    |ui| {
                        if event_logs.is_empty() {
                            ui.text("No events yet.", text().fg(text_dim));
                        } else {
                            for log in &event_logs {
                                let col = if log.contains("[ACTIVATED]") {
                                    hex("#86efac")
                                } else if log.contains("[MUTATION]") {
                                    hex("#fde047")
                                } else {
                                    hex("#cbd5e1")
                                };
                                let log_str = ui.fmt(format_args!("{log}"));
                                ui.text(log_str, text().fg(col).font_size(12));
                            }
                        }
                    },
                );

                ui.gap(6);

                ui.text("CONTROLS & SHORTCUTS", text().fg(hex("#94a3b8")).bold());
                ui.flow_down(
                    flow()
                        .fillw()
                        .gap(4)
                        .pad(8)
                        .bg(hex("#0f172a"))
                        .radius(4)
                        .border(border_col),
                    |ui| {
                        ui.text(
                            "[Tab] / [Shift+Tab]   : Sequential Traversal",
                            text().fg(text_dim).font_size(12),
                        );
                        ui.text(
                            "[Arrow Keys]          : 2D Directional Cone",
                            text().fg(text_dim).font_size(12),
                        );
                        ui.text(
                            "[Enter] / [Space]     : Activate Focused Item",
                            text().fg(text_dim).font_size(12),
                        );
                        ui.text(
                            "[Mouse Left Click]    : Click & Relocate Anchor",
                            text().fg(text_dim).font_size(12),
                        );
                        ui.text("[Esc]                 : Exit Demo", text().fg(text_dim).font_size(12));
                    },
                );
            });
        });
    }
}
