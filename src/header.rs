use tui::{
  style::{Color, Modifier, Style},
  widgets::{Row, Cell},
};

pub fn draw_header() -> Row<'static> {
  let header_style = Style::default().
    add_modifier(Modifier::BOLD).
    add_modifier(Modifier::SLOW_BLINK).
    add_modifier(Modifier::UNDERLINED).
    fg(Color::Gray).
    bg(Color::Blue);

  let header_cells = ["SHA", "Subject", "Author and co-authors"]
    .iter()
    .map(|h| Cell::from(*h).style(Style::default().fg(Color::Gray)));

  Row::new(header_cells)
    .style(header_style)
    .height(1)
}
