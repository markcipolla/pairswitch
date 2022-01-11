use cursive::Cursive;
use cursive::views::{
  Dialog,
  LinearLayout,
  ResizedView,
};

#[path = "select_pairs.rs"]
mod select_pairs;
use select_pairs::{ table_view };

pub fn start_pairing_session(siv: &mut Cursive) {
  siv.add_layer(Dialog::text("Start a pairing session?")
    .title("Pairswitch")
    .button("Yes", select_pair)
    .button("No", |s| s.quit())
  );
}


fn select_pair(siv: &mut Cursive) {
  siv.pop_layer();

  let contents = table_view();

  let layout = LinearLayout::vertical()
    .child(ResizedView::with_full_screen(contents));
  siv.add_layer(layout);
}
