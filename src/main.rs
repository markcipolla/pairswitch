use clap::{arg, App, AppSettings};

#[path = "initialize_git.rs"]
mod initialize_git;
use initialize_git::{ initialize_git };

#[path = "commit.rs"]
mod commit;
use commit::{ commit };

#[path = "pairing_session_start.rs"]
mod pairing_session_start;
use pairing_session_start::{ start_pairing_session };

#[path = "pairing_session_end.rs"]
mod pairing_session_end;
use pairing_session_end::{ end_pairing_session };

fn main() {
    let mut siv = cursive::default();

    let arguments = App::new("Pairswitch")
      .setting(AppSettings::SubcommandRequiredElseHelp)
      .setting(AppSettings::AllowExternalSubcommands)
      .setting(AppSettings::AllowInvalidUtf8ForExternalSubcommands)
      .version("0.0.1")
      .author("Mark Cipolla <mark@markcipolla.com>")
      .about("Teaches argument parsing")
      .subcommand(
        App::new("commit")
          .setting(AppSettings::Hidden)
          .about("Internally used to respond to Git hooks")
          .arg(
            arg!(<PATH> "The commit msg file")
            .takes_value(true)
            .allow_invalid_utf8(true)
          )
          .arg(
            arg!(<SOURCE> "The commit source")
            .takes_value(true)
            .allow_invalid_utf8(true)
          )
          .arg(
            arg!(<SHA> "The commit SHA")
            .takes_value(true)
            .allow_invalid_utf8(true)
          )
      )
      .subcommand(
        App::new("init")
          .display_order(1)
          .about("Sets up Pairswitch with the current Git repository's hooks")
      )
      .subcommand(
        App::new("start")
        .display_order(2)
        .about("Starts a pairing session")
      )
      .subcommand(
        App::new("end")
          .display_order(3)
          .about("Ends a pairing session")
      )
      .get_matches();

    match arguments.subcommand() {
      Some(("commit", sub_matches)) => {
        commit(sub_matches);
      }
      Some(("init", _sub_matches)) => {
        initialize_git(&mut siv);
        siv.run();
      }
      Some(("start", _sub_matches)) => {
        start_pairing_session(&mut siv);
        siv.run();
      }
      Some(("end", _sub_matches)) => {
        end_pairing_session(&mut siv);
        siv.run();
      }
      _ => {},
    }
}
