use ratatui::style::Color;

pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub selection: Color,
    pub highlight: Color,
    pub comment: Color,
    pub cyan: Color,
    pub purple: Color,
}

impl Theme {
    pub const fn dracula() -> Self {
        Self {
            background: Color::Rgb(40, 42, 54),
            foreground: Color::Rgb(248, 248, 242),
            selection: Color::Rgb(68, 71, 90),
            highlight: Color::Rgb(98, 114, 164),
            comment: Color::Rgb(98, 114, 164),
            cyan: Color::Rgb(139, 233, 253),
            purple: Color::Rgb(189, 147, 249),
        }
    }
}
