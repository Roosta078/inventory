use super::applet::Applet;
use crate::AppState;
use crate::db::inventory::{Inventory, Item, Location};
use crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;
use ratatui::style::Style;
use ratatui::widgets::{Block, Padding, Row, Table, TableState};

pub struct ListItemsApplet {
    table_state: TableState,
    items: Vec<Item>,
    location_strings: Vec<String>,
    next_state: AppState,
}

impl Default for ListItemsApplet {
    fn default() -> Self {
        Self {
            table_state: TableState::default().with_selected_cell(Some((0, 0))),
            items: Vec::new(),
            location_strings: Vec::new(),
            next_state: AppState::NoChange,
        }
    }
}

impl Applet for ListItemsApplet {
    fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        _db: &Inventory,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.next_state = AppState::NoChange;

        let header = Row::new(vec!["Item ID", "Name", "Comment", "Location"]);
        let mut rows: Vec<Row> = Vec::new();

        if self.items.is_empty() {
            rows.push(Row::new(["DB ERROR", "DB ERROR", "DB ERROR", "DB ERROR"]))
        } else {
            rows.append(
                &mut self
                    .items
                    .iter()
                    .zip(self.location_strings.iter())
                    .map(|(i, l)| {
                        Row::new([
                            format!("{}", i.id),
                            format!("{}", i.name),
                            format!("{}", i.comment.clone().unwrap_or_default()),
                            format!("{}", l),
                        ])
                    })
                    .collect::<Vec<Row>>(),
            );
        }

        let widths: Vec<u16> = Vec::new();
        let table = Table::new(rows, widths)
            .block(
                Block::bordered()
                    .title("Inventory Manager - List Items")
                    .title_bottom("Press 'q' or Esc to exit")
                    .border_type(ratatui::widgets::BorderType::Thick)
                    .padding(Padding::horizontal(1)),
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
                    self.next_state =
                        AppState::EditItem(self.items[self.table_state.selected().unwrap_or(0)].id)
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
        self.items = db.get_all_items().unwrap_or_default();
        self.location_strings = self
            .items
            .iter()
            .map(|i| {
                db.search_location_id(i.location_id.unwrap_or(0))
                    .map(|item| item.name)
                    .unwrap_or_else(|| "".to_string())
            })
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_default() {
        let my_applet = ListItemsApplet::default();
        assert!(my_applet.items.is_empty());
        assert!(my_applet.location_strings.is_empty());
        assert_eq!(my_applet.next_state, AppState::NoChange);
        assert_eq!(my_applet.table_state.selected(), Some(0));
        assert_eq!(my_applet.table_state.selected_column(), Some(0));
    }

    #[test]
    fn test_refresh() {
        let my_inv: Inventory = Inventory::open_in_memory().unwrap();
        for i in 0..5 {
            let loc = Location {
                id: i,
                name: format!("location{i}").to_string(),
                comment: Some(format!("comment{i}").to_string()),
            };
            let item = Item {
                id: i + 100,
                name: format!("item{i}").to_string(),
                comment: Some(format!("comment{i}").to_string()),
                location_id: Some(i),
            };
            assert!(my_inv.add_location(&loc).is_ok());
            assert!(my_inv.add_item(&item).is_ok());
        }
        let mut my_applet = ListItemsApplet::default();
        my_applet.refresh(&my_inv);
        assert!(!my_applet.items.is_empty());
        assert!(!my_applet.location_strings.is_empty());
        assert_eq!(my_applet.items.len(), 5);
        assert_eq!(my_applet.location_strings.len(), 5);
        assert_eq!(
            my_applet.location_strings.first(),
            Some(&"location0".to_string())
        );
        assert_eq!(my_applet.next_state, AppState::NoChange);
        assert_eq!(my_applet.table_state.selected(), Some(0));
        assert_eq!(my_applet.table_state.selected_column(), Some(0));
    }
    #[test]
    fn test_refresh_no_locations() {
        let my_inv: Inventory = Inventory::open_in_memory().unwrap();
        for i in 0..5 {
            let item = Item {
                id: i + 100,
                name: format!("item{i}").to_string(),
                comment: Some(format!("comment{i}").to_string()),
                location_id: None,
            };
            assert!(my_inv.add_item(&item).is_ok());
        }
        let mut my_applet = ListItemsApplet::default();
        my_applet.refresh(&my_inv);
        assert!(!my_applet.items.is_empty());
        assert!(!my_applet.location_strings.is_empty());
        assert_eq!(my_applet.items.len(), 5);
        assert_eq!(my_applet.location_strings.len(), 5);
        assert_eq!(my_applet.location_strings.first(), Some(&"".to_string()));
        assert_eq!(my_applet.next_state, AppState::NoChange);
        assert_eq!(my_applet.table_state.selected(), Some(0));
        assert_eq!(my_applet.table_state.selected_column(), Some(0));
    }
}
