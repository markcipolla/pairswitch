// Local modules
// #[path = "git.rs"]
// mod git;
// use git::interrogate_git_repository;

// #[path = "table.rs"]
// mod table;
// use table::table_view;

// // #[path = "theme.rs"]
// // mod theme;
// // use theme::theme;

// #[path = "structs.rs"]
// mod structs;
// use structs::{ Contributor, Commit };

#[path = "initialize_git.rs"]
mod initialize_git;
use initialize_git::{ initialize_git };

use clap::{Arg, App};
// use itertools::Itertools;

fn main() {
    let mut siv = cursive::default();

    let arguments = App::new("Pairswitch")
      .version("0.0.1")
      .author("Mark Cipolla <mark@markcipolla.com>")
      .about("Teaches argument parsing")
      .arg(Arg::new("stage")
        .short('s')
        .long("stage")
        .takes_value(true)
        .help("Internally used to respond to Git hooks")
      )
      .arg(Arg::new("init")
        .short('i')
        .long("init")
        .takes_value(false)
        .help("Sets up Git hooks to...")
      )
      .get_matches();

    let is_init = arguments.is_present("init");
    let stage = arguments.value_of("stage").unwrap_or("default");

    if is_init {
      initialize_git(&mut siv)
    } else if stage == "pre-commit" {
      println!("The stage is: {}", stage);
    } else {

    }

    // let commits: Vec<Commit> = interrogate_git_repository();

    // let collaborator_names: Vec<String> = contributors(commits.clone()).iter()
    //   .map(|collaborator| {
    //     collaborator.clone().name.to_string()
    //   })
    //   .collect();

    siv.run();
}
