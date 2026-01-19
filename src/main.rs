use crossterm::event::{self, KeyCode, KeyEvent};
use ratatui::DefaultTerminal;
use ratatui::layout::Constraint;
use ratatui::style::{Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Block, List, ListDirection, ListItem, ListState, Row, Table, TableState};
mod db;

struct App {
    state: AppState,
    applets: Vec<Box<dyn Applet>>,
    list_state: ListState,
}

pub trait Applet {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
        let text = Text::raw("TODO");
        terminal.draw(|frame| frame.render_widget(text, frame.area()))?;
        Ok(())
    }
    fn get_name(&self) -> String;
}

struct ListLocationApplet {}
struct ListItemsApplet {}

impl Applet for ListLocationApplet {
    // fn run(&mut self, _terminal: &mut DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
    //     Ok(())
    // }
    fn get_name(&self) -> String {
        "List Locations".to_string()
    }
}
impl Applet for ListItemsApplet {
    // fn run(&mut self, _terminal: &mut DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
    //     Ok(())
    // }
    fn get_name(&self) -> String {
        "List Items".to_string()
    }
}

enum AppState {
    TopMenu,
    ListItems,
    ListLocations,
    Exit,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self::new();
        app.applets.push(Box::new(ListLocationApplet {}));
        app.applets.push(Box::new(ListItemsApplet {}));
        app
    }
}

impl App {
    fn new() -> Self {
        Self {
            state: AppState::TopMenu,
            applets: Vec::new(),
            list_state: ListState::default(),
        }
    }
    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
        self.list_state.select_first();
        loop {
            match self.state {
                AppState::Exit => break,
                AppState::TopMenu => self.run_top_menu(terminal)?,
                AppState::ListLocations => self.applets[0].run(terminal)?,
                AppState::ListItems => self.applets[1].run(terminal)?,
                _ => break,
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
    let myapp = App::default();
    ratatui::run(|terminal| myapp.run(terminal))
}
