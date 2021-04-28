// Crate Dependencies ---------------------------------------------------------
extern crate cursive;
extern crate cursive_table_view;
extern crate rand;

// STD Dependencies -----------------------------------------------------------
use std::cmp::Ordering;

// Local modules
mod git;
use git::interrogate_git_repository;
mod structs;
use structs::{ Commit };

// External Dependencies ------------------------------------------------------
use cursive_table_view::{TableViewItem, TableView};
use cursive::{
    Cursive,
    align::HAlign,
    traits::*,
    theme::{
      Color,
      PaletteColor,
      Theme,
    },
    views::{
      BoxedView,
      TextView,
      ResizedView,
      Dialog,
    }
};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum BasicColumn {
    ShortSha,
    Subject,
    Authors,
}

impl BasicColumn {
  fn as_str(&self) -> &str {
    match *self {
      BasicColumn::ShortSha => "SHA",
      BasicColumn::Subject => "Subject",
      BasicColumn::Authors => "Authors",
    }
  }
}

fn authors_and_contributors(commit: &Commit) -> String {
  let co_authors: Vec<String> = commit.co_authors.iter().map(|co_author| { co_author.name.clone() }).collect();

  let author_list: String = if co_authors.len() > 0 {
    format!("{}, {}", commit.author.name, co_authors.join(", "))
  } else {
    format!("{}", commit.author.name)
  };

  return author_list.to_string();
}

impl TableViewItem<BasicColumn> for Commit {
  fn to_column(&self, column: BasicColumn) -> String {
    match column {
      BasicColumn::ShortSha => self.short_sha.to_string(),
      BasicColumn::Subject => format!("{}", self.subject),
      BasicColumn::Authors => format!("{}", authors_and_contributors(self)),
    }
  }

  fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering
    where
      Self: Sized,
    {
      match column {
        BasicColumn::ShortSha => self.short_sha.cmp(&other.short_sha),
        BasicColumn::Subject => self.subject.cmp(&other.subject),
        BasicColumn::Authors => authors_and_contributors(self).cmp(&authors_and_contributors(other)),
      }
    }
}

fn theme(siv: &Cursive) -> Theme {
  // We'll return the current theme with a small modification.
  let mut theme = siv.current_theme().clone();
  theme.palette[PaletteColor::Background] = Color::TerminalDefault;

  theme
}

fn main() {
  let mut siv = cursive::default();

  let theme = theme(&siv);
  siv.set_theme(theme);

  // We can quit by pressing `q`
  siv.add_global_callback('q', Cursive::quit);

  let mut table = TableView::<Commit, BasicColumn>::new()
    .column(BasicColumn::ShortSha, "SHA", |c| c.align(HAlign::Right).width(10))
    .column(BasicColumn::Subject, "Subject", |c| c)
    .column(BasicColumn::Authors, "Authors", |c| {
      c.ordering(Ordering::Greater)
    });

  let commits: Vec<Commit> = interrogate_git_repository();

  table.set_items(commits);

  table.set_on_sort(|siv: &mut Cursive, column: BasicColumn, order: Ordering| {
    siv.add_layer(
      Dialog::around(TextView::new(format!("{} / {:?}", column.as_str(), order)))
        .title("Sorted by")
        .button("Close", |s| {
          s.pop_layer();
        }),
    );
  });

  table.set_on_submit(|siv: &mut Cursive, row: usize, index: usize| {
    let value = siv.call_on_name("table", move |table: &mut TableView<Commit, BasicColumn>| {
        format!("{:?}", table.borrow_item(index).unwrap())
      }).unwrap();

      siv.add_layer(
        Dialog::around(TextView::new(value))
          .title(format!("Removing row # {}", row))
          .button("Close", move |s| {
            s.call_on_name("table", |_table: &mut TableView<Commit, BasicColumn>| {
              // table.remove_item(index);
            });
            s.pop_layer();
          }
        ),
      );
  });

  siv.add_layer(
    ResizedView::with_full_screen(table.with_name("table"))
  );

  siv.run();
}

// fn add_name(s: &mut Cursive) {
//     fn ok(s: &mut Cursive, name: &str) {
//         s.call_on_name("select", |view: &mut SelectView<String>| {
//             view.add_item_str(name)
//         });
//         s.pop_layer();
//     }

//     s.add_layer(Dialog::around(EditView::new()
//             .on_submit(ok)
//             .with_name("name")
//             .fixed_width(10))
//         .title("Enter a new name")
//         .button("Ok", |s| {
//             let name =
//                 s.call_on_name("name", |view: &mut EditView| {
//                     view.get_content()
//                 }).unwrap();
//             ok(s, &name);
//         })
//         .button("Cancel", |s| {
//             s.pop_layer();
//         }));
// }

// fn delete_name(s: &mut Cursive) {
//     let mut select = s.find_name::<SelectView<String>>("select").unwrap();
//     match select.selected_id() {
//         None => s.add_layer(Dialog::info("No name to remove")),
//         Some(focus) => {
//             select.remove_item(focus);
//         }
//     }
// }

// fn on_submit(s: &mut Cursive, name: &str) {
//     s.pop_layer();
//     s.add_layer(Dialog::text(format!("Name: {}\nAwesome: yes", name))
//         .title(format!("{}'s info", name))
//         .button("Quit", Cursive::quit));
// }
