use super::applet::Applet;
use crate::AppState;
use crate::db::inventory;
use crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph};
use std::error;
use std::fmt;

pub struct ItemLookupApplet {
    next_state: AppState,
    cursor_position: u16,
    id: String,
}

#[derive(Debug)]
struct ItemLookupError {
    error_text: String,
}

impl fmt::Display for ItemLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Item Lookup Error: {}", self.error_text)
    }
}

impl error::Error for ItemLookupError {}

impl ItemLookupError {
    fn new(msg: &str) -> Box<ItemLookupError> {
        Box::new(ItemLookupError {
            error_text: msg.to_string(),
        })
    }
}

impl ItemLookupApplet {
    pub fn new() -> Self {
        Self {
            next_state: AppState::NoChange,
            cursor_position: 0,
            id: String::default(),
        }
    }

    fn find_item(&mut self, db: &inventory::Inventory) -> Result<(), Box<dyn std::error::Error>> {
        let id = self
            .id
            .parse::<i64>()
            .map_err(|_| ItemLookupError::new("Failed to parse Item ID"))?;

        if !db.item_exists(id) {
            return Err(ItemLookupError::new("Item ID does not exist"));
        }

        self.next_state = AppState::EditItem(id);
        Ok(())
    }
}

impl Applet for ItemLookupApplet {
    fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        db: &inventory::Inventory,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.next_state = AppState::NoChange;
        let border = Block::bordered()
            .title_top("Inventory Manager")
            .title_bottom("Press 'q' or Esc to exit");
        let vertical = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ]);
        let line1 = Paragraph::new("Enter the Desired Item ID and hit 'Enter'")
            .style(Style::default())
            .centered();
        let id_widget = Paragraph::new(self.id.clone())
            .style(Style::default().yellow())
            .block(Block::bordered().title("Item ID"));

        terminal.draw(|frame| {
            let inner_area = border.inner(frame.area());
            let [_, l1_area, id_area, _] = vertical.areas(inner_area);
            let cent_id_area = id_area.centered_horizontally(Constraint::Length(20));
            frame.render_widget(border, frame.area());
            frame.render_widget(line1, l1_area);
            frame.render_widget(id_widget, cent_id_area);
            frame.set_cursor_position(Position::new(
                cent_id_area.x + self.cursor_position + 1,
                cent_id_area.y + 1,
            ));
        })?;

        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char(c) => {
                    self.id.insert(self.cursor_position.into(), c);
                    self.cursor_position += 1;
                }
                KeyCode::Esc => self.next_state = AppState::Exit,
                KeyCode::Left => self.cursor_position = self.cursor_position.saturating_sub(1),
                KeyCode::Right => {
                    self.cursor_position = self
                        .cursor_position
                        .saturating_add(1)
                        .min(self.id.len() as u16)
                }
                KeyCode::Backspace => {
                    if self.cursor_position != 0 {
                        self.cursor_position -= 1;
                        self.id.remove(self.cursor_position.into());
                    }
                }
                KeyCode::Delete => {
                    if self.cursor_position != self.id.len() as u16 {
                        self.id.remove(self.cursor_position.into());
                    }
                }
                KeyCode::Enter => self.find_item(db)?,
                _ => {}
            }
        }

        Ok(())
    }
    fn get_next_state(&self) -> AppState {
        self.next_state.clone()
    }
}
#[cfg(test)]
mod item_lookup_tests {
    use super::*;
    #[test]
    fn test_creation() {
        let my_applet = ItemLookupApplet::new();
        assert_eq!(my_applet.next_state, AppState::NoChange);
        assert_eq!(my_applet.cursor_position, 0);
        assert!(my_applet.id.is_empty());
    }

    #[test]
    fn test_parsing() {
        let my_inv = inventory::Inventory::open_in_memory().unwrap();
        fill_db(&my_inv);

        let mut my_applet = ItemLookupApplet::new();

        my_applet.id = "text".into();
        assert!(my_applet.find_item(&my_inv).is_err());

        my_applet.id = "2".into();
        assert!(my_applet.find_item(&my_inv).is_err());

        my_applet.id = "".into();
        assert!(my_applet.find_item(&my_inv).is_err());

        my_applet.id = "101".into();
        assert!(my_applet.find_item(&my_inv).is_ok());
        assert_eq!(my_applet.next_state, AppState::EditItem(101));
    }

    fn fill_db(my_inv: &inventory::Inventory) {
        for i in 0..5 {
            let item = inventory::Item {
                id: i + 100,
                name: format!("item{i}").to_string(),
                comment: Some(format!("comment{i}").to_string()),
                location_id: None,
            };
            assert!(my_inv.add_item(&item).is_ok());
        }
    }
}
