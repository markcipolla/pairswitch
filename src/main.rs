// Crate Dependencies ---------------------------------------------------------
extern crate cursive;
extern crate cursive_table_view;
extern crate rand;

// STD Dependencies -----------------------------------------------------------
use std::cmp::Ordering;

// Local modules
mod git;
use git::interrogate_git_repository;
mod structs;
use structs::{ Contributor, Commit };
use itertools::Itertools;
use serde_json::value::Value;
use fui::fields::{Autocomplete, Multiselect};
use fui::form::FormView;


use fui::validators::{Required};

// External Dependencies ------------------------------------------------------
use cursive_table_view::{TableViewItem, TableView};
use cursive::{
    Cursive,
    align::HAlign,
    traits::*,
    theme::{
      Color,
      PaletteColor,
      Theme,
    },
    views::{
      ResizedView,
      Dialog,
    }
};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum BasicColumn {
  ShortSha,
  Subject,
  Authors,
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

fn theme(siv: &Cursive) -> Theme {
  // We'll return the current theme with a small modification.
  let mut theme = siv.current_theme().clone();
  // theme.palette[PaletteColor::Background] = Color::Rgb(44, 62, 8);
  theme.palette[PaletteColor::View] = Color::Rgb(236, 240, 241);
  theme.palette[PaletteColor::Primary] = Color::Rgb(44, 62, 8);
  // theme.palette[PaletteColor::Secondary] = Color::Rgb(52, 152, 219);
  // theme.palette[PaletteColor::Tertiary] = Color::Rgb(41, 128, 185);
  // theme.palette[PaletteColor::TitlePrimary] = Color::Rgb(180,180,180);
  // theme.palette[PaletteColor::TitleSecondary] = Color::Rgb(200,200,200);
  // theme.palette[PaletteColor::Highlight] = Color::Rgb(231, 76, 60);
  // theme.palette[PaletteColor::HighlightInactive] = Color::Rgb(255,255,255);
  // theme.shadow = false;
  theme
}

fn cancel_form(s: &mut Cursive) {
  s.pop_layer();
}

fn submit_form(c: &mut Cursive, data: Value) {
  c.pop_layer();
  let text = format!("Got data: {:?}", data);
  c.add_layer(Dialog::info(text));
}

fn main() {
  let mut siv = cursive::default();

  let theme = theme(&siv);
  siv.set_theme(theme);

  // We can quit by pressing `q`
  siv.add_global_callback('q', Cursive::quit);

  let mut table = TableView::<Commit, BasicColumn>::new()
    .column(BasicColumn::ShortSha, "SHA", |c| c.align(HAlign::Right).width(10))
    .column(BasicColumn::Subject, "Subject", |c| c)
    .column(BasicColumn::Authors, "Authors", |c| {
      c.ordering(Ordering::Greater)
    });

  let commits: Vec<Commit> = interrogate_git_repository();

  let collaborator_names: Vec<String> = contributors(commits.clone()).iter()
    .map(|collaborator| {
      collaborator.clone().name.to_string()
    })
    .collect();

  table.set_items(commits.clone());

  table.set_on_submit(move |siv: &mut Cursive, _row: usize, index: usize| {
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
    siv.add_layer(Dialog::around(form));//.full_screen());
  });

  siv.add_layer(
    ResizedView::with_full_screen(table.with_name("table"))
  );

  siv.set_fps(32);
  siv.run();
}

// fn add_name(s: &mut Cursive) {
//     fn ok(s: &mut Cursive, name: &str) {
//         s.call_on_name("select", |view: &mut SelectView<String>| {
//             view.add_item_str(name)
//         });
//         s.pop_layer();
//     }

//     s.add_layer(Dialog::around(EditView::new()
//             .on_submit(ok)
//             .with_name("name")
//             .fixed_width(10))
//         .title("Enter a new name")
//         .button("Ok", |s| {
//             let name =
//                 s.call_on_name("name", |view: &mut EditView| {
//                     view.get_content()
//                 }).unwrap();
//             ok(s, &name);
//         })
//         .button("Cancel", |s| {
//             s.pop_layer();
//         }));
// }

// fn delete_name(s: &mut Cursive) {
//     let mut select = s.find_name::<SelectView<String>>("select").unwrap();
//     match select.selected_id() {
//         None => s.add_layer(Dialog::info("No name to remove")),
//         Some(focus) => {
//             select.remove_item(focus);
//         }
//     }
// }

// fn on_submit(s: &mut Cursive, name: &str) {
//     s.pop_layer();
//     s.add_layer(Dialog::text(format!("Name: {}\nAwesome: yes", name))
//         .title(format!("{}'s info", name))
//         .button("Quit", Cursive::quit));
// }
