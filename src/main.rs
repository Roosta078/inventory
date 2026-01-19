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
    fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        _db: &Inventory,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let text = Text::raw("TODO");
        terminal.draw(|frame| frame.render_widget(text, frame.area()))?;
        Ok(())
    }
    fn get_name(&self) -> String;
    fn get_next_state(&self) -> AppState;
    fn refresh(&self) {}
}

struct TopMenuApplet {
    exit_applet: bool,
    list_state: ListState,
    next_state: AppState,
}
struct ListLocationApplet {
    exit_applet: bool,
    table_state: TableState,
    next_state: AppState,
}
struct ListItemsApplet {
    exit_applet: bool,
}

impl Default for TopMenuApplet {
    fn default() -> Self {
        Self {
            exit_applet: false,
            list_state: ListState::default().with_selected(Some(0)),
            next_state: AppState::NoChange,
        }
    }
}
impl Default for ListLocationApplet {
    fn default() -> Self {
        Self {
            exit_applet: false,
            table_state: TableState::default().with_selected_cell(Some((1, 0))),
            next_state: AppState::NoChange,
        }
    }
}
impl Default for ListItemsApplet {
    fn default() -> Self {
        Self { exit_applet: false }
    }
}

impl Applet for TopMenuApplet {
    fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        _db: &Inventory,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.exit_applet = false;
        self.next_state = AppState::NoChange;
        let list = List::new(["List Locations", "List Items", "Exit"])
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
                KeyCode::Char('q') | KeyCode::Esc => self.next_state = AppState::Exit,
                KeyCode::Down => self.list_state.select_next(),
                KeyCode::Up => self.list_state.select_previous(),
                KeyCode::Enter => match self.list_state.selected().unwrap_or(2) {
                    0 => self.next_state = AppState::ListLocations,
                    1 => self.next_state = AppState::ListItems,
                    2 => self.next_state = AppState::Exit,
                    _ => (),
                },
                _ => {}
            }
        }
        Ok(())
    }

    fn get_name(&self) -> String {
        "Top Menu".to_string()
    }

    fn get_next_state(&self) -> AppState {
        self.next_state.clone()
    }
}

impl Applet for ListLocationApplet {
    fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        db: &Inventory,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.exit_applet = false;

        let data = db.get_all_locations().unwrap_or_default();

        let mut rows = vec![Row::new(vec!["Location ID", "Name", "Comment"])];

        if data.is_empty() {
            rows.push(Row::new(["DB ERROR", "DB ERROR", "DB_ERROR"]));
        } else {
            rows.append(
                &mut data
                    .iter()
                    .map(|l| {
                        Row::new([
                            format!("{}", l.id),
                            format!("{}", l.name),
                            format!("{}", l.comment.clone().unwrap_or("".to_string())),
                        ])
                    })
                    .collect::<Vec<Row>>(),
            );
        }
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
            .cell_highlight_style(Style::new().red())
            .row_highlight_style(Style::new().reversed())
            .highlight_symbol(">>");

        terminal.draw(|frame| {
            frame.render_stateful_widget(table, frame.area(), &mut self.table_state)
        })?;

        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.next_state = AppState::Exit,
                KeyCode::Down => self.table_state.select_next(),
                KeyCode::Up => self.table_state.select_previous(),
                KeyCode::Left => self.table_state.select_previous_column(),
                KeyCode::Right => self.table_state.select_next_column(),
                KeyCode::Enter => self.next_state = AppState::ListLocations, // REMOVE BEFORE FLIGHT
                _ => {}
            }
        }
        Ok(())
    }
    fn get_name(&self) -> String {
        "List Locations".to_string()
    }
    fn get_next_state(&self) -> AppState {
        self.next_state.clone()
    }
}
impl Applet for ListItemsApplet {
    fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        db: &Inventory,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
            AppState::Exit
        } else {
            AppState::NoChange
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
        app.applets.push(Box::new(TopMenuApplet::default()));
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
                AppState::ListItems => self.applets.push(Box::new(ListItemsApplet::default())),
                AppState::ListLocations => {
                    self.applets.push(Box::new(ListLocationApplet::default()))
                }
                AppState::Exit => _ = self.applets.pop(),
                _ => (),
            }
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

    // fn run_list_locations(
    //     &mut self,
    //     terminal: &mut DefaultTerminal,
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     let mut table_state = TableState::default();
    //     let rows = [
    //         Row::new(vec!["Row11", "Row12", "Row13"]),
    //         Row::new(vec!["Row21", "Row22", "Row23"]),
    //         Row::new(vec!["Row31", "Row32", "Row33"]),
    //     ];
    //     let widths = [
    //         Constraint::Length(5),
    //         Constraint::Length(5),
    //         Constraint::Length(10),
    //     ];
    //     let table = Table::new(rows, widths)
    //         .block(
    //             Block::bordered()
    //                 .title("Inventory Manager")
    //                 .title_bottom("Press 'q' or Esc to exit"),
    //         )
    //         .style(Style::new().white())
    //         .row_highlight_style(Style::new().reversed())
    //         .highlight_symbol(">>");

    //     terminal
    //         .draw(|frame| frame.render_stateful_widget(table, frame.area(), &mut table_state))?;

    //     if let Some(key) = event::read()?.as_key_press_event() {
    //         match key.code {
    //             KeyCode::Char('q') | KeyCode::Esc => self.state = AppState::TopMenu,
    //             _ => {}
    //         }
    //     }
    //     Ok(())
    // }
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
