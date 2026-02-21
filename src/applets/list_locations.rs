use super::applet::Applet;
use crate::AppState;
use crate::db::inventory::{Inventory, Location};
use crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Style;
use ratatui::widgets::{Block, Padding, Paragraph, Row, Table, TableState};

pub struct ListLocationsApplet {
    table_state: TableState,
    locations: Vec<Location>,
    next_state: AppState,
    search: String,
    cursor_position: u16,
}

impl Default for ListLocationsApplet {
    fn default() -> Self {
        Self {
            table_state: TableState::default().with_selected_cell(Some((0, 0))),
            next_state: AppState::NoChange,
            locations: Vec::new(),
            search: String::default(),
            cursor_position: 0,
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
        let vertical = Layout::vertical([Constraint::Min(1), Constraint::Length(3)]);
        let border = Block::bordered()
            .title("Inventory Manager - List Locations")
            .title_bottom("Press Esc to exit")
            .border_type(ratatui::widgets::BorderType::Thick)
            .padding(Padding::horizontal(1));
        let table = Table::new(rows, widths)
            .style(Style::new().white())
            .cell_highlight_style(Style::new().red())
            .row_highlight_style(Style::new().reversed())
            .highlight_symbol(">>")
            .header(header);
        let search_bar = Paragraph::new(self.search.clone())
            .style(Style::default().yellow())
            .block(Block::bordered().title("Search Term"));

        terminal.draw(|frame| {
            let inner_area = border.inner(frame.area());
            let [table_area, search_area] = vertical.areas(inner_area);

            frame.render_widget(border, frame.area());

            frame.render_stateful_widget(table, table_area, &mut self.table_state);
            frame.render_widget(search_bar, search_area);
        })?;

        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Esc => self.next_state = AppState::Exit,
                KeyCode::Down => self.table_state.select_next(),
                KeyCode::Up => self.table_state.select_previous(),
                KeyCode::Left => self.table_state.select_previous_column(),
                KeyCode::Right => self.table_state.select_next_column(),
                KeyCode::Enter => {
                    self.next_state = AppState::EditLocation(
                        self.locations[self.table_state.selected().unwrap_or(0)].id,
                    )
                }
                KeyCode::Char(c) => {
                    self.search.insert(self.cursor_position.into(), c);
                    self.cursor_position += 1;
                    self.locations = db
                        .search_locations(self.search.as_str())
                        .unwrap_or_default();
                }
                KeyCode::Backspace => {
                    if self.cursor_position != 0 {
                        self.cursor_position -= 1;
                        self.search.remove(self.cursor_position.into());
                        if self.cursor_position == 0 {
                            self.refresh(db);
                        } else {
                            self.locations = db
                                .search_locations(self.search.as_str())
                                .unwrap_or_default();
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn get_next_state(&self) -> AppState {
        self.next_state.clone()
    }
    fn refresh(&mut self, db: &Inventory) {
        self.locations = db.get_all_locations().unwrap_or_default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_default() {
        let my_applet = ListLocationsApplet::default();
        assert!(my_applet.locations.is_empty());
        assert_eq!(my_applet.next_state, AppState::NoChange);
        assert_eq!(my_applet.table_state.selected(), Some(0));
        assert_eq!(my_applet.table_state.selected_column(), Some(0));
    }
}
