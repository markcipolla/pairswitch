use cursive::Cursive;
use cursive::views::{
  Dialog,
};

pub fn initialize_git(siv: &mut Cursive) {
  let init_text = "This will add git hooks to help adding pair programming contributions.";

  siv.add_layer(Dialog::text(init_text)
    .title("Pairswitch")
    .button("Next", show_next));

}

fn show_answer(siv: &mut Cursive, msg: &str) {
  siv.pop_layer();
  siv.add_layer(Dialog::text(msg)
    .title("Results")
    .button("Finish", |s| s.quit()));
}

fn show_next(siv: &mut Cursive) {
  siv.pop_layer();
  siv.add_layer(Dialog::text("Did you do the thing?")
    .title("Question 1")
    .button("Yes!", |s| show_answer(s, "I knew it! Well done!"))
    .button("No!", |s| show_answer(s, "I knew you couldn't be trusted!"))
    .button("Uh?", |s| s.add_layer(Dialog::info("Try again!"))));
}
