use tui::{
  style::{Color, Modifier, Style},
  text::{Span, Spans},
  widgets::{Tabs},
};

pub fn draw_menu() -> Tabs<'static> {
  let menu_titles = vec!["Change author", "Add pair", "Quit"];
  let menu = menu_titles
    .iter()
    .map(|t| {
      let (first, rest) = t.split_at(1);
      Spans::from(vec![
        Span::styled(
          first,
          Style::default()
          .fg(Color::Yellow)
          .add_modifier(Modifier::UNDERLINED),
          ),
        Span::styled(rest, Style::default().fg(Color::White)),
        ])
    })
    .collect();

  Tabs::new(menu)
    .style(Style::default().fg(Color::White))
    .divider(Span::raw("|"))
}
