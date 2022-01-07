use std::{
  path::Path,
  fs::File,
  io::Write
};

use cursive::Cursive;
use cursive::views::{
  Dialog,
};

use fui::{
  fields::{Autocomplete, Multiselect},
  form::FormView,
  validators::Required,
  Value
};

#[path = "run_command.rs"]
mod run_command;

#[path = "git.rs"]
mod git;
use git::{
  interrogate_git_repository,
  contributors,
  // co_author_names,
};

#[path = "structs.rs"]
mod structs;
use structs::{ Contributor, Commit };

pub fn start_pairing_session(siv: &mut Cursive) {
  siv.add_layer(Dialog::text("Start a pairing session?")
    .title("Pairswitch")
    .button("Yes", select_pair)
    .button("No", |s| s.quit())
  );
}

fn submit_form(siv: &mut Cursive, data: Value) {
  let text: String = format!("{:?}", data["Who are you pairing with?"].as_str());
  let git_top_level_path: String = run_command::run_command::run("git", &["rev-parse", "--show-toplevel"], "").replace("\n", "");
  let path = Path::new(&git_top_level_path);

  let pairswitch_config = path.join(".git").join("pairswitch.conf");
  let _display = pairswitch_config.display();

  let content = "Be prepared to appreciate what you meet.";
  let pairswitch_config_path = format!("{0}/.git/pairswitch.conf", git_top_level_path);
  let path = Path::new(&pairswitch_config_path);

  if path.exists() {
    panic!("File already exists");
  }

  let mut file = match File::create(&path) {
    Ok(file) => file,
    Err(e) => panic!("Error creating file. {}", e),
  };

  match file.write_all(text.as_bytes()) {
    Ok(_) => println!("File created."),
    Err(e) => panic!("Error writing to file. {}", e),
  }

  println!("{:?}", _display.to_string());

  // let text = format!("Got data: {:?}", data["Who are you pairing with?"]);
}

fn cancel_form(siv: &mut Cursive) {
  siv.quit()
}

fn select_pair(siv: &mut Cursive) {
  siv.pop_layer();

  let commits: Vec<Commit> = interrogate_git_repository();

  let collaborator_names: Vec<String> = contributors(commits.clone()).iter()
    .map(|collaborator| {
      collaborator.clone().name.to_string()
    })
    .collect();

  let form = FormView::new()
    .field(
      Autocomplete::new("Who are you pairing with?", collaborator_names.clone())
        .validator(Required)
    )
    .on_submit(submit_form)
    .on_cancel(cancel_form);

  siv.add_layer(Dialog::around(form)
    .title("Pairswitch"))

}
