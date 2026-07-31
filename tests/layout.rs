use neoui::*;

fn main() {
    let mut ui = ui("layout", 800, 600);

    ui.frame(|ui| {
        // Flow direction, padding and gap.
        ui.place_right(bounds(Rect::new(0, 0, 200, 100)).pad(10).gap(5), |ui| {
            assert_eq!(ui.rect(style().wh(20)).bounds, Rect::new(10, 10, 20, 20));
            assert_eq!(ui.rect(style().wh(20)).bounds, Rect::new(35, 10, 20, 20));
            ui.gap(5);
            assert_eq!(ui.rect(style().wh(20)).bounds, Rect::new(65, 10, 20, 20));
        });

        ui.place_down(bounds(Rect::new(0, 0, 100, 200)).padtb(10).gap(4), |ui| {
            assert_eq!(ui.rect(style().wh(20)).bounds, Rect::new(0, 10, 20, 20));
            assert_eq!(ui.rect(style().wh(20)).bounds, Rect::new(0, 34, 20, 20));
        });

        // Reverse flows anchor to the far edge and walk backwards.
        ui.place_left(bounds(Rect::new(0, 0, 200, 100)), |ui| {
            assert_eq!(ui.rect(style().wh(20)).bounds, Rect::new(180, 0, 20, 20));
            assert_eq!(ui.rect(style().wh(20)).bounds, Rect::new(160, 0, 20, 20));
        });

        ui.place_up(bounds(Rect::new(0, 0, 100, 200)), |ui| {
            assert_eq!(ui.rect(style().wh(20)).bounds, Rect::new(0, 180, 20, 20));
            assert_eq!(ui.rect(style().wh(20)).bounds, Rect::new(0, 160, 20, 20));
        });

        // Sizes.
        ui.place_right(bounds(Rect::new(0, 0, 200, 50)), |ui| {
            assert_eq!(ui.rect(style().w(30).h(10)).bounds, Rect::new(0, 0, 30, 10));
            let percentage = ui.rect(style().w(Size::Percentage(0.25)).h(10));
            assert_eq!(percentage.bounds, Rect::new(30, 0, 50, 10));
            let fill_minus = ui.rect(style().w(Size::FillMinus(40)).h(10));
            assert_eq!(fill_minus.bounds, Rect::new(80, 0, 80, 10));
            assert_eq!(ui.rect(style().fill_width().h(10)).bounds, Rect::new(160, 0, 40, 10));
        });

        ui.place_down(bounds(Rect::new(0, 0, 50, 200)), |ui| {
            assert_eq!(ui.rect(style().w(10).h(60)).bounds, Rect::new(0, 0, 10, 60));
            assert_eq!(ui.rect(style().w(10).fill_height()).bounds, Rect::new(0, 60, 10, 140));
        });

        // Cross axis alignment of the flow.
        ui.place_down(bounds(Rect::new(0, 0, 200, 100)).align_flow(AlignFlow::Center), |ui| {
            assert_eq!(ui.rect(style().wh(20)).bounds, Rect::new(90, 0, 20, 20))
        });

        ui.place_down(bounds(Rect::new(0, 0, 200, 100)).align_flow(AlignFlow::End), |ui| {
            assert_eq!(ui.rect(style().wh(20)).bounds, Rect::new(180, 0, 20, 20))
        });

        ui.place_right(bounds(Rect::new(0, 0, 200, 100)).align_flow(AlignFlow::Center), |ui| {
            assert_eq!(ui.rect(style().wh(20)).bounds, Rect::new(0, 40, 20, 20))
        });

        // Explicit coordinates are absolute and do not move the cursor.
        ui.place_down(bounds(Rect::new(0, 0, 200, 200)), |ui| {
            assert_eq!(ui.rect(style().x(50).y(60).wh(10)).bounds, Rect::new(50, 60, 10, 10));
            assert_eq!(ui.rect(style().wh(20)).bounds, Rect::new(0, 10, 20, 20));
        });

        // A nested flow shrinks to fit its children, then advances the parent by that size.
        ui.place_down(bounds(Rect::new(0, 0, 200, 200)), |ui| {
            let row = ui.flow_right(style().pad(4).gap(2), |ui| {
                ui.rect(style().wh(10));
                ui.rect(style().wh(10));
            });
            assert_eq!(row.bounds, Rect::new(0, 0, 30, 18));
            assert_eq!(ui.rect(style().wh(20)).bounds, Rect::new(0, 18, 20, 20));
        });

        // `place_*` lays out its children but never advances the parent.
        ui.place_down(bounds(Rect::new(0, 0, 200, 200)), |ui| {
            ui.place_right(style().wh(30), |ui| {
                assert_eq!(ui.rect(style().wh(10)).bounds, Rect::new(0, 0, 10, 10));
            });
            assert_eq!(ui.rect(style().wh(20)).bounds, Rect::new(0, 0, 20, 20));
        });

        // Margin grows the painted and interactable box without reserving space.
        ui.place_right(bounds(Rect::new(20, 20, 200, 100)), |ui| {
            assert_eq!(ui.rect(style().wh(20).mar(5)).bounds, Rect::new(15, 15, 30, 30));
            assert_eq!(ui.rect(style().wh(20)).bounds, Rect::new(40, 20, 20, 20));
        });

        // Fills stop at the padding, unless they bleed through it.
        ui.place_right(bounds(Rect::new(0, 0, 200, 100)).pad(20), |ui| {
            assert_eq!(ui.rect(style().fill_width().h(10)).bounds, Rect::new(20, 20, 160, 10));
        });

        ui.place_right(bounds(Rect::new(0, 0, 200, 100)).pad(20), |ui| {
            let bled = ui.rect(style().fill_width().bleed().h(10));
            assert_eq!(bled.bounds, Rect::new(20, 20, 180, 10));
        });

        // Splitting the remaining space of the current frame.
        ui.place_down(bounds(Rect::new(0, 0, 200, 100)), |ui| {
            let (left, right) = ui.split_h(80);
            assert_eq!(left, Rect::new(0, 0, 80, 100));
            assert_eq!(right, Rect::new(80, 0, 120, 100));

            let (top, bottom) = ui.split_v(-40);
            assert_eq!(top, Rect::new(0, 0, 200, 60));
            assert_eq!(bottom, Rect::new(0, 60, 200, 40));
        });

        // Scrolling reports how far the content overflows the viewport.
        let mut scroll_y = 0;
        let scroll = ui.scroll(bounds(Rect::new(0, 0, 100, 100)), &mut scroll_y, |ui| {
            for _ in 0..10 {
                ui.rect(style().w(10).h(30));
            }
        });
        assert_eq!(scroll.content_height, 300);
        assert_eq!(scroll.max_scroll, 200);

        // Items scrolled out of the viewport are culled, the rest are clipped to it.
        let mut scroll_y = 60;
        ui.scroll(bounds(Rect::new(0, 0, 100, 100)), &mut scroll_y, |ui| {
            assert!(ui.rect(style().w(10).h(30)).bounds.is_empty());
            assert!(ui.rect(style().w(10).h(30)).bounds.is_empty());
            assert_eq!(ui.rect(style().w(10).h(30)).bounds, Rect::new(0, 0, 10, 30));
            assert_eq!(ui.rect(style().w(10).h(30)).bounds, Rect::new(0, 30, 10, 30));
        });

        ui.window.close();
    });
}
