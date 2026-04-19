pub struct Button<'a> {
    pub label: &'a str,
    pub bg_color: Option<u32>,
}

impl<'a> Button<'a> {
    pub fn bg(mut self, color: u32) -> Self {
        self.bg_color = Some(color);
        self
    }

    pub fn clicked(self) -> bool {
        // button(ctx, window, label)
        todo!()
    }

    pub fn show(self) {
        self.clicked();
    }
}
