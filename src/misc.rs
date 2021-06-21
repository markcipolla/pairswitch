
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
