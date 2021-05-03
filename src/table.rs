// Crate Dependencies ---------------------------------------------------------
extern crate cursive;
extern crate cursive_table_view;
extern crate rand;

// STD Dependencies -----------------------------------------------------------
use std::cmp::Ordering;
use itertools::Itertools;

use serde_json::value::Value;
use cursive_table_view::TableViewItem;

#[path = "run_command.rs"]
mod run_command;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum BasicColumn {
  ShortSha,
  Subject,
  Authors,
}

#[path = "structs.rs"]
mod structs;
use super::structs::{ Contributor, Commit };

use fui::{
  fields::{Autocomplete, Multiselect},
  form::FormView,
  validators::Required,
};

use cursive::{
  Cursive,
  align::HAlign,
  views::{
    Dialog
  }
};

use cursive_table_view::{TableView};

fn submit_form(c: &mut Cursive, data: Value) {
  c.pop_layer();
  let text = format!("Got data: {:?}", data);
  c.add_layer(Dialog::info(text));
}


impl TableViewItem<BasicColumn> for Commit {
  fn to_column(&self, column: BasicColumn) -> String {
    match column {
      BasicColumn::ShortSha => self.short_sha.to_string(),
      BasicColumn::Subject => self.subject.to_string(),
      BasicColumn::Authors => authors_and_contributors_names(self).to_string(),
    }
  }

  fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering
    where
      Self: Sized,
    {
      match column {
        BasicColumn::ShortSha => self.short_sha.cmp(&other.short_sha),
        BasicColumn::Subject => self.subject.cmp(&other.subject),
        BasicColumn::Authors => authors_and_contributors_names(self).cmp(&authors_and_contributors_names(other)),
      }
    }
}

pub fn table_view(commits: Vec<Commit>) -> TableView::<Commit, BasicColumn> {
  let mut table_view = TableView::<Commit, BasicColumn>::new()
    .column(BasicColumn::ShortSha, "SHA", |c| c.align(HAlign::Right).width(10))
    .column(BasicColumn::Subject, "Subject", |c| c)
    .column(BasicColumn::Authors, "Authors", |c| c);

  let collaborator_names: Vec<String> = contributors(commits.clone()).iter()
    .map(|collaborator| {
      collaborator.clone().name.to_string()
    })
    .collect();

  table_view.set_items(commits.clone());
  table_view.set_on_submit(move |siv: &mut Cursive, _row: usize, index: usize| {
    let commit: &Commit = &commits.clone()[index];

    let author_name = commit.clone().author.name;
    let form = FormView::new()
      .field(
        Autocomplete::new("Author", collaborator_names.clone())
          .initial(author_name)
          .validator(Required)
      )
      .field(
        Multiselect::new("Co-author (s)", collaborator_names.clone())
          .initial(co_author_names(&commit))
      )
      .on_submit(submit_form)
      .on_cancel(cancel_form);
    siv.add_layer(Dialog::around(form));
  });

  return table_view;
}

fn contributors(commits: Vec<Commit>) -> Vec<Contributor> {
  let people: Vec<Contributor> = commits.iter()
    .map(|commit| {
      let mut list = [commit.author.clone()].to_vec();
      if commit.co_authors.len() > 0 {
        commit.co_authors.iter().map(|c| list.push(c.clone()));
      }
      return list;
    })
    .flatten()
    .unique_by(|c| format!("{}-{}", c.name.clone(), c.email.clone()))
    .collect();

    return people;
}

fn cancel_form(s: &mut Cursive) {
  s.pop_layer();
}

fn co_author_names(commit: &Commit) -> Vec<String> {
  commit.co_authors.iter().map(|co_author| { co_author.name.clone() }).collect()
}

fn authors_and_contributors(commit: &Commit) -> Vec<Contributor> {
  let mut list = [commit.author.clone()].to_vec();
  if commit.co_authors.len() > 0 {
    commit.co_authors.iter().map(|c| list.push(c.clone()));
  }
  return list;
}

fn authors_and_contributors_names(commit: &Commit) -> String {
  let contributors = authors_and_contributors(commit);

  return contributors.iter().map(|contributor| contributor.clone().name).join(", ").to_string();
}
