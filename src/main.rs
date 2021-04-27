use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

extern crate cursive;
extern crate fui;
extern crate serde_json;

use cursive::traits::Resizable;
use cursive::views::Dialog;
use cursive::Cursive;
use serde_json::value::Value;

use fui::fields::Autocomplete;
use fui::form::FormView;
use fui::validators::{OneOf, Required};

use regex::Regex;
use lazy_static::lazy_static;

use crossterm::{
  event::{self, Event as CEvent, KeyCode},
  terminal::{disable_raw_mode, enable_raw_mode},
};

use thiserror::Error;
use tui::{
  backend::CrosstermBackend,
  layout::{Constraint, Constraint::{Length, Percentage}, Direction, Layout},
  style::{Color, Modifier, Style},
  text::{Span, Spans},
  widgets::{
    Block, Borders, Cell, Row, Table, Tabs,
  },
  Terminal,
};

mod run_command;

mod stateful_table;
use stateful_table::StatefulTable;
mod structs;
use structs::{ Author, CommitRow };

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
  let output: String = run_command::run_command::run("git", &["log", "--pretty=%H‖%h‖%s‖%an‖%ae‖%cn‖%cE‖%(trailers:key=Co-authored-by)」", "--max-count=200"], "");

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

fn draw_menu() -> Tabs<'static> {
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

fn draw_table(commit_rows: Vec<CommitRow>) -> Table<'static> {
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

fn draw_manage_commit_menu() -> Tabs<'static> {
  let menu_titles = vec!["Escape to go back"];
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

fn draw_manage_commit(selected_commit_index: u32, commit_rows: Vec<CommitRow>) -> Block<'static> {
  let commit = &commit_rows[selected_commit_index as usize];
  let block = Block::default()
    .borders(Borders::ALL)
    .style(Style::default().bg(Color::Rgb(60,60,60)))
    .title(format!("{}", commit.subject));

  return block;
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

  let mut selected_commit_index = 0;
  let mut interacting_with_selected_commit = false;



  loop {
    terminal.draw(|rect| {
      let size = rect.size();

      let commit_list = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
          [
            Constraint::Min(2),
            Constraint::Length(1),
          ]
          .as_ref(),
        )
        .split(size);
        rect.render_widget(draw_menu(), commit_list[1]);
        rect.render_stateful_widget(draw_table(commit_rows.clone()), commit_list[0], &mut table.state);

      if interacting_with_selected_commit == true {
        let modal_margin = if size.width > 80 { 5 } else { 0 };
        let modal = Layout::default()
        .direction(Direction::Vertical)
        .margin(modal_margin)
        .constraints(
          [
            Constraint::Min(2),
            Constraint::Length(1),
          ]
          .as_ref(),
        )
        .split(size);
        rect.render_widget(draw_manage_commit_menu(), modal[1]);
        rect.render_widget(draw_manage_commit(selected_commit_index, commit_rows.clone()), modal[0]);
      }
    })?;

    if interacting_with_selected_commit == true {
       match rx.recv()? {
        Event::Input(event) => match event.code {
          KeyCode::Esc => {
            interacting_with_selected_commit = false
          }
          _ => {}
        },
        Event::Tick => {}
      }
    } else  {
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

          KeyCode::Char('a') => {
            interacting_with_selected_commit = true;
          }

          KeyCode::Down => {
            table.next();
            selected_commit_index = table.state.selected().unwrap() as u32;
          }
          KeyCode::Up => {
            table.previous();
            selected_commit_index = table.state.selected().unwrap() as u32;
          }
          KeyCode::PageUp => {
            table.first();
            selected_commit_index = table.state.selected().unwrap() as u32;
          }
          KeyCode::PageDown => {
            table.last();
            selected_commit_index = table.state.selected().unwrap() as u32;
          }
          _ => {}
        },

        Event::Tick => {}
      }
    }
  }

  Ok(())
}
