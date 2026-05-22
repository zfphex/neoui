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
    let ctx = create_ctx("Basic", 1000, 700);
    let mut current_menu: Option<(Menu, Rect)> = None;

    loop {
        if exit() {
            break;
        }

        let ctx_width = ctx.width();
        let ctx_height = ctx.height();

        begin_ui(black());

        {
            let height = 30;
            let menu_style = style()
                .font_size(13)
                .height(height)
                .padl(14)
                .padr(14)
                .bg(rgb(25, 25, 25))
                .hover(rgb(45, 45, 45));

            begin_layout_with_bounds(Flow::Right, Rect::new(0, 0, ctx_width, height));

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
                    current_menu = Some((menu, state.rect));
                }
            }

            ctx.button("", style().width(1).height(height).bg(hex("#424242")));

            ctx.spacer(menu_style);

            end_layout();
        }

        if let Some((menu, rect)) = current_menu {
            let dropdown_width = 180;
            let item_style = style()
                .font_size(13)
                .padlr(12)
                .padtb(8)
                .bg(rgb(35, 35, 35))
                .hover(rgb(60, 60, 60));

            begin_layout_with_bounds(Flow::Down, Rect::new(rect.x, 30, dropdown_width, 400));

            let menu_items: &[&str] = match menu {
                Menu::File => &["New Project", "Open File...", "Save"],
                Menu::Edit => &["Undo", "Redo", "Cut"],
                Menu::View => &["Toggle Sidebar", "Zoom In"],
                Menu::Playback => &["Play / Pause", "Stop"],
                Menu::Library => &["Scan Folders..."],
                Menu::Help => &["Documentation", "About"],
            };

            for &item in menu_items {
                if ctx
                    .list_item(item, false, dropdown_width, item_style)
                    .clicked
                {
                    println!("{}", item);
                    current_menu = None;
                }
            }

            end_layout();

            if ctx.clicked(Rect::new(0, 0, ctx_width, ctx_height)) {
                current_menu = None;
            }
        }

        draw_cmd();
        ctx.draw();
    }
}
