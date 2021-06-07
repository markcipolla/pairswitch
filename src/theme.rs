extern crate cursive;
use cursive::{
  Cursive,
  theme::{
    Color,
    PaletteColor,
    Theme,
  },
};

pub fn theme(siv: &Cursive) -> Theme {
  // We'll return the current theme with a small modification.
  let mut theme = siv.current_theme().clone();
  theme.palette[PaletteColor::Background] = Color::Rgb(44, 62, 8);
  theme.palette[PaletteColor::View] = Color::Rgb(236, 240, 241);
  theme.palette[PaletteColor::Primary] = Color::Rgb(44, 62, 8);
  theme.palette[PaletteColor::Secondary] = Color::Rgb(52, 152, 219);
  theme.palette[PaletteColor::Tertiary] = Color::Rgb(41, 128, 185);
  theme.palette[PaletteColor::TitlePrimary] = Color::Rgb(180,180,180);
  // theme.palette[PaletteColor::TitleSecondary] = Color::Rgb(200,200,200);
  // theme.palette[PaletteColor::Highlight] = Color::Rgb(231, 76, 60);
  // theme.palette[PaletteColor::HighlightInactive] = Color::Rgb(255,255,255);
  // theme.shadow = false;
  theme
}
