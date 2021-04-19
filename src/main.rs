use chrono::prelude::*;
use crossterm::{
  event::{self, Event as CEvent, KeyCode},
  terminal::{disable_raw_mode, enable_raw_mode},
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::process::Command;
use std::time::{Duration, Instant};
use thiserror::Error;
use tui::{
  backend::CrosstermBackend,
  layout::{Constraint, Direction, Layout},
  style::{Color, Modifier, Style},
  text::{Span, Spans},
  widgets::{
    Block, BorderType, Borders, Cell, List, ListItem, ListState, Row, Table, Tabs,
  },
  Terminal,
};

use git2::Repository;

const DB_PATH: &str = "./data/db.json";

#[derive(Error, Debug)]
pub enum Error {
  #[error("error reading the DB file: {0}")]
  ReadDBError(#[from] io::Error),
  #[error("error parsing the DB file: {0}")]
  ParseDBError(#[from] serde_json::Error),
}

enum Event<I> {
  Input(I),
  Tick,
}

#[derive(Serialize, Deserialize, Clone)]
struct Commit {
  id: usize,
  name: String,
  category: String,
  age: usize,
  created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone)]
struct CommitRow {
  sha: String,
  author: String,
  author_email: String
}

#[derive(Copy, Clone, Debug)]
enum MenuItem {
  Home,
}

impl From<MenuItem> for usize {
  fn from(input: MenuItem) -> usize {
    match input {
      MenuItem::Home => 0,
    }
  }
}

fn interrogate_git_repository() -> (Vec<CommitRow>, Vec<String>) {
  let mut git_log = Command::new("git");
  git_log.arg("log");
  git_log.arg("--pretty=%H‖%h‖%s‖%an‖%ae‖%cn‖%cE」");

  // let thing = Command::new("git").args(&["rev-parse", "HEAD"]).output().unwrap();
  // let git_hash = String::from_utf8(thing.stdout).unwrap();
  // println!("cargo:rustc-env=GIT_HASH={}", git_hash);

  let output: String = format!("{:?}", git_log.output());
  // println!("{:?}", output);

  let rows: Vec<&str> = output.split("」\n").collect();
  let commits: Vec<CommitRow> = rows
    .iter()
    .map(|row| {
      let row_without_line_endings: String = row.replace(r#"\\n\"#, "");
      let field: Vec<&str> = row_without_line_endings.split("‖").collect();

      println!("{:?}", field);
      return CommitRow {
        sha: format!("{}", field[0]),
        author: format!("{}", field[1]),
        author_email: format!("{}", field[2]),
      };
    }).collect();

  let shas: Vec<String> = commits
    .iter()
    .map(|commit| {
      println!("{}", commit.sha);
      return commit.sha.clone();
    })
    .collect();

  return (commits, shas);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  enable_raw_mode().expect("can run in raw mode");

  let (commits, shas) = interrogate_git_repository();

  let (tx, rx) = mpsc::channel();
  let tick_rate = Duration::from_millis(200);

  thread::spawn(move || {
    let mut last_tick = Instant::now();
    loop {
      let timeout = tick_rate
      .checked_sub(last_tick.elapsed())
      .unwrap_or_else(|| Duration::from_secs(0));

      if event::poll(timeout).expect("poll works") {
        if let CEvent::Key(key) = event::read().expect("can read events") {
          tx.send(Event::Input(key)).expect("can send events");
        }
      }

      if last_tick.elapsed() >= tick_rate {
        if let Ok(_) = tx.send(Event::Tick) {
          last_tick = Instant::now();
        }
      }
    }
  });

  let stdout = io::stdout();
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;
  terminal.clear()?;

  let menu_titles = vec!["Home", "Change ownership", "Quit"];
  let active_menu_item = MenuItem::Home;
  let mut highlighted_commit = ListState::default();
  highlighted_commit.select(Some(0));

  loop {
    terminal.draw(|rect| {
      let size = rect.size();
      let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
        [
        Constraint::Length(3),
        Constraint::Min(2)
        ]
        .as_ref(),
        )
      .split(size);

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

      let tabs = Tabs::new(menu)
      .select(active_menu_item.into())
      .block(Block::default().title("Menu").borders(Borders::ALL))
      .style(Style::default().fg(Color::White))
        // .highlight_style(Style::default().fg(Color::Yellow))
        .divider(Span::raw("|"));

        rect.render_widget(tabs, chunks[0]);

        match active_menu_item {
          MenuItem::Home => {
            let pets_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
              [Constraint::Length(15), Constraint::Min(10)].as_ref(),
              )
            .split(chunks[1]);

            let (list, right) = render_commit_list(commits, shas);
            rect.render_stateful_widget(list, pets_chunks[0], &mut highlighted_commit);
            rect.render_widget(right, pets_chunks[1]);
          }
        }
      })?;

    match rx.recv()? {
      Event::Input(event) => match event.code {
        KeyCode::Char('q') => {
          disable_raw_mode()?;
          terminal.show_cursor()?;
          break;
        }

        KeyCode::Esc => {
          disable_raw_mode()?;
          terminal.show_cursor()?;
          break;
        }

        KeyCode::Down => {
          if let Some(selected) = highlighted_commit.selected() {

            if selected >= shas.len() - 1 {
              highlighted_commit.select(Some(0));
            } else {
              highlighted_commit.select(Some(selected + 1));
            }
          }
        }

        KeyCode::Up => {
          if let Some(selected) = highlighted_commit.selected() {

            if selected > 0 {
              highlighted_commit.select(Some(selected - 1));
            } else {
              highlighted_commit.select(Some(shas.len() - 1));
            }
          }
        }
        _ => {}
      },

      Event::Tick => {}
    }
  }

  Ok(())
}


fn render_commit_list<'a>(commits: Vec<CommitRow>, shas: Vec<String>) -> (List<'a>, Table<'a>) {
  let commit_list = Block::default()
  .borders(Borders::TOP | Borders::BOTTOM | Borders::LEFT)
  .style(Style::default().fg(Color::White))
  .title("Commits")
  .border_type(BorderType::Plain);


  // let commits = pet_list;
  //     // .get(highlighted_commit.selected().expect("there is always a selected pet"))
  //     // .expect("exists")
  //     // .clone();

  let list = List::new(shas)
  .block(commit_list)
  .highlight_style(
    Style::default()
    .bg(Color::Yellow)
    .fg(Color::Black)
    .add_modifier(Modifier::BOLD),
  );

  let rows = commits.iter()
  .map(|commit| {
    Row::new(vec![
      Cell::from(Span::raw(commit.sha.clone())),
      Cell::from(Span::raw(commit.author.clone())),
      Cell::from(Span::raw(commit.author_email.clone().to_string())),
      Cell::from(Span::raw(commit.author_email.clone().to_string())),
    ])
  });

  let commit_list = Table::new(rows)
  .highlight_style(
    Style::default()
    .bg(Color::Yellow)
    .fg(Color::Black)
    .add_modifier(Modifier::BOLD),
  )
  .block(
    Block::default()
    .borders(Borders::TOP | Borders::BOTTOM | Borders::RIGHT)
    .style(Style::default().fg(Color::White))
    .border_type(BorderType::Plain),
  )
  .widths(&[
    Constraint::Length(20),
    Constraint::Length(20),
    Constraint::Length(20),
    Constraint::Length(20),
  ]);

  (list, commit_list)
}

fn read_db() -> Result<Vec<Commit>, Error> {
  let db_content = fs::read_to_string(DB_PATH)?;
  let parsed: Vec<Commit> = serde_json::from_str(&db_content)?;
  Ok(parsed)
}


