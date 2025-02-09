use raylib::prelude::*;
use crate::ui::colours::*;

const BUTTON_VPADDING: i32 = 7;
const BUTTON_HPADDING: i32 = 15;


pub struct Button {
    pub x: i32,
    pub y: i32,
    pub font_size: i32,
    pub width: i32,
    pub height: i32,
    pub label: String,
}

impl Button {
    pub fn new(rl: &RaylibHandle, x: i32, y: i32, label: impl Into<String>, font_size: i32) -> Self {
        let label = label.into();
        Self {
            x,
            y,
            font_size,
            width: rl.measure_text(&label, font_size) + 2 * BUTTON_HPADDING,
            height: font_size + 2 * BUTTON_VPADDING,
            label,
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        let color = if !self.is_hovered(d) {
            VIOLET
        } else {
            PURPLE
        };

        d.draw_rectangle(self.x, self.y, self.width, self.height, color);
        d.draw_text(&self.label, self.x + BUTTON_HPADDING, self.y + BUTTON_VPADDING, self.font_size, Color::WHITE);
    }

    pub fn is_clicked(&self, rl: &RaylibHandle) -> bool {
        self.is_hovered(rl) && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
    }

    pub fn is_hovered(&self, rl: &RaylibHandle) -> bool {
        let pos = rl.get_mouse_position();
        let (x, y, width, height) = (self.x as f32, self.y as f32, self.width as f32, self.height as f32);
        pos.x >= x && pos.x <= x + width && pos.y >= y && pos.y <= y + height
    }
}
