use bracket_lib::prelude::*;
use std::sync::{Arc, Mutex};

pub struct UiRect {
    rect: Rect,
    ui: Ui,
}

impl DrawUi for UiRect {
    fn draw(&self, ctx: &mut DrawContext) {
        ctx.bterm.draw_box(
            self.rect.position.x,
            self.rect.position.y,
            self.rect.width,
            self.rect.height,
            GREEN,
            DARK_GREEN,
        );

        self.ui.draw(ctx);
    }
}

pub struct UiPrint {
    rect: Rect,
    text: String,
}

impl DrawUi for UiPrint {
    fn draw(&self, ctx: &mut DrawContext) {
        ctx.bterm.print_color(
            self.rect.position.x,
            self.rect.position.y,
            GREEN,
            BLACK,
            &self.text,
        );
    }
}

pub trait DrawUi {
    fn draw(&self, ctx: &mut DrawContext);
}

#[derive(Clone)]
pub struct Rect {
    pub position: Point,
    pub width: i32,
    pub height: i32,
}

pub struct DrawContext<'a> {
    pub bterm: &'a mut BTerm,
}

pub struct Ui {
    pub mouse_point: Point,
    pub mouse_click: bool,
    pub offset: Point,
    pub rect: Rect,
    pub drawables: Vec<Box<dyn DrawUi>>,
}

impl Ui {
    pub fn new(ctx: &BTerm, rect: Rect) -> Self {
        Self {
            mouse_point: ctx.mouse_point(),
            mouse_click: INPUT.lock().is_mouse_button_pressed(0) && ctx.left_click,
            offset: Point::new(0, 0),
            rect,
            drawables: Vec::new(),
        }
    }

    pub fn sub(&self, width: i32, height: i32, offset: Point) -> Ui {
        Ui {
            mouse_point: self.mouse_point,
            mouse_click: self.mouse_click,
            rect: Rect {
                position: self.offset.clone(),
                width,
                height,
            },
            offset,
            drawables: Vec::new(),
        }
    }

    pub fn get_rect(&self) -> Rect {
        Rect {
            position: self.rect.position + self.offset,
            width: self.rect.width,
            height: self.rect.height,
        }
    }

    pub fn offset(&mut self, offset: Point) {
        self.offset = self.offset + offset;
    }

    pub fn rect(&mut self, width: i32, height: i32, mut f: impl FnMut(&mut Ui)) {
        let mut ui = self.sub(width, height, Point::new(1, 1));

        f(&mut ui);

        self.drawables.push(Box::new(UiRect {
            rect: ui.rect.clone(),
            ui,
        }));

        self.offset(Point::new(0, height));
    }

    pub fn print(&mut self, text: impl Into<String>) {
        self.drawables.push(Box::new(UiPrint {
            rect: self.get_rect(),
            text: text.into(),
        }));
        self.offset(Point::new(0, 1));
    }

    pub fn text(&mut self, text: impl Into<String>, mut f: impl FnMut(&Ui)) {
        let text = text.into();
        let ui = self.sub(text.len() as i32, 1, Point::new(0, 0));
        f(&ui);

        self.drawables.push(Box::new(UiPrint {
            rect: self.get_rect(),
            text: text,
        }));
        self.offset(Point::new(0, 1));
    }

    pub fn clicked(&self) -> bool {
        self.mouse_click
            && self.mouse_point.x >= self.rect.position.x
            && self.mouse_point.x <= self.rect.position.x + self.rect.width
            && self.mouse_point.y >= self.rect.position.y
            && self.mouse_point.y <= self.rect.position.y + self.rect.height
    }
}

impl DrawUi for Ui {
    fn draw(&self, ctx: &mut DrawContext) {
        for drawable in &self.drawables {
            drawable.draw(ctx);
        }
    }
}
