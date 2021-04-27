#[path = "header.rs"]
mod header;
use header::draw_header;

#[path = "structs.rs"]
mod structs;
use super::structs::{ CommitRow };

use tui::{
  layout::{Constraint::{Length, Percentage}},
  style::{Color, Style},
  widgets::{Borders, Block, Row, Table},
};

pub fn draw_commit_list(commit_rows: Vec<CommitRow>) -> Table<'static> {
  let tablestyle = Style::default().
    bg(Color::Black).
    fg(Color::Gray);

  let selected_style = Style::default()
    .bg(Color::Rgb(40, 40, 40))
    .fg(Color::Gray);

  let rows = commit_rows.iter()
    .map(|commit| {
      let commit = commit.clone();
      let co_authors: Vec<String> = commit.co_authors.iter().map(|co_author| { co_author.name.clone() }).collect();

    let author_list: String = if co_authors.len() > 0 {
      format!("{}, {}", commit.author.name, co_authors.join(", "))
    } else {
      format!("{}", commit.author.name)
    };

    Row::new(vec![
      commit.short_sha,
      commit.subject,
      author_list.to_string(),
    ])
  });

  Table::new(rows)
    .header(draw_header())
    .block(Block::default().borders(Borders::ALL).title("Commits"))
    .highlight_style(selected_style)
    .highlight_symbol("➡️  ")
    .widths(&[Length(10), Percentage(50), Percentage(50)])
    .style(tablestyle)
}
