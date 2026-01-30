use super::applet::Applet;
use crate::AppState;
use crate::db::inventory;
use crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::Style;
use ratatui::widgets::{Block, Padding, Paragraph};
use std::error;
use std::fmt;

pub struct CreateItemApplet {
    next_state: AppState,
    cursor_position: u16,
    selection: CreateItemSelection,
    id: String,
    name: String,
    comment: String,
    location_id: String,
}
#[derive(Debug)]
struct CreateItemError {
    error_text: String,
}

impl fmt::Display for CreateItemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Create Item Error: {}", self.error_text)
    }
}

impl error::Error for CreateItemError {}

impl CreateItemError {
    fn new(msg: &str) -> Box<CreateItemError> {
        Box::new(CreateItemError {
            error_text: msg.to_string(),
        })
    }
}

#[derive(Debug, PartialEq)]
enum CreateItemSelection {
    Id,
    Name,
    Comment,
    LocationID,
    Cancel,
    Save,
}

impl CreateItemSelection {
    fn next(&self) -> Self {
        match self {
            CreateItemSelection::Id => CreateItemSelection::Name,
            CreateItemSelection::Name => CreateItemSelection::Comment,
            CreateItemSelection::Comment => CreateItemSelection::LocationID,
            CreateItemSelection::LocationID => CreateItemSelection::Cancel,
            CreateItemSelection::Cancel => CreateItemSelection::Save,
            CreateItemSelection::Save => CreateItemSelection::Id,
        }
    }
    fn previous(&self) -> Self {
        match self {
            CreateItemSelection::Id => CreateItemSelection::Save,
            CreateItemSelection::Name => CreateItemSelection::Id,
            CreateItemSelection::Comment => CreateItemSelection::Name,
            CreateItemSelection::LocationID => CreateItemSelection::Comment,
            CreateItemSelection::Cancel => CreateItemSelection::LocationID,
            CreateItemSelection::Save => CreateItemSelection::Cancel,
        }
    }
}

impl CreateItemApplet {
    pub fn new() -> Self {
        Self {
            next_state: AppState::NoChange,
            cursor_position: 0,
            selection: CreateItemSelection::Id,
            id: String::default(),
            name: String::default(),
            comment: String::default(),
            location_id: String::default(),
        }
    }
    fn save_item(&mut self, db: &inventory::Inventory) -> Result<(), Box<dyn std::error::Error>> {
        //Check ID
        let id = self
            .id
            .parse::<i64>()
            .map_err(|_| CreateItemError::new("Failed to parse Item ID"))?;
        if db.item_exists(id) {
            return Err(CreateItemError::new("Item ID already exists"));
        }

        //check Name
        if self.name.is_empty() {
            return Err(CreateItemError::new("Name cannot be empty"));
        }

        //check comment
        let comment_opt = if self.comment.is_empty() {
            None
        } else {
            Some(self.comment.clone())
        };

        //check locationid
        let lid_opt = if self.location_id.is_empty() {
            None
        } else {
            let lid = self
                .location_id
                .parse::<i64>()
                .map_err(|_| CreateItemError::new("Failed to parse Location ID"))?;
            if !db.location_exists(lid) {
                return Err(CreateItemError::new("Location ID does not exist"));
            }
            Some(lid)
        };
        let new_item = inventory::Item {
            id,
            name: self.name.clone(),
            comment: comment_opt,
            location_id: lid_opt,
        };
        db.add_item(&new_item)?;
        Ok(())
    }
}

impl Applet for CreateItemApplet {
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
        let border = Block::bordered()
            .title_top("Inventory Manager - Create Item")
            .title_bottom("Press 'q' or Esc to exit")
            .border_type(ratatui::widgets::BorderType::Thick)
            .padding(Padding::horizontal(1));
        let id_widget = Paragraph::new(self.id.clone())
            .style(if self.selection == CreateItemSelection::Id {
                Style::default().yellow()
            } else {
                Style::default()
            })
            .block(Block::bordered().title("Item ID"));
        let name_widget = Paragraph::new(self.name.clone())
            .style(if self.selection == CreateItemSelection::Name {
                Style::default().yellow()
            } else {
                Style::default()
            })
            .block(Block::bordered().title("Name"));
        let comment_widget = Paragraph::new(self.comment.clone())
            .style(if self.selection == CreateItemSelection::Comment {
                Style::default().yellow()
            } else {
                Style::default()
            })
            .block(Block::bordered().title("Comment"));
        let location_widget = Paragraph::new(self.location_id.clone())
            .style(if self.selection == CreateItemSelection::LocationID {
                Style::default().yellow()
            } else {
                Style::default()
            })
            .block(Block::bordered().title("Location ID"));
        let cancel_button = Paragraph::new("Cancel".to_string())
            .style(if self.selection == CreateItemSelection::Cancel {
                Style::default().yellow().bold()
            } else {
                Style::default().bold()
            })
            .block(Block::bordered());
        let save_button = Paragraph::new("Save Changes".to_string())
            .style(if self.selection == CreateItemSelection::Save {
                Style::default().yellow().bold()
            } else {
                Style::default().bold()
            })
            .block(Block::bordered());

        terminal.draw(|frame| {
            let inner_area = border.inner(frame.area());
            let [
                id_area,
                name_area,
                comment_area,
                location_area,
                cancel_area,
                save_area,
            ] = vertical.areas(inner_area);
            frame.render_widget(border, frame.area());
            frame.render_widget(id_widget, id_area);
            frame.render_widget(name_widget, name_area);
            frame.render_widget(comment_widget, comment_area);
            frame.render_widget(location_widget, location_area);
            frame.render_widget(cancel_button, cancel_area);
            frame.render_widget(save_button, save_area);
            match self.selection {
                CreateItemSelection::Id => {
                    frame.set_cursor_position(Position::new(
                        id_area.x + self.cursor_position + 1,
                        id_area.y + 1,
                    ));
                }
                CreateItemSelection::Name => {
                    frame.set_cursor_position(Position::new(
                        name_area.x + self.cursor_position + 1,
                        name_area.y + 1,
                    ));
                }
                CreateItemSelection::Comment => {
                    frame.set_cursor_position(Position::new(
                        comment_area.x + self.cursor_position + 1,
                        comment_area.y + 1,
                    ));
                }
                CreateItemSelection::LocationID => {
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
                CreateItemSelection::Id => match key.code {
                    KeyCode::Char(c) => {
                        self.id.insert(self.cursor_position.into(), c);
                        self.cursor_position += 1
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
                    KeyCode::Left => self.cursor_position = self.cursor_position.saturating_sub(1),
                    KeyCode::Right => {
                        self.cursor_position = self
                            .cursor_position
                            .saturating_add(1)
                            .min(self.id.len() as u16)
                    }
                    _ => {}
                },
                CreateItemSelection::Name => match key.code {
                    KeyCode::Char(c) => {
                        self.name.insert(self.cursor_position.into(), c);
                        self.cursor_position += 1
                    }
                    KeyCode::Backspace => {
                        if self.cursor_position != 0 {
                            self.cursor_position -= 1;
                            self.name.remove(self.cursor_position.into());
                        }
                    }
                    KeyCode::Delete => {
                        if self.cursor_position != self.name.len() as u16 {
                            self.name.remove(self.cursor_position.into());
                        }
                    }
                    KeyCode::Left => self.cursor_position = self.cursor_position.saturating_sub(1),
                    KeyCode::Right => {
                        self.cursor_position = self
                            .cursor_position
                            .saturating_add(1)
                            .min(self.name.len() as u16)
                    }
                    _ => {}
                },

                CreateItemSelection::Comment => match key.code {
                    KeyCode::Char(c) => {
                        self.comment.insert(self.cursor_position.into(), c);
                        self.cursor_position += 1
                    }
                    KeyCode::Backspace => {
                        if self.cursor_position != 0 {
                            self.cursor_position -= 1;
                            self.comment.remove(self.cursor_position.into());
                        }
                    }
                    KeyCode::Delete => {
                        if self.cursor_position != self.comment.len() as u16 {
                            self.comment.remove(self.cursor_position.into());
                        }
                    }
                    KeyCode::Left => self.cursor_position = self.cursor_position.saturating_sub(1),
                    KeyCode::Right => {
                        self.cursor_position = self
                            .cursor_position
                            .saturating_add(1)
                            .min(self.comment.len() as u16)
                    }
                    _ => {}
                },
                CreateItemSelection::LocationID => match key.code {
                    KeyCode::Char(c) => {
                        self.location_id.insert(self.cursor_position.into(), c);
                        self.cursor_position += 1
                    }
                    KeyCode::Backspace => {
                        if self.cursor_position != 0 {
                            self.cursor_position -= 1;
                            self.location_id.remove(self.cursor_position.into());
                        }
                    }
                    KeyCode::Delete => {
                        if self.cursor_position != self.location_id.len() as u16 {
                            self.location_id.remove(self.cursor_position.into());
                        }
                    }
                    KeyCode::Left => self.cursor_position = self.cursor_position.saturating_sub(1),
                    KeyCode::Right => {
                        self.cursor_position = self
                            .cursor_position
                            .saturating_add(1)
                            .min(self.location_id.len() as u16)
                    }
                    _ => {}
                },
                CreateItemSelection::Cancel => match key.code {
                    KeyCode::Enter => self.next_state = AppState::Exit,
                    _ => (),
                },
                CreateItemSelection::Save => match key.code {
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
}

#[cfg(test)]
mod create_item_tests {
    use super::*;
    #[test]
    fn test_new() {
        let my_applet = CreateItemApplet::new();
        assert_eq!(my_applet.next_state, AppState::NoChange);
        assert!(my_applet.id.is_empty());
        assert!(my_applet.name.is_empty());
        assert!(my_applet.comment.is_empty());
        assert!(my_applet.location_id.is_empty());
        assert_eq!(my_applet.cursor_position, 0);
        assert_eq!(my_applet.selection, CreateItemSelection::Id);
    }

    #[test]
    fn test_save_parsing() {
        let my_inv = inventory::Inventory::open_in_memory().unwrap();
        fill_db(&my_inv);

        let mut my_applet = CreateItemApplet::new();

        my_applet.name = "Some_name".into();
        my_applet.id = "1".into();
        assert!(my_applet.save_item(&my_inv).is_ok());
        my_applet.id = "nan".into();
        assert!(my_applet.save_item(&my_inv).is_err());
        my_applet.id = "0xff".into();
        assert!(my_applet.save_item(&my_inv).is_err());
        my_applet.id = "101".into();
        assert!(my_applet.save_item(&my_inv).is_err());
        my_applet.id = "2".into();
        assert!(my_applet.save_item(&my_inv).is_ok());

        my_applet.id = "3".into();
        my_applet.name = "".into();
        assert!(my_applet.save_item(&my_inv).is_err());
        my_applet.name = "43".into();
        assert!(my_applet.save_item(&my_inv).is_ok());

        my_applet.id = "4".into();
        my_applet.location_id = "nan".into();
        assert!(my_applet.save_item(&my_inv).is_err());
        my_applet.location_id = "0x02".into();
        assert!(my_applet.save_item(&my_inv).is_err());
        my_applet.location_id = "101".into();
        assert!(my_applet.save_item(&my_inv).is_err());
        my_applet.location_id = "2".into();
        assert!(my_applet.save_item(&my_inv).is_ok());
    }

    #[test]
    fn test_save() {
        let my_inv = inventory::Inventory::open_in_memory().unwrap();
        fill_db(&my_inv);
        let mut my_applet = CreateItemApplet::new();
        my_applet.id = "201".into();
        my_applet.name = "n".into();
        my_applet.comment = "".into();
        my_applet.location_id = "".into();

        assert!(my_applet.save_item(&my_inv).is_ok());
        assert_eq!(
            my_inv.search_item_id(201),
            Some(inventory::Item {
                id: 201,
                name: "n".into(),
                comment: None,
                location_id: None
            })
        );

        my_applet.id = "202".into();
        my_applet.name = "n".into();
        my_applet.comment = "some_comment".into();
        my_applet.location_id = "".into();

        assert!(my_applet.save_item(&my_inv).is_ok());
        assert_eq!(
            my_inv.search_item_id(202),
            Some(inventory::Item {
                id: 202,
                name: "n".into(),
                comment: Some("some_comment".into()),
                location_id: None
            })
        );

        my_applet.id = "203".into();
        my_applet.name = "n".into();
        my_applet.comment = "some_comment".into();
        my_applet.location_id = "2".into();

        assert!(my_applet.save_item(&my_inv).is_ok());
        assert_eq!(
            my_inv.search_item_id(203),
            Some(inventory::Item {
                id: 203,
                name: "n".into(),
                comment: Some("some_comment".into()),
                location_id: Some(2)
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
