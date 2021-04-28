// Crate Dependencies ---------------------------------------------------------
extern crate cursive;
extern crate cursive_table_view;
extern crate rand;

// STD Dependencies -----------------------------------------------------------
use std::cmp::Ordering;

// External Dependencies ------------------------------------------------------
// ----------------------------------------------------------------------------
use cursive_table_view::{TableViewItem, TableView};
use cursive::{
    Cursive,
    align::HAlign,
    traits::*,
    views::{
        Button,
        Dialog,
        DummyView,
        EditView,
        LinearLayout,
        SelectView,
        TextView
    }
};


use rand::Rng;


#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum BasicColumn {
    Name,
    Count,
    Rate,
}

impl BasicColumn {
    fn as_str(&self) -> &str {
        match *self {
            BasicColumn::Name => "Name",
            BasicColumn::Count => "Count",
            BasicColumn::Rate => "Rate",
        }
    }
}

#[derive(Clone, Debug)]
struct Foo {
    name: String,
    count: usize,
    rate: usize,
}

impl TableViewItem<BasicColumn> for Foo {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Name => self.name.to_string(),
            BasicColumn::Count => format!("{}", self.count),
            BasicColumn::Rate => format!("{}", self.rate),
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            BasicColumn::Name => self.name.cmp(&other.name),
            BasicColumn::Count => self.count.cmp(&other.count),
            BasicColumn::Rate => self.rate.cmp(&other.rate),
        }
    }
}

fn create_table() -> TableView<Foo, BasicColumn> {
    let mut items = Vec::new();
    let mut rng = rand::thread_rng();

    for i in 0..50 {
        items.push(Foo {
            name: format!("Name {}", i),
            count: rng.gen_range(0..255),
            rate: rng.gen_range(0..255),
        });
    }

    TableView::<Foo, BasicColumn>::new()
        .column(BasicColumn::Name, "Name", |c| c.width_percent(20))
        .column(BasicColumn::Count, "Count", |c| c.align(HAlign::Center))
        .column(BasicColumn::Rate, "Rate", |c| {
            c.ordering(Ordering::Greater)
                .align(HAlign::Right)
                .width_percent(20)
        })
        .items(items)
}

fn main() {
    let mut rng = rand::thread_rng();
    let mut siv = cursive::default();

    let select = SelectView::<String>::new()
        .on_submit(on_submit)
        .with_name("select")
        .fixed_size((10, 5));

    let buttons = LinearLayout::vertical()
        .child(Button::new("Add new", add_name))
        .child(Button::new("Delete", delete_name))
        .child(DummyView)
        .child(Button::new("Quit", Cursive::quit));

    siv.add_layer(Dialog::around(LinearLayout::horizontal()
            .child(select)
            .child(DummyView)
            .child(buttons))
        .title("Select a profile"));

    siv.call_on_name("select", |view: &mut SelectView<String>| {
      view.add_item_str("Staring name from cimmits")
    });

    let mut table = TableView::<Foo, BasicColumn>::new()
        .column(BasicColumn::Name, "Name", |c| c.width_percent(20))
        .column(BasicColumn::Count, "Count", |c| c.align(HAlign::Center))
        .column(BasicColumn::Rate, "Rate", |c| {
            c.ordering(Ordering::Greater)
                .align(HAlign::Right)
                .width_percent(20)
        });

    let mut items = Vec::new();
    for i in 0..50 {
        items.push(Foo {
            name: format!("Name {}", i),
            count: rng.gen_range(0..=255),
            rate: rng.gen_range(0..=255),
        });
    }

    table.set_items(items);

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
        let value = siv
            .call_on_name("table", move |table: &mut TableView<Foo, BasicColumn>| {
                format!("{:?}", table.borrow_item(index).unwrap())
            })
            .unwrap();

        siv.add_layer(
            Dialog::around(TextView::new(value))
                .title(format!("Removing row # {}", row))
                .button("Close", move |s| {
                    s.call_on_name("table", |table: &mut TableView<Foo, BasicColumn>| {
                        table.remove_item(index);
                    });
                    s.pop_layer();
                }),
        );
    });

    siv.add_layer(Dialog::around(table.with_name("table").min_size((50, 20))).title("Table View"));

    siv.run();
}

fn add_name(s: &mut Cursive) {
    fn ok(s: &mut Cursive, name: &str) {
        s.call_on_name("select", |view: &mut SelectView<String>| {
            view.add_item_str(name)
        });
        s.pop_layer();
    }

    s.add_layer(Dialog::around(EditView::new()
            .on_submit(ok)
            .with_name("name")
            .fixed_width(10))
        .title("Enter a new name")
        .button("Ok", |s| {
            let name =
                s.call_on_name("name", |view: &mut EditView| {
                    view.get_content()
                }).unwrap();
            ok(s, &name);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }));
}

fn delete_name(s: &mut Cursive) {
    let mut select = s.find_name::<SelectView<String>>("select").unwrap();
    match select.selected_id() {
        None => s.add_layer(Dialog::info("No name to remove")),
        Some(focus) => {
            select.remove_item(focus);
        }
    }
}

fn on_submit(s: &mut Cursive, name: &str) {
    s.pop_layer();
    s.add_layer(Dialog::text(format!("Name: {}\nAwesome: yes", name))
        .title(format!("{}'s info", name))
        .button("Quit", Cursive::quit));
}
