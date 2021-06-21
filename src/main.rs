// Crate Dependencies ---------------------------------------------------------
extern crate cursive;
extern crate cursive_table_view;
extern crate rand;

// Local modules
#[path = "git.rs"]
mod git;
use git::interrogate_git_repository;

#[path = "table.rs"]
mod table;
use table::table_view;

#[path = "theme.rs"]
mod theme;
use theme::theme;

#[path = "structs.rs"]
mod structs;
use structs::{ Commit };

// External Dependencies ------------------------------------------------------

use cursive::{
    Cursive,
    traits::*,
    views::{
      ResizedView,
      LinearLayout,
      TextView,
      Button
    }
};

fn draw_start_pairing_session() -> LinearLayout {
  LinearLayout::vertical()
    .child(TextView::new("Start a pairing session?"))
    .child(Button::new("Ok", |s| s.quit()))
}

fn draw_main_interface(commits: Vec<Commit>) -> LinearLayout {
  LinearLayout::vertical()
    .child(draw_start_pairing_session())
    .child(ResizedView::with_full_screen(table_view(commits).with_name("table")))
}

fn main() {
  let mut siv = cursive::default();
  let commits: Vec<Commit> = interrogate_git_repository();
  let theme = theme(&siv);
  siv.set_theme(theme);

  siv.add_global_callback('q', Cursive::quit);

  siv.add_layer(draw_main_interface(commits));

  siv.set_fps(32);
  siv.run();
}
