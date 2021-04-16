use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Row, Table, Tabs, TableState
    },
    Terminal,
};

use git2::Repository;

const DB_PATH: &str = "./data/db.json";

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Serialize, Deserialize, Clone)]
struct Commit {
    id: usize,
    name: String,
    category: String,
    age: usize,
    created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Home,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
        }
    }
}

pub struct StatefulTable<'a> {
    state: TableState,
    items: Vec<Vec<&'a str>>,
}


impl<'a> StatefulTable<'a> {
    fn new() -> StatefulTable<'a> {
        StatefulTable {
            state: TableState::default(),
            items: vec![
                vec!["Row11", "Row12", "Row13"],
                vec!["Row21", "Row22", "Row23"],
                vec!["Row31", "Row32", "Row33"],
                vec!["Row41", "Row42", "Row43"],
                vec!["Row51", "Row52", "Row53"],
                vec!["Row61", "Row62\nTest", "Row63"],
                vec!["Row71", "Row72", "Row73"],
                vec!["Row81", "Row82", "Row83"],
                vec!["Row91", "Row92", "Row93"],
                vec!["Row101", "Row102", "Row103"],
                vec!["Row111", "Row112", "Row113"],
                vec!["Row121", "Row122", "Row123"],
                vec!["Row131", "Row132", "Row133"],
                vec!["Row141", "Row142", "Row143"],
                vec!["Row151", "Row152", "Row153"],
                vec!["Row161", "Row162", "Row163"],
                vec!["Row171", "Row172", "Row173"],
                vec!["Row181", "Row182", "Row183"],
                vec!["Row191", "Row192", "Row193"],
            ],
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can run in raw mode");

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let menu_titles = vec!["Home", "Change ownership", "Quit"];
    let mut active_menu_item = MenuItem::Home;
    let mut commit_list_state = ListState::default();
    commit_list_state.select(Some(0));

    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2)
                    ]
                    .as_ref(),
                )
                .split(size);

            let menu = menu_titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                })
                .collect();

            let tabs = Tabs::new(menu)
                .select(active_menu_item.into())
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);

            match active_menu_item {
                MenuItem::Home => {
                    let pets_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Length(15), Constraint::Min(10)].as_ref(),
                        )
                        .split(chunks[1]);

                    let (list, right) = render_commit_list(&commit_list_state);

                    rect.render_stateful_widget(list, pets_chunks[0], &mut commit_list_state);
                    rect.render_widget(right, pets_chunks[1]);
                }
            }
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }

                KeyCode::Down => {
                    if let Some(selected) = commit_list_state.selected() {
                        let amount_pets = read_db().expect("can fetch pet list").len();
                        if selected >= amount_pets - 1 {
                            commit_list_state.select(Some(0));
                        } else {
                            commit_list_state.select(Some(selected + 1));
                        }
                    }
                }

                KeyCode::Up => {
                    if let Some(selected) = commit_list_state.selected() {
                        let amount_pets = read_db().expect("can fetch pet list").len();
                        if selected > 0 {
                            commit_list_state.select(Some(selected - 1));
                        } else {
                            commit_list_state.select(Some(amount_pets - 1));
                        }
                    }
                }
                _ => {}
            },

            Event::Tick => {}
        }
    }

    Ok(())
}


fn render_commit_list<'a>(commit_list_state: &ListState) -> (List<'a>, Table<'a>) {
    let commit_list = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Commits")
        .border_type(BorderType::Plain);

    let repo = match Repository::discover("./") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };



    let pet_list = read_db().expect("can fetch pet list");
    let items: Vec<_> = pet_list
        .iter()
        .map(|pet| {
            ListItem::new(
                Spans::from(
                    vec![
                        Span::styled(pet.name.clone(), Style::default())
                    ]
                )
            )
        })
        .collect();

    let commits = pet_list
        .get(commit_list_state.selected().expect("there is always a selected pet"))
        .expect("exists")
        .clone();

    let list = List::new(items).block(commit_list).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let commit_list = Table::new(
        vec![Row::new(vec![
            Cell::from(Span::raw(commits.id.to_string())),
            Cell::from(Span::raw(commits.name)),
            Cell::from(Span::raw(commits.category)),
            Cell::from(Span::raw(commits.age.to_string())),
            Cell::from(Span::raw(commits.created_at.to_string())),
        ])]
    )
    .highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    )
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "Commit",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Author",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Date",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Age",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Created At",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Detail")
            .border_type(BorderType::Plain),
    )
    .widths(&[
        Constraint::Percentage(5),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(5),
        Constraint::Percentage(20),
    ]);

    (list, commit_list)
}

fn read_db() -> Result<Vec<Commit>, Error> {
    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: Vec<Commit> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}


