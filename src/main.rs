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
    Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,TableState
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
  enable_raw_mode().expect("can run in raw mode");

  let mut echo_hello = Command::new("git");
  echo_hello.arg("log");
  echo_hello.arg("--pretty='%H|%h|%s|%an|&aN|%ae|&aE|%aD|%aR|%cn|%cE'");

  println!("{:?}", echo_hello.output());

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
  let mut active_menu_item = MenuItem::Home;
  let mut commit_list_state = ListState::default();
  commit_list_state.select(Some(0));

  let repo = match Repository::discover("./") {
    Ok(repo) => repo,
    Err(e) => panic!("failed to open: {}", e),
  };

    // let repo_root = std::env::args().nth(1).unwrap_or(".".to_string());
    // let repo = Repository::open(repo_root.as_str()).expect("Couldn't open repository");


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

                    let (list, right) = render_commit_list(&commit_list_state);
                    rect.render_stateful_widget(list, pets_chunks[0], &mut commit_list_state);
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
            if let Some(selected) = commit_list_state.selected() {
              let amount_pets = read_db().expect("can fetch pet list").len();
              if selected >= amount_pets - 1 {
                commit_list_state.select(Some(0));
              } else {
                commit_list_state.select(Some(selected + 1));
              }
            }
          }

          KeyCode::Up => {
            if let Some(selected) = commit_list_state.selected() {
              let amount_pets = read_db().expect("can fetch pet list").len();
              if selected > 0 {
                commit_list_state.select(Some(selected - 1));
              } else {
                commit_list_state.select(Some(amount_pets - 1));
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


fn render_commit_list<'a>(commit_list_state: &ListState) -> (List<'a>, Table<'a>) {
  let commit_list = Block::default()
  .borders(Borders::TOP | Borders::BOTTOM | Borders::LEFT)
  .style(Style::default().fg(Color::White))
  .title("Commits")
  .border_type(BorderType::Plain);


  let pet_list = read_db().expect("can fetch pet list");
  let items: Vec<_> = pet_list
  .iter()
  .map(|pet| {
    ListItem::new(
      Spans::from(
        vec![
        Span::styled(pet.name.clone(), Style::default())
        ]
        )
      )
  })
  .collect();

  let commits = pet_list;
      // .get(commit_list_state.selected().expect("there is always a selected pet"))
      // .expect("exists")
      // .clone();

  let list = List::new(items)
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
      Cell::from(Span::raw(commit.name.clone())),
      Cell::from(Span::raw(commit.category.clone())),
      Cell::from(Span::raw(commit.age.clone().to_string())),
      Cell::from(Span::raw(commit.created_at.clone().to_string())),
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


