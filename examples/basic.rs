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
    let mut open: Option<Menu> = None;

    loop {
        if exit() {
            break;
        }

        begin_ui(black());

        {
            begin_layout_with_bounds(Flow::Right, Rect::new(0, 0, ctx.width(), 30));

            let menu_style = style()
                .font_size(13)
                .height(30)
                .pad(13)
                .bg(rgb(25, 25, 25))
                .hover(rgb(45, 45, 45));

            if ctx.button("File", menu_style).clicked() {
                open = Some(Menu::File);
            }

            if ctx.button("Edit", menu_style).clicked() {
                open = Some(Menu::Edit);
            }

            if ctx.button("View", menu_style).clicked() {
                open = Some(Menu::View);
            }

            if ctx.button("Playback", menu_style).clicked() {
                open = Some(Menu::Playback);
            }

            if ctx.button("Library", menu_style).clicked() {
                open = Some(Menu::Library);
            }

            if ctx.button("Help", menu_style).clicked() {
                open = Some(Menu::Help);
            }

            ctx.spacer(menu_style);

            end_layout();

            if let Some(menu) = open {
                let dropdown_width = 180;
                let font_sz = 13;
                let padding_v = 8;

                let item_style = style()
                    .font_size(font_sz)
                    .padl(12)
                    .padr(12)
                    .padt(padding_v)
                    .padb(padding_v)
                    .bg(rgb(35, 35, 35))
                    .hover(rgb(60, 60, 60));

                let drop_x = match menu {
                    Menu::File => 0,
                    Menu::Edit => 50,
                    Menu::View => 100,
                    Menu::Playback => 150,
                    Menu::Library => 235,
                    Menu::Help => 305,
                };

                begin_layout_with_bounds(Flow::Down, Rect::new(drop_x, 30, dropdown_width, 400));

                match menu {
                    Menu::File => {
                        if ctx.list_item("New Project", false, dropdown_width, item_style) {
                            println!("Clicked: New Project");
                            open = None;
                        }
                        if ctx.list_item("Open File...", false, dropdown_width, item_style) {
                            println!("Clicked: Open File");
                            open = None;
                        }
                        if ctx.list_item("Save", false, dropdown_width, item_style) {
                            println!("Clicked: Save");
                            open = None;
                        }
                    }
                    Menu::Edit => {
                        if ctx.list_item("Undo", false, dropdown_width, item_style) {
                            open = None;
                        }
                        if ctx.list_item("Redo", false, dropdown_width, item_style) {
                            open = None;
                        }
                        if ctx.list_item("Cut", false, dropdown_width, item_style) {
                            open = None;
                        }
                    }
                    Menu::View => {
                        if ctx.list_item("Toggle Sidebar", false, dropdown_width, item_style) {
                            open = None;
                        }
                        if ctx.list_item("Zoom In", false, dropdown_width, item_style) {
                            open = None;
                        }
                    }
                    Menu::Playback => {
                        if ctx.list_item("Play / Pause", false, dropdown_width, item_style) {
                            open = None;
                        }
                        if ctx.list_item("Stop", false, dropdown_width, item_style) {
                            open = None;
                        }
                    }
                    Menu::Library => {
                        if ctx.list_item("Scan Folders...", false, dropdown_width, item_style) {
                            open = None;
                        }
                    }
                    Menu::Help => {
                        if ctx.list_item("Documentation", false, dropdown_width, item_style) {
                            open = None;
                        }
                        if ctx.list_item("About", false, dropdown_width, item_style) {
                            open = None;
                        }
                    }
                }

                end_layout();
            }
        }

        draw_cmd();
        ctx.draw();
    }
}
