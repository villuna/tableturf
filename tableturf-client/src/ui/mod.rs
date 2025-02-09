mod button;
mod textbox;

pub mod colours {
    use raylib::color::Color;

    pub const VIOLET: Color = Color::new(135, 60, 190, 255);
    pub const PURPLE: Color = Color::new(200, 122, 255, 255);
    pub const DARKGRAY: Color = Color::new(80, 80, 80, 255);
}

pub use button::Button;
pub use textbox::TextBox;
