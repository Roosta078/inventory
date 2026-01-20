use crate::db::inventory::Inventory;
use crossterm::event::{self, KeyCode, KeyEvent};
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::{Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{
    Block, List, ListDirection, ListItem, ListState, Paragraph, Row, Table, TableState,
};

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

struct EditLocationApplet {
    next_state: AppState,
    loc: db::inventory::Location,
    cursor_position: u16,
    selection: EditLocationSelection,
}
#[derive(PartialEq)]
enum EditLocationSelection {
    Name,
    Comment,
    Cancel,
    Save,
}

impl EditLocationSelection {
    fn next(&self) -> Self {
        match self {
            EditLocationSelection::Name => EditLocationSelection::Comment,
            EditLocationSelection::Comment => EditLocationSelection::Cancel,
            EditLocationSelection::Cancel => EditLocationSelection::Save,
            EditLocationSelection::Save => EditLocationSelection::Name,
        }
    }
    fn previous(&self) -> Self {
        match self {
            EditLocationSelection::Name => EditLocationSelection::Save,
            EditLocationSelection::Comment => EditLocationSelection::Name,
            EditLocationSelection::Cancel => EditLocationSelection::Comment,
            EditLocationSelection::Save => EditLocationSelection::Cancel,
        }
    }
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

impl Default for EditLocationApplet {
    fn default() -> Self {
        Self {
            next_state: AppState::NoChange,
            loc: db::inventory::Location {
                id: 260126,
                name: "Tweezers".to_string(),
                comment: Some("ESD Safe".to_string()),
            },
            cursor_position: 0,
            selection: EditLocationSelection::Name,
        }
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
        self.next_state = AppState::NoChange;

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
                KeyCode::Enter => self.next_state = AppState::EditLocation, // REMOVE BEFORE FLIGHT
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

impl Applet for EditLocationApplet {
    fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        _db: &Inventory,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.next_state = AppState::NoChange;

        //Prepare Draw
        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(3),
        ]);
        let id_widget = Paragraph::new(self.loc.id.to_string())
            .style(Style::default())
            .block(Block::bordered().title("Location ID"));
        let name_widget = Paragraph::new(self.loc.name.to_string())
            .style(if self.selection == EditLocationSelection::Name {
                Style::default().yellow()
            } else {
                Style::default()
            })
            .block(Block::bordered().title("Name"));
        let comment_widget = Paragraph::new(self.loc.comment.clone().unwrap_or("".to_string()))
            .style(if self.selection == EditLocationSelection::Comment {
                Style::default().yellow()
            } else {
                Style::default()
            })
            .block(Block::bordered().title("Comment"));
        let cancel_button = Paragraph::new("Cancel".to_string())
            .style(if self.selection == EditLocationSelection::Cancel {
                Style::default().yellow()
            } else {
                Style::default()
            })
            .block(Block::bordered());
        let save_button = Paragraph::new("Save Changes".to_string())
            .style(if self.selection == EditLocationSelection::Save {
                Style::default().yellow()
            } else {
                Style::default()
            })
            .block(Block::bordered());

        terminal.draw(|frame| {
            let [id_area, name_area, comment_area, cancel_area, save_area] =
                vertical.areas(frame.area());
            frame.render_widget(id_widget, id_area);
            frame.render_widget(name_widget, name_area);
            frame.render_widget(comment_widget, comment_area);
            frame.render_widget(cancel_button, cancel_area);
            frame.render_widget(save_button, save_area);
            match self.selection {
                EditLocationSelection::Name => {
                    frame.set_cursor_position(Position::new(
                        name_area.x + self.cursor_position + 1,
                        name_area.y + 1,
                    ));
                }
                EditLocationSelection::Comment => {
                    frame.set_cursor_position(Position::new(
                        comment_area.x + self.cursor_position + 1,
                        comment_area.y + 1,
                    ));
                }
                _ => (),
            }
        })?;

        //Handle Input
        if let Some(key) = event::read()?.as_key_press_event() {
            match self.selection {
                EditLocationSelection::Name => match key.code {
                    KeyCode::Char(c) => {
                        self.loc.name.insert(self.cursor_position.into(), c);
                        self.cursor_position += 1
                    }
                    KeyCode::Backspace => {
                        if self.cursor_position != 0 {
                            self.cursor_position -= 1;
                            self.loc.name.remove(self.cursor_position.into());
                        }
                    }
                    KeyCode::Left => self.cursor_position = self.cursor_position.saturating_sub(1),
                    KeyCode::Right => {
                        self.cursor_position = self.cursor_position.saturating_add(1).min(8)
                    }
                    _ => {}
                },

                EditLocationSelection::Comment => match key.code {
                    KeyCode::Char(c) => {
                        self.loc.name.insert(self.cursor_position.into(), c);
                        self.cursor_position += 1
                    }
                    KeyCode::Backspace => {
                        if self.cursor_position != 0 {
                            self.cursor_position -= 1;
                        }
                    }
                    KeyCode::Left => self.cursor_position = self.cursor_position.saturating_sub(1),
                    KeyCode::Right => {
                        self.cursor_position = self.cursor_position.saturating_add(1).min(8)
                    }
                    _ => {}
                },
                EditLocationSelection::Cancel => match key.code {
                    KeyCode::Enter => self.next_state = AppState::Exit,
                    _ => (),
                },
                EditLocationSelection::Save => match key.code {
                    KeyCode::Enter => self.next_state = AppState::Exit, //TODO implement save
                    _ => (),
                },
            }

            match key.code {
                KeyCode::Esc => self.next_state = AppState::Exit,
                KeyCode::Down | KeyCode::Tab => self.selection = self.selection.next(),
                KeyCode::Up => self.selection = self.selection.previous(),
                _ => {}
            }
        }
        Ok(())
    }
    fn get_name(&self) -> String {
        "Edit Location".to_string()
    }
    fn get_next_state(&self) -> AppState {
        self.next_state.clone()
    }
}

#[derive(Clone)]
enum AppState {
    TopMenu,
    ListItems,
    ListLocations,
    Exit,
    EditLocation,
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
                // It is possible to create new applets until we run out of memory.  Probably should add limits at some point
                AppState::ListItems => self.applets.push(Box::new(ListItemsApplet::default())),
                AppState::ListLocations => {
                    self.applets.push(Box::new(ListLocationApplet::default()))
                }
                AppState::EditLocation => {
                    self.applets.push(Box::new(EditLocationApplet::default()))
                }
                AppState::Exit => _ = self.applets.pop(),
                _ => (),
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
