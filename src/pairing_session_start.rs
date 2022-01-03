use cursive::Cursive;
use cursive::views::{
  Dialog,
};

pub fn start_pairing_session(siv: &mut Cursive) {
  siv.add_layer(Dialog::text("Start a pairing session?")
    .title("Pairswitch")
    .button("Yes", select_pair)
    .button("No", |s| s.quit())
  );
}

fn select_pair(siv: &mut Cursive) {
  siv.pop_layer();
  siv.add_layer(Dialog::text("Who are you pairing with?")
    .title("Pairswitch")
    .button("Finish", |s| s.quit()));
}
