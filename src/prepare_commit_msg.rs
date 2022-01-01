use cursive::Cursive;
use cursive::views::{
  Dialog,
};

pub fn prepare_commit_msg(siv: &mut Cursive) {
  siv.add_layer(Dialog::text("Are you pairing? Add contributor attribution?")
    .title("Pairswitch")
    .button("No", |s| s.quit())
    .button("Yes", |s| select_co_author(s))
  );
}

fn select_co_author(siv: &mut Cursive) {
  siv.pop_layer();
  siv.add_layer(Dialog::text("Who are you pairing with?")
      .title("Pairswitch")
      .button("Finish", |s| s.quit()));
}
