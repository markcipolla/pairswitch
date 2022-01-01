// // Crate Dependencies ---------------------------------------------------------
// extern crate cursive;
// extern crate cursive_table_view;
// extern crate rand;

// Local modules
#[path = "git.rs"]
mod git;
use git::interrogate_git_repository;

// #[path = "table.rs"]
// mod table;
// use table::table_view;

// // #[path = "theme.rs"]
// // mod theme;
// // use theme::theme;

#[path = "structs.rs"]
mod structs;
use structs::{ Contributor, Commit };

// STD Dependencies -----------------------------------------------------------
use std::env;
use std::cmp::Ordering;
use itertools::Itertools;
use regex::Regex;
use clap::Parser;

use clap::{Arg, App};

// // External Dependencies ------------------------------------------------------

// use cursive::{
//     Cursive,
//     traits::*,
//     views::{
//       ResizedView,
//       LinearLayout,
//       TextView,
//       Button
//     }
// };

// fn draw_start_pairing_session(screen: String) -> LinearLayout {
//   LinearLayout::vertical()
//     .child(TextView::new("Start a pairing session?"))
//     .child(
//       LinearLayout::horizontal()
//         .child(Button::new("No", |s| s.quit()))
//         .child(Button::new("Yes", move || screen = "pick_commits".to_string()))
//     )
// }


// fn main() {
//   let mut siv = cursive::default();
//   let mut screen: String = "start_pairing_session".to_string();
//   let commits: Vec<Commit> = interrogate_git_repository();
//   // let theme = theme(&siv);
//   // siv.set_theme(theme);

//   siv.add_global_callback('q', Cursive::quit);
//   if screen == "start_pairing_session".to_string() {
//     siv.add_layer(LinearLayout::vertical().child(draw_start_pairing_session(screen)))
//   } else if screen == "pick_commits".to_string() {
//     siv.add_layer(LinearLayout::vertical().child(ResizedView::with_full_screen(table_view(commits).with_name("table"))))
//   } else {
//     siv.add_layer(LinearLayout::vertical().child(draw_start_pairing_session(screen)))
//   };


//   siv.set_fps(32);
//   siv.run();
// }


use cursive::Cursive;
use cursive::views::{
  Button,
  Dialog,
  DummyView,
  EditView,
  LinearLayout,
  SelectView
};
use cursive::traits::*;

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

fn main() {
    let mut siv = cursive::default();

    let matches = App::new("My Test Program")
      .version("0.1.0")
      .author("Hackerman Jones <hckrmnjones@hack.gov>")
      .about("Teaches argument parsing")
      .arg(Arg::with_name("stage")
        .short('s')
        .long("stage")
        .takes_value(true)
        .help("Internally used to respond to Git hooks")
      )
      .get_matches();

    let stage = matches.value_of("stage").unwrap_or("default");
    println!("The stage is: {}", stage);

    let commits: Vec<Commit> = interrogate_git_repository();

    let collaborator_names: Vec<String> = contributors(commits.clone()).iter()
      .map(|collaborator| {
        collaborator.clone().name.to_string()
      })
      .collect();

    let select = SelectView::<String>::new()
        .on_submit(on_submit)
        .with_name("select");

    let buttons = LinearLayout::vertical()
      .child(DummyView)
      .child(Button::new("Yes", add_name))
      .child(Button::new("No", delete_name))
      .child(DummyView)
      .child(Button::new("Cancel", Cursive::quit))
      .child(DummyView);


    siv.add_layer(Dialog::around(LinearLayout::horizontal()
      .child(select)
      .child(DummyView)
      .child(buttons))
      .title("Are you pairing?"));

    siv.run();
}

fn add_name(s: &mut Cursive) {
    fn ok(s: &mut Cursive, name: &str) {
        s.call_on_name("select", |view: &mut SelectView<String>| {
            view.add_item_str(name)
        });
        s.pop_layer();
    }

    s.add_layer(Dialog::around(EditView::new()
            .on_submit(ok)
            .with_name("name")
            .fixed_width(10))
        .title("Enter a new name")
        .button("Ok", |s| {
            let name =
                s.call_on_name("name", |view: &mut EditView| {
                    view.get_content()
                }).unwrap();
            ok(s, &name);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }));
}

fn delete_name(s: &mut Cursive) {
    let mut select = s.find_name::<SelectView<String>>("select").unwrap();
    match select.selected_id() {
        None => s.add_layer(Dialog::info("No name to remove")),
        Some(focus) => {
            select.remove_item(focus);
        }
    }
}

fn on_submit(s: &mut Cursive, name: &str) {
    s.pop_layer();
    s.add_layer(Dialog::text(format!("Name: {}\nAwesome: yes", name))
        .title(format!("{}'s info", name))
        .button("Quit", Cursive::quit));
}
