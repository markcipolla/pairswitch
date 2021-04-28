#[path = "structs.rs"]
mod structs;
use super::structs::{ CommitRow };

use tui::{
  style::{Color, Style},
  widgets::{Block, Borders},
};

pub fn draw_manage_commit(selected_commit_index: u32, commit_rows: Vec<CommitRow>) -> Block<'static> {
  let commit = &commit_rows[selected_commit_index as usize];
  let block = Block::default()
    .borders(Borders::ALL)
    .style(Style::default().bg(Color::Rgb(60,60,60)))
    .title(format!("{}", commit.subject));

  return block;
}
