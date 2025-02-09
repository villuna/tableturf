//class TextBox {
//    std::string contents;
//    Rectangle bounds;
//    int char_limit;
//    int font_size;

use raylib::{color::Color, ffi::KeyboardKey, math::Rectangle, prelude::{RaylibDraw, RaylibDrawHandle}, RaylibHandle};

use super::colours::DARKGRAY;

const TEXTBOX_HPADDING: i32 = 5;
const TEXTBOX_VPADDING: i32 = 3;

pub struct TextBox {
    contents: String,
    bounds: Rectangle,
    char_limit: usize,
    font_size: i32,
}

impl TextBox {
    /// Creates a new textbox.
    ///
    /// Will be displayed at the given x and y coordinates. `char_limit` dictates the maximum number
    /// of characters that can be entered into the textbox.
    pub fn new(rl: &RaylibHandle, x: i32, y: i32, char_limit: usize, font_size: i32) -> Self {
        let mut contents = String::with_capacity(char_limit + 1);
        for _ in 0..char_limit {
            contents.push('G');
        }

        let width = rl.measure_text(&contents, font_size);

        let bounds = Rectangle::new(
            x as _,
            y as _,
            (width + 2 * TEXTBOX_HPADDING) as f32,
            (font_size + 2 * TEXTBOX_VPADDING) as f32,
        );

        contents.clear();

        Self {
            contents,
            bounds,
            char_limit,
            font_size,
        }
    }

    /// Responds to user input and updates the contents of the textbox accordingly. This should be
    /// called every frame while the textbox is active.
    pub fn update(&mut self, rl: &mut RaylibHandle) {
        while let Some(c) = rl.get_char_pressed() {
            if c.is_ascii() {
                if self.contents.len() == self.char_limit {
                    break;
                }

                self.contents.push(c);
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
            self.contents.pop();
        }
    }

    /// Draws the textbox to the screen.
    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        // Draw the box
        d.draw_rectangle_rec(self.bounds, Color::LIGHTGRAY);

        // Draw the text
        let draw_underscore = self.contents.len() < self.char_limit;
        let mut to_draw = self.contents.clone();

        if draw_underscore {
            to_draw.push('_');
        }

        d.draw_text(
            &to_draw,
            self.bounds.x as i32 + TEXTBOX_HPADDING,
            self.bounds.y as i32 + TEXTBOX_VPADDING,
            self.font_size,
            Color::BLUE,
        );

        // Draw how many characters are in the textbox and the limit
        d.draw_text(
            &format!("{}/{}", self.contents.len(), self.char_limit),
            self.bounds.x as i32,
            (self.bounds.y + self.bounds.height) as i32 + TEXTBOX_VPADDING,
            20,
            DARKGRAY,
        );
    }

    /// Returns the string contained in the textbox, replacing it with an empty string.
    pub fn take(&mut self) -> String {
        std::mem::take(&mut self.contents)
    }

    pub fn bounds(&self) -> Rectangle {
        self.bounds
    }
}
