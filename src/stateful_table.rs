use tui::{
  widgets::{
    TableState,
  },
};

// mod structs;
use crate::structs::{CommitRow};

pub struct StatefulTable {
  pub state: TableState,
  pub commits: Vec<CommitRow>,
}

impl<'a> StatefulTable {
  pub fn new(commits: Vec<CommitRow>) -> StatefulTable {
    StatefulTable {
      state: TableState::default(),
      commits: commits
    }
  }

  pub fn next(&mut self) {
    let i = match self.state.selected() {
      Some(i) => {
        if i >= self.commits.len() - 1 {
          0
        } else {
          i + 1
        }
      }
      None => 0,
    };
    self.state.select(Some(i));
  }

  pub fn previous(&mut self) {
    let i = match self.state.selected() {
      Some(i) => {
        if i == 0 {
          self.commits.len() - 1
        } else {
          i - 1
        }
      }
      None => 0,
    };
    self.state.select(Some(i));
  }

  pub fn first(&mut self) {
    self.state.select(Some(0));
  }

  pub fn last(&mut self) {
    self.state.select(Some(self.commits.len() - 1));
  }
}
