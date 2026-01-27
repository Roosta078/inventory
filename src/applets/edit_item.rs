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

pub struct EditItemApplet {
    next_state: AppState,
    item: inventory::Item,
    id: i64,
    cursor_position: u16,
    selection: EditItemSelection,
    loc_id_str: String,
}
#[derive(Debug)]
struct EditItemError {
    error_text: String,
}

impl fmt::Display for EditItemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Edit Item Error: {}", self.error_text)
    }
}

impl error::Error for EditItemError {}

impl EditItemError {
    fn new(msg: &str) -> Box<EditItemError> {
        Box::new(EditItemError {
            error_text: msg.to_string(),
        })
    }
}

#[derive(Debug, PartialEq)]
enum EditItemSelection {
    Name,
    Comment,
    LocationID,
    Cancel,
    Save,
}

impl EditItemSelection {
    fn next(&self) -> Self {
        match self {
            EditItemSelection::Name => EditItemSelection::Comment,
            EditItemSelection::Comment => EditItemSelection::LocationID,
            EditItemSelection::LocationID => EditItemSelection::Cancel,
            EditItemSelection::Cancel => EditItemSelection::Save,
            EditItemSelection::Save => EditItemSelection::Name,
        }
    }
    fn previous(&self) -> Self {
        match self {
            EditItemSelection::Name => EditItemSelection::Save,
            EditItemSelection::Comment => EditItemSelection::Name,
            EditItemSelection::LocationID => EditItemSelection::Comment,
            EditItemSelection::Cancel => EditItemSelection::LocationID,
            EditItemSelection::Save => EditItemSelection::Cancel,
        }
    }
}

impl EditItemApplet {
    pub fn new(id: i64) -> Self {
        Self {
            next_state: AppState::NoChange,
            item: inventory::Item {
                id: -1,
                name: "".to_string(),
                comment: None,
                location_id: None,
            },
            id,
            cursor_position: 0,
            selection: EditItemSelection::Name,
            loc_id_str: String::default(),
        }
    }
    fn save_item(&mut self, db: &inventory::Inventory) -> Result<(), Box<dyn std::error::Error>> {
        if self.loc_id_str.is_empty() {
            self.item.location_id = None;
        } else {
            match self.loc_id_str.parse::<i64>() {
                Ok(id) => self.item.location_id = Some(id),
                Err(_) => return Err(EditItemError::new("Could not parse Location ID")),
            }
            if !db.location_exists(self.item.location_id.unwrap()) {
                return Err(EditItemError::new("Location ID not found in Database"));
            }
        }
        if self.item.comment.clone().unwrap_or_default().is_empty() {
            self.item.comment = None;
        }

        if self.item.name.is_empty() {
            return Err(EditItemError::new("Name cannot be empty"));
        }

        if db.edit_item(&self.item).is_err() {
            return Err(EditItemError::new("Failed to write update to Database"));
        }

        Ok(())
    }
}

impl Applet for EditItemApplet {
    fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        db: &inventory::Inventory,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.next_state = AppState::NoChange;

        //Prepare Draw
        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ]);
        let id_widget = Paragraph::new(self.item.id.to_string())
            .style(Style::default().bold())
            .block(Block::bordered().title("Item ID"));
        let name_widget = Paragraph::new(self.item.name.clone())
            .style(if self.selection == EditItemSelection::Name {
                Style::default().yellow()
            } else {
                Style::default()
            })
            .block(Block::bordered().title("Name"));
        let comment_widget = Paragraph::new(self.item.comment.clone().unwrap_or("".to_string()))
            .style(if self.selection == EditItemSelection::Comment {
                Style::default().yellow()
            } else {
                Style::default()
            })
            .block(Block::bordered().title("Comment"));
        let location_widget = Paragraph::new(self.loc_id_str.clone())
            .style(if self.selection == EditItemSelection::LocationID {
                Style::default().yellow()
            } else {
                Style::default()
            })
            .block(Block::bordered().title("Location ID"));
        let cancel_button = Paragraph::new("Cancel".to_string())
            .style(if self.selection == EditItemSelection::Cancel {
                Style::default().yellow().bold()
            } else {
                Style::default().bold()
            })
            .block(Block::bordered());
        let save_button = Paragraph::new("Save Changes".to_string())
            .style(if self.selection == EditItemSelection::Save {
                Style::default().yellow().bold()
            } else {
                Style::default().bold()
            })
            .block(Block::bordered());

        terminal.draw(|frame| {
            let [
                id_area,
                name_area,
                comment_area,
                location_area,
                cancel_area,
                save_area,
            ] = vertical.areas(frame.area());
            frame.render_widget(id_widget, id_area);
            frame.render_widget(name_widget, name_area);
            frame.render_widget(comment_widget, comment_area);
            frame.render_widget(location_widget, location_area);
            frame.render_widget(cancel_button, cancel_area);
            frame.render_widget(save_button, save_area);
            match self.selection {
                EditItemSelection::Name => {
                    frame.set_cursor_position(Position::new(
                        name_area.x + self.cursor_position + 1,
                        name_area.y + 1,
                    ));
                }
                EditItemSelection::Comment => {
                    frame.set_cursor_position(Position::new(
                        comment_area.x + self.cursor_position + 1,
                        comment_area.y + 1,
                    ));
                }
                EditItemSelection::LocationID => {
                    frame.set_cursor_position(Position::new(
                        location_area.x + self.cursor_position + 1,
                        location_area.y + 1,
                    ));
                }
                _ => (),
            }
        })?;

        //Handle Input
        if let Some(key) = event::read()?.as_key_press_event() {
            match self.selection {
                EditItemSelection::Name => match key.code {
                    KeyCode::Char(c) => {
                        self.item.name.insert(self.cursor_position.into(), c);
                        self.cursor_position += 1
                    }
                    KeyCode::Backspace => {
                        if self.cursor_position != 0 {
                            self.cursor_position -= 1;
                            self.item.name.remove(self.cursor_position.into());
                        }
                    }
                    KeyCode::Delete => {
                        if self.cursor_position != self.item.name.len() as u16 {
                            self.item.name.remove(self.cursor_position.into());
                        }
                    }
                    KeyCode::Left => self.cursor_position = self.cursor_position.saturating_sub(1),
                    KeyCode::Right => {
                        self.cursor_position = self
                            .cursor_position
                            .saturating_add(1)
                            .min(self.item.name.len() as u16)
                    }
                    _ => {}
                },

                EditItemSelection::Comment => match key.code {
                    KeyCode::Char(c) => {
                        let mut comment = self.item.comment.clone().unwrap_or_default();
                        comment.insert(self.cursor_position.into(), c);
                        self.item.comment = Some(comment);
                        self.cursor_position += 1
                    }
                    KeyCode::Backspace => {
                        if self.cursor_position != 0 {
                            self.cursor_position -= 1;
                            let mut comment = self.item.comment.clone().unwrap_or_default();
                            comment.remove(self.cursor_position.into());
                            self.item.comment = Some(comment);
                        }
                    }
                    KeyCode::Delete => {
                        if self.cursor_position != self.item.comment.as_ref().unwrap().len() as u16
                        {
                            let mut comment = self.item.comment.clone().unwrap_or_default();
                            comment.remove(self.cursor_position.into());
                            self.item.comment = Some(comment);
                        }
                    }
                    KeyCode::Left => self.cursor_position = self.cursor_position.saturating_sub(1),
                    KeyCode::Right => {
                        self.cursor_position = self
                            .cursor_position
                            .saturating_add(1)
                            .min(self.item.comment.as_ref().unwrap().len() as u16)
                    }
                    _ => {}
                },
                EditItemSelection::LocationID => match key.code {
                    KeyCode::Char(c) => {
                        self.loc_id_str.insert(self.cursor_position.into(), c);
                        self.cursor_position += 1
                    }
                    KeyCode::Backspace => {
                        if self.cursor_position != 0 {
                            self.cursor_position -= 1;
                            self.loc_id_str.remove(self.cursor_position.into());
                        }
                    }
                    KeyCode::Delete => {
                        if self.cursor_position != self.loc_id_str.len() as u16 {
                            self.loc_id_str.remove(self.cursor_position.into());
                        }
                    }
                    KeyCode::Left => self.cursor_position = self.cursor_position.saturating_sub(1),
                    KeyCode::Right => {
                        self.cursor_position = self
                            .cursor_position
                            .saturating_add(1)
                            .min(self.loc_id_str.len() as u16)
                    }
                    _ => {}
                },
                EditItemSelection::Cancel => match key.code {
                    KeyCode::Enter => self.next_state = AppState::Exit,
                    _ => (),
                },
                EditItemSelection::Save => match key.code {
                    KeyCode::Enter => {
                        self.save_item(db)?;
                        self.next_state = AppState::Exit
                    }
                    _ => (),
                },
            }

            match key.code {
                KeyCode::Esc => self.next_state = AppState::Exit,
                KeyCode::Down | KeyCode::Tab => {
                    self.selection = self.selection.next();
                    self.cursor_position = 0
                }
                KeyCode::Up => {
                    self.selection = self.selection.previous();
                    self.cursor_position = 0
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn get_next_state(&self) -> AppState {
        self.next_state.clone()
    }

    fn refresh(&mut self, db: &inventory::Inventory) {
        //check if we need to load
        if self.item.id != self.id {
            self.item = db.search_item_id(self.id).unwrap(); //shouldn't be none, as we shouldn't get here if it doesn't exist
            if self.item.comment.is_none() {
                self.item.comment = Some("".to_string());
            }
            self.loc_id_str = self
                .item
                .location_id
                .map(|lid| lid.to_string())
                .unwrap_or("".to_string())
        }
    }
}

#[cfg(test)]
mod edit_item_tests {
    use super::*;
    #[test]
    fn test_creation() {
        let my_applet = EditItemApplet::new(1);
        assert_eq!(my_applet.next_state, AppState::NoChange);
        assert_eq!(my_applet.item.id, -1);
        assert_eq!(my_applet.item.name, "".to_string());
        assert_eq!(my_applet.item.comment, None);
        assert_eq!(my_applet.item.location_id, None);
        assert_eq!(my_applet.id, 1);
        assert_eq!(my_applet.cursor_position, 0);
        assert_eq!(my_applet.selection, EditItemSelection::Name);
        assert_eq!(my_applet.loc_id_str, "".to_string());
    }

    #[test]
    fn test_refresh() {
        let my_inv = inventory::Inventory::open_in_memory().unwrap();
        fill_db(&my_inv);

        let mut my_applet = EditItemApplet::new(101);
        my_applet.refresh(&my_inv);
        assert_eq!(
            my_applet.item,
            inventory::Item {
                id: 101,
                name: "item1".to_string(),
                comment: Some("comment1".to_string()),
                location_id: Some(1)
            }
        );
        my_applet.item.location_id = None;
        assert_eq!(
            my_applet.item,
            inventory::Item {
                id: 101,
                name: "item1".to_string(),
                comment: Some("comment1".to_string()),
                location_id: None,
            }
        );
        my_applet.refresh(&my_inv);
        assert_eq!(
            my_applet.item,
            inventory::Item {
                id: 101,
                name: "item1".to_string(),
                comment: Some("comment1".to_string()),
                location_id: None,
            }
        );
    }

    #[test]
    fn test_save_parsing() {
        let my_inv = inventory::Inventory::open_in_memory().unwrap();
        fill_db(&my_inv);
        let mut my_applet = EditItemApplet::new(101);
        my_applet.refresh(&my_inv);

        my_applet.loc_id_str = "nan".to_string();
        assert!(my_applet.save_item(&my_inv).is_err());
        my_applet.loc_id_str = "0xff".to_string();
        assert!(my_applet.save_item(&my_inv).is_err());
        my_applet.loc_id_str = "99".to_string();
        assert!(my_applet.save_item(&my_inv).is_err());
        my_applet.loc_id_str = "4".to_string();
        assert!(my_applet.save_item(&my_inv).is_ok());

        my_applet.item.name = "".to_string();
        assert!(my_applet.save_item(&my_inv).is_err());
    }

    #[test]
    fn test_save() {
        let my_inv = inventory::Inventory::open_in_memory().unwrap();
        fill_db(&my_inv);
        let mut my_applet = EditItemApplet::new(101);
        my_applet.refresh(&my_inv);
        assert_eq!(
            my_inv.search_item_id(my_applet.item.id),
            Some(inventory::Item {
                id: 101,
                name: "item1".to_string(),
                comment: Some("comment1".to_string()),
                location_id: Some(1)
            })
        );

        my_applet.loc_id_str = "nan".to_string();
        let _ = my_applet.save_item(&my_inv);
        assert_eq!(
            my_inv.search_item_id(my_applet.item.id),
            Some(inventory::Item {
                id: 101,
                name: "item1".to_string(),
                comment: Some("comment1".to_string()),
                location_id: Some(1)
            })
        );

        my_applet.loc_id_str = "".to_string();
        let _ = my_applet.save_item(&my_inv);
        assert_eq!(
            my_inv.search_item_id(my_applet.item.id),
            Some(inventory::Item {
                id: 101,
                name: "item1".to_string(),
                comment: Some("comment1".to_string()),
                location_id: None
            })
        );

        my_applet.loc_id_str = "4".to_string();
        let _ = my_applet.save_item(&my_inv);
        assert_eq!(
            my_inv.search_item_id(my_applet.item.id),
            Some(inventory::Item {
                id: 101,
                name: "item1".to_string(),
                comment: Some("comment1".to_string()),
                location_id: Some(4)
            })
        );
        my_applet.item.name = "".to_string();
        let _ = my_applet.save_item(&my_inv);
        assert_eq!(
            my_inv.search_item_id(my_applet.item.id),
            Some(inventory::Item {
                id: 101,
                name: "item1".to_string(),
                comment: Some("comment1".to_string()),
                location_id: Some(4)
            })
        );

        my_applet.item.name = "newname".to_string();
        let _ = my_applet.save_item(&my_inv);
        assert_eq!(
            my_inv.search_item_id(my_applet.item.id),
            Some(inventory::Item {
                id: 101,
                name: "newname".to_string(),
                comment: Some("comment1".to_string()),
                location_id: Some(4)
            })
        );

        my_applet.item.comment = Some("newcomment".to_string());
        let _ = my_applet.save_item(&my_inv);
        assert_eq!(
            my_inv.search_item_id(my_applet.item.id),
            Some(inventory::Item {
                id: 101,
                name: "newname".to_string(),
                comment: Some("newcomment".to_string()),
                location_id: Some(4)
            })
        );
        my_applet.item.comment = Some("".to_string());
        let _ = my_applet.save_item(&my_inv);
        assert_eq!(
            my_inv.search_item_id(my_applet.item.id),
            Some(inventory::Item {
                id: 101,
                name: "newname".to_string(),
                comment: None,
                location_id: Some(4)
            })
        );
    }

    fn fill_db(my_inv: &inventory::Inventory) {
        for i in 0..5 {
            let loc = inventory::Location {
                id: i,
                name: format!("location{i}").to_string(),
                comment: Some(format!("comment{i}").to_string()),
            };
            let item = inventory::Item {
                id: i + 100,
                name: format!("item{i}").to_string(),
                comment: Some(format!("comment{i}").to_string()),
                location_id: Some(i),
            };
            assert!(my_inv.add_location(&loc).is_ok());
            assert!(my_inv.add_item(&item).is_ok());
        }
    }
}
