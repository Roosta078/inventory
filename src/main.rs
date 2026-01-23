use crossterm::event::{self, KeyCode, KeyEvent};
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::{Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{
    Block, List, ListDirection, ListItem, ListState, Paragraph, Row, Table, TableState,
};

mod applets;
use crate::db::inventory::{Inventory, Location};

mod db;
// use applets::{
//     Applet, CreateLocationApplet, EditLocationApplet, ListItemsApplet, ListLocationsApplet,
//     TopMenuApplet,
// };

struct App {
    state: AppState,
    applets: Vec<Box<dyn applets::Applet>>,
    list_state: ListState,
    db: Inventory,
}

#[derive(Clone)]
enum AppState {
    TopMenu,
    ListItems,
    ListLocations,
    Exit,
    EditLocation(i64),
    NoChange,
    CreateLocation,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self::new();
        app.applets
            .push(Box::new(applets::TopMenuApplet::default()));
        app
    }
}

impl App {
    fn new() -> Self {
        Self {
            state: AppState::TopMenu,
            applets: Vec::new(),
            list_state: ListState::default(),
            db: Inventory::open_in_memory().unwrap(),
        }
    }
    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(top_applet) = self.applets.last_mut() {
            top_applet.run(terminal, &self.db)?;
            match top_applet.get_next_state() {
                // It is possible to create new applets until we run out of memory.  Probably should add limits at some point
                AppState::ListItems => self
                    .applets
                    .push(Box::new(applets::ListItemsApplet::default())),
                AppState::ListLocations => self
                    .applets
                    .push(Box::new(applets::ListLocationsApplet::default())),
                AppState::EditLocation(id) => self
                    .applets
                    .push(Box::new(applets::EditLocationApplet::new(id))),
                AppState::CreateLocation => self
                    .applets
                    .push(Box::new(applets::CreateLocationApplet::new())),
                AppState::Exit => _ = self.applets.pop(),
                _ => continue,
            }
            if let Some(new_top) = self.applets.last_mut() {
                new_top.refresh(&self.db);
            }
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let myapp = App::default();
    for i in 0..100 {
        let loc = db::inventory::Location {
            id: i,
            name: format!("location{i}").to_string(),
            comment: Some(format!("comment{i}").to_string()),
        };
        myapp.db.add_location(&loc)?;
    }

    ratatui::run(|terminal| myapp.run(terminal))
}
