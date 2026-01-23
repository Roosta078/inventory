use super::applet::Applet;
use crate::AppState;
use crate::db::inventory::{Inventory, Location};
use crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;
use ratatui::style::Style;
use ratatui::widgets::{Block, Row, Table, TableState};

pub struct ListLocationsApplet {
    exit_applet: bool,
    table_state: TableState,
    locations: Vec<Location>,
    next_state: AppState,
}

impl Default for ListLocationsApplet {
    fn default() -> Self {
        Self {
            exit_applet: false,
            table_state: TableState::default().with_selected_cell(Some((0, 0))),
            next_state: AppState::NoChange,
            locations: Vec::new(),
        }
    }
}

impl Applet for ListLocationsApplet {
    fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        db: &Inventory,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.next_state = AppState::NoChange;

        //let data = db.get_all_locations().unwrap_or_default();

        let header = Row::new(vec!["Location ID", "Name", "Comment"]);
        let mut rows = Vec::new();

        if self.locations.is_empty() {
            rows.push(Row::new(["DB ERROR", "DB ERROR", "DB_ERROR"]));
        } else {
            rows.append(
                &mut self
                    .locations
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
            .highlight_symbol(">>")
            .header(header);

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
                KeyCode::Char('e') | KeyCode::Enter => {
                    self.next_state = AppState::EditLocation(
                        self.locations[self.table_state.selected().unwrap_or(0)].id,
                    )
                }
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
    fn refresh(&mut self, db: &Inventory) {
        self.locations = db.get_all_locations().unwrap_or_default();
    }
}
