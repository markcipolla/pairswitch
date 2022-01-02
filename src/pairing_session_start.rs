use cursive::Cursive;
use cursive::views::{
  Dialog,
};

pub fn start_pairing_session(siv: &mut Cursive) {
  siv.add_layer(Dialog::text("Start a pairing session?")
    .title("Pairswitch")
    .button("Yes", |s| s.quit())
    .button("No", |s| s.quit())
  );
}
