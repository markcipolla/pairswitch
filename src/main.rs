#[path = "initialize_git.rs"]
mod initialize_git;
use initialize_git::{ initialize_git };

#[path = "pre_commit.rs"]
mod pre_commit;
use pre_commit::{ pre_commit };

use clap::{Arg, App};

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
      initialize_git(&mut siv);
      siv.run();
    } else if stage == "pre-commit" {
      pre_commit(&mut siv);
      siv.run();
    } else if stage == "prepare-commit-msg" {
      println!("The stage is: {}", stage);
    } else {

    }
}
