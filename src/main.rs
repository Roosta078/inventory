use crate::db::inventory::Inventory;
use crossterm::event::{self, KeyCode, KeyEvent};
use ratatui::DefaultTerminal;
use ratatui::layout::Constraint;
use ratatui::style::{Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Block, List, ListDirection, ListItem, ListState, Row, Table, TableState};
use std::rc::Rc;
mod db;

struct App {
    state: AppState,
    applets: Vec<Box<dyn Applet>>,
    list_state: ListState,
    db: Inventory,
}

pub trait Applet {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
        let text = Text::raw("TODO");
        terminal.draw(|frame| frame.render_widget(text, frame.area()))?;
        Ok(())
    }
    fn get_name(&self) -> String;
    fn get_next_state(&self) -> AppState;
}

struct ListLocationApplet {
    exit_applet: bool,
}
struct ListItemsApplet {
    exit_applet: bool,
}

impl Default for ListLocationApplet {
    fn default() -> Self {
        Self { exit_applet: false }
    }
}
impl Default for ListItemsApplet {
    fn default() -> Self {
        Self { exit_applet: false }
    }
}

impl Applet for ListLocationApplet {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
        self.exit_applet = false;
        let mut table_state = TableState::default();
        let rows = [
            Row::new(vec!["Row11", "Row12", "Row13"]),
            Row::new(vec!["Row21", "Row22", "Row23"]),
            Row::new(vec!["Row31", "Row32", "Row33"]),
        ];

        // let widths = [
        //     Constraint::Length(5),
        //     Constraint::Length(10),
        //     Constraint::Length(10),
        // ];
        let widths: Vec<u16> = Vec::new();
        let table = Table::new(rows, widths)
            .block(
                Block::bordered()
                    .title("Inventory Manager - List Locations")
                    .title_bottom("Press 'q' or Esc to exit"),
            )
            .style(Style::new().white())
            .row_highlight_style(Style::new().reversed())
            .highlight_symbol(">>");

        terminal
            .draw(|frame| frame.render_stateful_widget(table, frame.area(), &mut table_state))?;

        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.exit_applet = true,
                _ => {}
            }
        }
        Ok(())
    }
    fn get_name(&self) -> String {
        "List Locations".to_string()
    }
    fn get_next_state(&self) -> AppState {
        if self.exit_applet {
            AppState::TopMenu
        } else {
            AppState::ListLocations
        }
    }
}
impl Applet for ListItemsApplet {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
        self.exit_applet = false;
        let text = Text::raw("TODO");
        terminal.draw(|frame| frame.render_widget(text, frame.area()))?;
        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.exit_applet = true,
                _ => {}
            }
        }
        Ok(())
    }
    fn get_name(&self) -> String {
        "List Items".to_string()
    }
    fn get_next_state(&self) -> AppState {
        if self.exit_applet {
            AppState::TopMenu
        } else {
            AppState::ListItems
        }
    }
}
#[derive(Clone)]
enum AppState {
    TopMenu,
    ListItems,
    ListLocations,
    Exit,
    NoChange,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self::new();
        app.applets.push(Box::new(ListLocationApplet::default()));
        app.applets.push(Box::new(ListItemsApplet::default()));
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
        self.list_state.select_first();
        loop {
            match self.state {
                AppState::Exit => break,
                AppState::TopMenu => self.run_top_menu(terminal)?,
                AppState::ListLocations => {
                    self.applets[0].run(terminal)?;
                    self.state = self.applets[0].get_next_state();
                }
                AppState::ListItems => {
                    self.applets[1].run(terminal)?;
                    self.state = self.applets[1].get_next_state();
                }
                _ => break,
            };
        }
        Ok(())
    }

    fn run_top_menu(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let list = List::new(
            self.applets
                .iter()
                .map(|a| a.get_name())
                .collect::<Vec<String>>(),
        )
        .block(
            Block::bordered()
                .title("Inventory Manager")
                .title_bottom("Press 'q' or Esc to exit"),
        )
        .style(Style::new().white())
        .highlight_style(Style::new().bold())
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom);

        terminal
            .draw(|frame| frame.render_stateful_widget(list, frame.area(), &mut self.list_state))?;
        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.state = AppState::Exit,
                KeyCode::Down => self.list_state.select_next(),
                KeyCode::Up => self.list_state.select_previous(),
                KeyCode::Enter => match self.list_state.selected().unwrap_or(2) {
                    0 => self.state = AppState::ListLocations,
                    1 => self.state = AppState::ListItems,
                    2 => self.state = AppState::Exit,
                    _ => (),
                },
                _ => {}
            }
        }
        Ok(())
    }

    fn run_list_locations(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut table_state = TableState::default();
        let rows = [
            Row::new(vec!["Row11", "Row12", "Row13"]),
            Row::new(vec!["Row21", "Row22", "Row23"]),
            Row::new(vec!["Row31", "Row32", "Row33"]),
        ];
        let widths = [
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Length(10),
        ];
        let table = Table::new(rows, widths)
            .block(
                Block::bordered()
                    .title("Inventory Manager")
                    .title_bottom("Press 'q' or Esc to exit"),
            )
            .style(Style::new().white())
            .row_highlight_style(Style::new().reversed())
            .highlight_symbol(">>");

        terminal
            .draw(|frame| frame.render_stateful_widget(table, frame.area(), &mut table_state))?;

        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.state = AppState::TopMenu,
                _ => {}
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let l1 = db::inventory::Location {
        id: 101,
        name: "location1".to_string(),
        comment: None,
    };
    let l2 = db::inventory::Location {
        id: 102,
        name: "location2".to_string(),
        comment: Some("with comment".to_string()),
    };

    let myapp = App::default();
    myapp.db.add_location(&l1)?;
    myapp.db.add_location(&l2)?;
    ratatui::run(|terminal| myapp.run(terminal))
}
