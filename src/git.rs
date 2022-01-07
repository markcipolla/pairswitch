
use regex::Regex;
use lazy_static::lazy_static;
use itertools::Itertools;

#[path = "run_command.rs"]
mod run_command;

#[path = "structs.rs"]
mod structs;
use super::structs::{ Contributor, Commit };

fn extract_name(input: &str) -> String {
  lazy_static! {
    static ref RE: Regex = Regex::new(r"^Co-authored-by: (.+) <").unwrap();
  }
  let cap = RE.captures(input).unwrap();

  format!("{}", &cap[1])
}

fn extract_email(input: &str) -> String {
  lazy_static! {
    static ref RE: Regex = Regex::new(r"<(.*)>").unwrap();
  }
  let cap = RE.captures(input).unwrap();

  format!("{}", &cap[1])
}

pub fn interrogate_git_repository() -> Vec<Commit> {
  let output: String = run_command::run_command::run("git", &["log", "--pretty=%H‖%h‖%s‖%an‖%ae‖%cn‖%cE‖%(trailers:key=Co-authored-by)」", "--max-count=300"], "");

  let tidied_output: String = output.replace(r"」\n$", "");
  let mut rows: Vec<&str> = tidied_output.split("」\n").collect();
  rows = rows.into_iter().filter(|&i| i != "").collect::<Vec<_>>();
  let commits: Vec<Commit> = rows
    .iter()
    .map(|row| {
      let field: Vec<&str> = row.split("‖").collect();

      let co_authors = field[7].split("\n")
        .filter(|&i| i != "")
        .map(|co_author| {
          Contributor {
            name: extract_name(co_author),
            email: extract_email(co_author),
          }
        })
        .collect();

      let commit_row = Commit {
        sha: format!("{}", field[0]),
        short_sha: format!("{}", field[1]),
        subject: format!("{}", field[2]),
        author: Contributor {
          name: format!("{}", field[3]),
          email: format!("{}", field[4]),
        },
        contributor: Contributor {
          name: format!("{}", field[5]),
          email: format!("{}", field[6]),
        },
        co_authors: co_authors
      };
      return commit_row;
    }).collect();
  return commits;
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

pub fn authors_and_contributors_names(commit: &Commit) -> String {
  let contributors = authors_and_contributors(commit);

  return contributors.iter().map(|contributor| contributor.clone().name).join(", ").to_string();
}

pub fn contributors(commits: Vec<Commit>) -> Vec<Contributor> {
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
