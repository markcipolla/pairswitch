use chrono::DateTime;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

impl Author {}
#[derive(Serialize, Deserialize, Clone)]
pub struct Author {
  pub name: String,
  pub email: String,
}

impl Commit {}
#[derive(Serialize, Deserialize, Clone)]
pub struct Commit {
  pub id: usize,
  pub name: String,
  pub category: String,
  pub age: usize,
  pub created_at: DateTime<Utc>,
}

impl CommitRow {}
#[derive(Serialize, Deserialize, Clone)]
pub struct CommitRow {
  pub sha: String,
  pub short_sha: String,
  pub subject: String,
  pub author: Author,
  pub contributor: Author,
  pub co_authors: Vec<Author>,
}
