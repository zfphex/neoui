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

fn main() {
    let ctx = create_ctx("Basic", 1000, 700, WindowStyle::DEFAULT);
    let mut current_menu: Option<(Menu, usize)> = None;

    loop {
        if exit() {
            break;
        }

        begin_ui(black());

        {
            let menu_style = style()
                .font_size(13)
                .height(30)
                .padl(14)
                .padr(14)
                .bg(rgb(25, 25, 25))
                .hover(rgb(45, 45, 45));

            begin_layout(Flow::Right);

            let items = [
                ("File", Menu::File),
                ("Edit", Menu::Edit),
                ("View", Menu::View),
                ("Playback", Menu::Playback),
                ("Library", Menu::Library),
                ("Help", Menu::Help),
            ];

            for (label, variant) in items {
                let state = ctx.button(label, menu_style);
                if state.clicked {
                    current_menu = Some((variant, state.rect.x));
                }
            }

            ctx.spacer(menu_style);

            end_layout();
        }

        if let Some((menu, absolute_x)) = current_menu {
            let dropdown_width = 180;
            let item_style = style()
                .font_size(13)
                .padlr(12)
                .padtb(8)
                .bg(rgb(35, 35, 35))
                .hover(rgb(60, 60, 60));

            begin_layout_with_bounds(Flow::Down, Rect::new(absolute_x, 30, dropdown_width, 400));

            let menu_items: &[&str] = match menu {
                Menu::File => &["New Project", "Open File...", "Save"],
                Menu::Edit => &["Undo", "Redo", "Cut"],
                Menu::View => &["Toggle Sidebar", "Zoom In"],
                Menu::Playback => &["Play / Pause", "Stop"],
                Menu::Library => &["Scan Folders..."],
                Menu::Help => &["Documentation", "About"],
            };

            for &item in menu_items {
                if ctx.list_item(item, false, dropdown_width, item_style) {
                    println!("{}", item);
                    current_menu = None;
                }
            }

            end_layout();
        }

        draw_cmd();
        ctx.draw();
    }
}
