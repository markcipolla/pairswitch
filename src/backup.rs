use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use cursive::view::{Offset, Position};
use cursive::traits::*;
use cursive::{
  views::{Button, Dialog, FixedLayout, TextView},
  Cursive,
  Rect,
};

extern crate fui;
extern crate serde_json;

use crossterm::{
  event::{self, Event as CEvent, KeyCode},
  terminal::{disable_raw_mode, enable_raw_mode},
};

use thiserror::Error;
use tui::{
  backend::CrosstermBackend,
  layout::{Constraint, Direction, Layout},
  Terminal,
};

mod git;
use git::interrogate_git_repository;

mod dialog;
use dialog::draw_dialog;

mod commit_list;
use commit_list::draw_commit_list;
mod manage_commit;
use manage_commit::draw_manage_commit;
mod manage_commit_menu;
use manage_commit_menu::draw_manage_commit_menu;
mod menu;
use menu::draw_menu;

mod stateful_table;
use stateful_table::StatefulTable;
mod structs;
use structs::{ Commit };

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let commit_rows: Vec<Commit> = interrogate_git_repository();
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
        rect.render_stateful_widget(draw_commit_list(commit_rows.clone()), commit_list[0], &mut table.state);

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

        // draw_dialog();
        rect.render_widget(draw_manage_commit_menu(), modal[1]);
        rect.render_widget(draw_manage_commit(selected_commit_index, commit_rows.clone()), modal[0]);
      };
    })?;

    if interacting_with_selected_commit == true {
      match rx.recv()? {
        Event::Input(event) => match event.code {
          KeyCode::Esc => {
            interacting_with_selected_commit = false;
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
        }, Event::Tick => {}
      }
    }

  }

  Ok(())
}
