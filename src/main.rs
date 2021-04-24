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
    Block, BorderType, Borders, Cell, List, ListItem, ListState, Row, Table, Tabs,
  },
  Terminal,
};

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
  description: String,
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
  println!("{:?}", cap);
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
  let output: String = simple_run_command::run("git", &["log", "--pretty=%H‖%h‖%s‖%an‖%ae‖%cn‖%cE‖%(trailers:key=Co-authored-by)」"], "");
  //let test: String = simple_run_command::run("/bin/sh", &["-c", r#"echo test "something in quotes" "#], "");
  let tidied_output: String = output.replace(r"」\n$", "");
  let mut rows: Vec<&str> = tidied_output.split("」\n").collect();
  rows = rows.into_iter().filter(|&i| i != "").collect::<Vec<_>>();
  let commits: Vec<CommitRow> = rows
    .iter()
    .map(|row| {
      let field: Vec<&str> = row.split("‖").collect();


      let cos: Vec<String> = field[7].split("\n")
        .filter(|&i| i != "")
        .map(|co_author| {
          extract_name(co_author)
        })
        .collect();

      println!("{:?}", cos);

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
        description: format!("{}", field[2]),
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


mod simple_run_command {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
  enable_raw_mode().expect("can run in raw mode");

  let commits: Vec<CommitRow> = interrogate_git_repository();

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
  // terminal.clear()?;

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
        .divider(Span::raw("|"));

      rect.render_widget(tabs, chunks[0]);

      match active_menu_item {
        MenuItem::Home => {
          let pets_chunks = Layout::default()
          .direction(Direction::Horizontal)
          .constraints(
            [Constraint::Length(9), Constraint::Min(10)].as_ref(),
            )
          .split(chunks[1]);

          let (list, right) = render_commit_list(commits.clone());
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

            if selected >= commits.len() - 1 {
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
              highlighted_commit.select(Some(commits.len() - 1));
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


fn render_commit_list<'a>(commits: Vec<CommitRow>) -> (List<'a>, Table<'a>) {
  let sha_list = Block::default()
    .borders(Borders::TOP | Borders::BOTTOM | Borders::LEFT)
    .style(Style::default().fg(Color::White))
    .title("Commits")
    .border_type(BorderType::Plain);

  let items: Vec<_> = commits
    .iter()
    .map(|commit| {
      ListItem::new(
        Spans::from(
          vec![
            Span::styled(commit.short_sha.clone(), Style::default())
          ]
        )
      )
    })
    .collect();

  let list = List::new(items)
  .block(sha_list)
  .highlight_style(
    Style::default()
    .bg(Color::Yellow)
    .fg(Color::Black)
    .add_modifier(Modifier::BOLD),
  );

  let rows = commits.iter()
    .map(|commit| {
      let commit = commit.clone();
      let co_authors: Vec<String> = commit.co_authors.iter().map(|co_author| { co_author.name.clone() }).collect();
      Row::new(vec![
        Cell::from(Span::raw(String::from(commit.description))),
        Cell::from(Span::raw(String::from(commit.author.name))),
        Cell::from(Span::raw(String::from(co_authors.join(", ")))),
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
      Constraint::Min(20),
      Constraint::Min(20),

    ]);

  (list, commit_list)
}
