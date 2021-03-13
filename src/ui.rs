use bracket_lib::prelude::*;

pub struct Ui<'a> {
    pub position: Point,
    pub offset: Point,
    pub width: i32,
    pub height: i32,
    pub ctx: &'a mut BTerm,
}

impl<'a> Ui<'a> {
    pub fn new(ctx: &'a mut BTerm, width: i32, height: i32) -> Self {
        Self {
            position: Point::new(0, 0),
            offset: Point::new(0, 0),
            width,
            height,
            ctx,
        }
    }

    pub fn add_offset(&mut self, point: Point) {
        self.offset = self.offset + point;
    }

    pub fn rect(
        &'a mut self,
        width: i32,
        height: i32,
        mut f: impl FnMut(&mut Self),
    ) {
        self.ctx.draw_box(
            self.offset.x,
            self.offset.y,
            width,
            height,
            GREEN,
            DARK_GREEN,
        );

        let mut ui = Self {
            position: self.offset,
            offset: Point::new(1, 1),
            width,
            height,
            ctx: &mut *self.ctx,
        };

        f(&mut ui);
    }

    pub fn text(
        &'a mut self,
        text: impl Into<String>,
        mut f: impl FnMut(&mut Self),
    ) {
        let text = text.into();
        self.ctx.print_color(
            self.offset.x,
            self.offset.y,
            GREEN,
            DARK_GREEN,
            &text,
        );

        self.offset.y += 1;

        let mut ui = Self {
            position: self.offset,
            offset: Point::new(0, 0),
            width: text.len() as i32,
            height: 1,
            ctx: &mut *self.ctx,
        };

        f(&mut ui);
    }

    pub fn print(&mut self, text: impl Into<String>) {
        let text = text.into();
        self.ctx.print_color(
            self.position.x + self.offset.x,
            self.position.y + self.offset.y,
            GREEN,
            DARK_GREEN,
            &text,
        );
    }

    pub fn clicked(&self) -> bool {
        INPUT.lock().is_mouse_button_pressed(0)
            && self.ctx.mouse_point().x >= self.position.x
            && self.ctx.mouse_point().x <= self.position.x + self.width
            && self.ctx.mouse_point().y >= self.position.y
            && self.ctx.mouse_point().y <= self.position.y + self.height
    }
}
