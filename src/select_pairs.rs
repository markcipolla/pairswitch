// Crate Dependencies ---------------------------------------------------------
extern crate cursive;
extern crate cursive_table_view;
extern crate rand;

// STD Dependencies -----------------------------------------------------------
use std::cmp::Ordering;

use cursive_table_view::TableViewItem;

mod structs;
use structs::{ Contributor, Commit };

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum BasicColumn {
  Name,
  Email,
}

use cursive::{
  Cursive,
  align::HAlign
};

#[path = "git.rs"]
mod git;
use git::{ interrogate_git_repository };

use cursive_table_view::{TableView};

impl TableViewItem<BasicColumn> for Contributor {
  fn to_column(&self, column: BasicColumn) -> String {
    match column {
      BasicColumn::Name => self.name.to_string(),
      BasicColumn::Email => self.email.to_string(),
    }
  }

  fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering
    where
      Self: Sized,
    {
      match column {
        BasicColumn::Name => self.name.cmp(&other.name),
        BasicColumn::Email => self.email.cmp(&other.email),
      }
    }
}

pub fn table_view() -> TableView::<Contributor, BasicColumn> {
  let contributors: Vec<Contributor> = interrogate_git_repository();

  let mut table_view = TableView::<Contributor, BasicColumn>::new()
    .column(BasicColumn::Name, "Name", |c| c.align(HAlign::Right))
    .column(BasicColumn::Email, "Email", |c| c);

  table_view.set_items(contributors.clone());
  table_view.set_on_submit(move |_siv: &mut Cursive, _row: usize, index: usize| {

    let contributor: &Contributor = &contributors.clone()[index];
    println!("{:?} {:?}", contributor.name, contributor.email)
    //Set pair
  });

  return table_view;
}
