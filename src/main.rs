use tui::layout::Rect;
use tui::layout::Constraint::{Length, Percentage};

use regex::Regex;
use lazy_static::lazy_static;
use chrono::prelude::*;

use crossterm::{
  event::{self, Event as CEvent, KeyCode},
  terminal::{disable_raw_mode, enable_raw_mode},
};
use serde::{Deserialize, Serialize};

use std::io;
use std::sync::mpsc;
use std::thread;

use std::time::{Duration, Instant};
use thiserror::Error;
use tui::{
  backend::CrosstermBackend,
  layout::{Constraint, Direction, Layout},
  style::{Color, Modifier, Style},
  text::{Span, Spans},
  widgets::{
    Block, Borders, Cell, Row, Table, TableState, Tabs,
  },
  Terminal,
};

pub struct StatefulTable {
  state: TableState,
  commits: Vec<CommitRow>,
}

impl<'a> StatefulTable {
  fn new(commits: Vec<CommitRow>) -> StatefulTable {
    StatefulTable {
      state: TableState::default(),
      commits: commits
    }
  }

  pub fn next(&mut self) {
    let i = match self.state.selected() {
      Some(i) => {
        if i >= self.commits.len() - 1 {
          0
        } else {
          i + 1
        }
      }
      None => 0,
    };
    self.state.select(Some(i));
  }

  pub fn previous(&mut self) {
    let i = match self.state.selected() {
      Some(i) => {
        if i == 0 {
          self.commits.len() - 1
        } else {
          i - 1
        }
      }
      None => 0,
    };
    self.state.select(Some(i));
  }

  pub fn first(&mut self) {
    self.state.select(Some(0));
  }

  pub fn last(&mut self) {
    self.state.select(Some(self.commits.len() - 1));
  }
}


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
struct Author {
  name: String,
  email: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct CommitRow {
  sha: String,
  short_sha: String,
  subject: String,
  author: Author,
  contributor: Author,
  co_authors: Vec<Author>,
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

fn extract_name(input: &str) -> String {
    lazy_static! {
      static ref RE: Regex = Regex::new(r"^Co-authored-by: (.+) <").unwrap();
    }
  let cap = RE.captures(input).unwrap();

  format!("{}", &cap[1])
}

fn extract_email(input: &str) -> String {
  lazy_static! {
    static ref RE: Regex = Regex::new(r"<(.*)>").unwrap();
  }
  let cap = RE.captures(input).unwrap();

  format!("{}", &cap[1])
}

fn interrogate_git_repository() -> Vec<CommitRow> {
  let output: String = run_command::run("git", &["log", "--pretty=%H‖%h‖%s‖%an‖%ae‖%cn‖%cE‖%(trailers:key=Co-authored-by)」", "--max-count=200"], "");

  let tidied_output: String = output.replace(r"」\n$", "");
  let mut rows: Vec<&str> = tidied_output.split("」\n").collect();
  rows = rows.into_iter().filter(|&i| i != "").collect::<Vec<_>>();
  let commits: Vec<CommitRow> = rows
    .iter()
    .map(|row| {
      let field: Vec<&str> = row.split("‖").collect();

      let co_authors = field[7].split("\n")
        .filter(|&i| i != "")
        .map(|co_author| {
          Author {
            name: extract_name(co_author),
            email: extract_email(co_author),
          }
        })
        .collect();

      let commit_row = CommitRow {
        sha: format!("{}", field[0]),
        short_sha: format!("{}", field[1]),
        subject: format!("{}", field[2]),
        author: Author {
          name: format!("{}", field[3]),
          email: format!("{}", field[4]),
        },
        contributor: Author {
          name: format!("{}", field[5]),
          email: format!("{}", field[6]),
        },
        co_authors: co_authors
      };
      return commit_row;
    }).collect();
  return commits;
}

mod run_command {
  use std::process::{Command, Stdio};
  use std::str;
  use std::io::Write;
  use std::{thread, time};

  #[allow(unused)]
  pub fn run_basic(program:&str) -> String {
    let arguments: &[&str] = &[];
    let std_in_string: &str = "";
    run(program,arguments,std_in_string)
   }

  pub fn run(program:&str,arguments:&[&str],std_in_string:&str) -> String {
    let mut child = Command::new(program)
      .args(arguments)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .spawn()
      .expect("failed to execute child");

    {
      let stdin = child.stdin.as_mut().expect("Failed to get stdin");
      stdin.write_all(std_in_string.as_bytes()).expect("Failed to write to stdin");
    }

    let check_every = time::Duration::from_millis(10);
    loop {
      match child.try_wait() {
        Ok(Some(_status)) => {break;},  // finished running
        Ok(None) => {}                  // still running
        Err(e) => {panic!("error attempting to wait: {}", e)},
      }
      thread::sleep(check_every);
    }

    let output = child
      .wait_with_output()
      .expect("failed to wait on child");

    let final_output: String = match str::from_utf8(&output.stdout){
      Ok(output) => {output.to_string()},
      Err(e) => {panic!("{}", e);},
    };

    final_output
  }
}

fn draw_header() -> Row<'static> {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let commit_rows: Vec<CommitRow> = interrogate_git_repository();
  let mut table = StatefulTable::new(commit_rows.clone());

  enable_raw_mode().expect("can run in raw mode");

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

  loop {
    terminal.draw(|rect| {
      let size = rect.size();
      let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
          [
            Constraint::Min(2),
            Constraint::Length(1),
          ]
          .as_ref(),
        )
        .split(size);

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

      let tabs = Tabs::new(menu)
        .style(Style::default().fg(Color::White))
        .divider(Span::raw("|"));

      rect.render_widget(tabs, chunks[1]);

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

      let t = Table::new(rows)
        .header(draw_header())
        .block(Block::default().borders(Borders::ALL).title("Commits"))
        .highlight_style(selected_style)
        .highlight_symbol("➡️  ")
        .widths(&[Length(10), Percentage(50), Percentage(50)])
        .style(tablestyle);

        rect.render_stateful_widget(t, chunks[0], &mut table.state);
    })?;

    match rx.recv()? {
      Event::Input(event) => match event.code {
        KeyCode::Char('q') => {
          disable_raw_mode()?;
          terminal.clear()?;
          terminal.show_cursor()?;
          break;
        }

        KeyCode::Esc => {
          disable_raw_mode()?;
          terminal.clear()?;
          terminal.show_cursor()?;
          break;
        }

        KeyCode::Down => {
          table.next();
        }

        KeyCode::Up => {
          table.previous();
        }

        KeyCode::PageUp => {
          table.first();
        }

        KeyCode::PageDown => {
          table.last();
        }
        _ => {}
      },

      Event::Tick => {}
    }
  }

  Ok(())
}
