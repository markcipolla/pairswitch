use cursive::Cursive;
use cursive::views::{
  Dialog,
};

pub fn end_pairing_session(siv: &mut Cursive) {
  siv.add_layer(Dialog::text("End current pairing session with NAME?")
    .title("Pairswitch")
    .button("Yes", |s| s.quit())
    .button("No", |s| s.quit())
  );
}
