// Crate Dependencies ---------------------------------------------------------
extern crate cursive;
extern crate cursive_table_view;
extern crate rand;

// Local modules
#[path = "git.rs"]
mod git;
use git::interrogate_git_repository;

#[path = "table.rs"]
mod table;
use table::table_view;

#[path = "structs.rs"]
mod structs;
use structs::{ Commit };

// External Dependencies ------------------------------------------------------

use cursive::{
    Cursive,
    traits::*,
    theme::{
      Color,
      PaletteColor,
      Theme,
    },
    views::{
      ResizedView,
      LinearLayout,
      TextView,
      Button
    }
};

fn theme(siv: &Cursive) -> Theme {
  // We'll return the current theme with a small modification.
  let mut theme = siv.current_theme().clone();
  // theme.palette[PaletteColor::Background] = Color::Rgb(44, 62, 8);
  theme.palette[PaletteColor::View] = Color::Rgb(236, 240, 241);
  theme.palette[PaletteColor::Primary] = Color::Rgb(44, 62, 8);
  // theme.palette[PaletteColor::Secondary] = Color::Rgb(52, 152, 219);
  // theme.palette[PaletteColor::Tertiary] = Color::Rgb(41, 128, 185);
  // theme.palette[PaletteColor::TitlePrimary] = Color::Rgb(180,180,180);
  // theme.palette[PaletteColor::TitleSecondary] = Color::Rgb(200,200,200);
  // theme.palette[PaletteColor::Highlight] = Color::Rgb(231, 76, 60);
  // theme.palette[PaletteColor::HighlightInactive] = Color::Rgb(255,255,255);
  // theme.shadow = false;
  theme
}


fn draw_start_pairing_session() -> LinearLayout {
  LinearLayout::vertical()
    .child(TextView::new("Start a pairing session?"))
    .child(Button::new("Ok", |s| s.quit()))
}

fn draw_main_interface(commits: Vec<Commit>) -> LinearLayout {
  LinearLayout::vertical()
    .child(draw_start_pairing_session())
    .child(ResizedView::with_full_screen(table_view(commits).with_name("table")))
}

fn main() {
  let mut siv = cursive::default();
  let commits: Vec<Commit> = interrogate_git_repository();
  let theme = theme(&siv);
  siv.set_theme(theme);

  // We can quit by pressing `q`
  siv.add_global_callback('q', Cursive::quit);

  siv.add_layer(draw_main_interface(commits));

  siv.set_fps(32);
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
