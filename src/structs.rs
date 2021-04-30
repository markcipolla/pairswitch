

use serde::{Deserialize, Serialize};

impl Contributor {}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Contributor {
  pub name: String,
  pub email: String,
}

impl Commit {}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Commit {
  pub sha: String,
  pub short_sha: String,
  pub subject: String,
  pub author: Contributor,
  pub contributor: Contributor,
  pub co_authors: Vec<Contributor>,
}
