use regex::Regex;

#[path = "run_command.rs"]
mod run_command;

#[path = "structs.rs"]
mod structs;
use super::structs::{ Contributor, Commit };

pub fn interrogate_git_repository() -> Vec<Contributor> {
  let output: String = run_command::run_command::run("git", &["shortlog", "-se", "--all"], "");

  let mut rows: Vec<&str> = output.split("\n").collect();
  rows = rows.into_iter().filter(|&i| i != "").collect::<Vec<_>>();

  let contributors: Vec<Contributor> = rows.iter()
    .map(|row| {
      let regex = Regex::new(r"\w*\d+	(.*) <(.*)>").unwrap();
      let values = regex.captures(row).unwrap();
      let contributor = Contributor {
        name: values[1].to_string(),
        email: values[2].to_string(),
      };
      return contributor;
    })
    .collect();

  return contributors;
}
