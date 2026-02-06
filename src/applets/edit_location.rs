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

pub struct EditLocationApplet {
    next_state: AppState,
    loc: inventory::Location,
    id: i64,
    cursor_position: u16,
    selection: EditLocationSelection,
}

#[derive(Debug)]
struct EditLocationError {
    error_text: String,
}

impl fmt::Display for EditLocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Edit Item Error: {}", self.error_text)
    }
}

impl error::Error for EditLocationError {}

impl EditLocationError {
    fn new(msg: &str) -> Box<EditLocationError> {
        Box::new(EditLocationError {
            error_text: msg.to_string(),
        })
    }
}

#[derive(PartialEq, Debug)]
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

impl Default for EditLocationApplet {
    fn default() -> Self {
        Self {
            next_state: AppState::NoChange,
            loc: inventory::Location {
                id: 260126,
                name: "Tweezers".to_string(),
                comment: Some("ESD Safe".to_string()),
            },
            id: -1,
            cursor_position: 0,
            selection: EditLocationSelection::Name,
        }
    }
}

impl EditLocationApplet {
    pub fn new(id: i64) -> Self {
        Self {
            next_state: AppState::NoChange,
            loc: inventory::Location {
                id: -1,
                name: "".to_string(),
                comment: None,
            },
            id,
            cursor_position: 0,
            selection: EditLocationSelection::Name,
        }
    }
    fn save_location(
        &mut self,
        db: &inventory::Inventory,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.loc.name.is_empty() {
            return Err(EditLocationError::new("Name cannot be empty"));
        }
        if self.loc.comment.clone().unwrap_or_default().is_empty() {
            self.loc.comment = None;
        }
        db.edit_location(&self.loc)?;
        Ok(())
    }
}
impl Applet for EditLocationApplet {
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
        ]);
        let border = Block::bordered()
            .title_top("Inventory Manager - Edit Location")
            .title_bottom("Press 'q' or Esc to exit")
            .border_type(ratatui::widgets::BorderType::Thick)
            .padding(Padding::horizontal(1));
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
            let inner_area = border.inner(frame.area());
            let [id_area, name_area, comment_area, cancel_area, save_area] =
                vertical.areas(inner_area);
            frame.render_widget(border, frame.area());
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
                    KeyCode::Delete => {
                        if self.cursor_position != self.loc.name.len() as u16 {
                            self.loc.name.remove(self.cursor_position.into());
                        }
                    }
                    KeyCode::Left => self.cursor_position = self.cursor_position.saturating_sub(1),
                    KeyCode::Right => {
                        self.cursor_position = self
                            .cursor_position
                            .saturating_add(1)
                            .min(self.loc.name.len() as u16)
                    }
                    _ => {}
                },

                EditLocationSelection::Comment => match key.code {
                    KeyCode::Char(c) => {
                        let mut comment = self.loc.comment.clone().unwrap_or_default();
                        comment.insert(self.cursor_position.into(), c);
                        self.loc.comment = Some(comment);
                        self.cursor_position += 1
                    }
                    KeyCode::Backspace => {
                        if self.cursor_position != 0 {
                            self.cursor_position -= 1;
                            let mut comment = self.loc.comment.clone().unwrap_or_default();
                            comment.remove(self.cursor_position.into());
                            self.loc.comment = Some(comment);
                        }
                    }
                    KeyCode::Delete => {
                        if self.cursor_position != self.loc.comment.as_ref().unwrap().len() as u16 {
                            let mut comment = self.loc.comment.clone().unwrap_or_default();
                            comment.remove(self.cursor_position.into());
                            self.loc.comment = Some(comment);
                        }
                    }
                    KeyCode::Left => self.cursor_position = self.cursor_position.saturating_sub(1),
                    KeyCode::Right => {
                        self.cursor_position = self
                            .cursor_position
                            .saturating_add(1)
                            .min(self.loc.comment.as_ref().unwrap().len() as u16)
                    }
                    _ => {}
                },
                EditLocationSelection::Cancel => match key.code {
                    KeyCode::Enter => self.next_state = AppState::Exit,
                    _ => (),
                },
                EditLocationSelection::Save => match key.code {
                    KeyCode::Enter => {
                        self.save_location(db)?;
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
        if self.loc.id != self.id {
            self.loc = db.search_location_id(self.id).unwrap(); //shouldn't be none, as we shouldn't get here if it doesn't exist
            if self.loc.comment.is_none() {
                self.loc.comment = Some(String::new());
            }
        }
    }
}

#[cfg(test)]
mod edit_location_tests {
    use super::*;
    #[test]
    fn test_creation() {
        let my_applet = EditLocationApplet::new(1);
        assert_eq!(my_applet.next_state, AppState::NoChange);
        assert_eq!(my_applet.loc.id, -1);
        assert_eq!(my_applet.loc.name, "".to_string());
        assert_eq!(my_applet.loc.comment, None);
        assert_eq!(my_applet.id, 1);
        assert_eq!(my_applet.cursor_position, 0);
        assert_eq!(my_applet.selection, EditLocationSelection::Name);
    }

    #[test]
    fn test_refresh() {
        let my_inv = inventory::Inventory::open_in_memory().unwrap();
        fill_db(&my_inv);

        let mut my_applet = EditLocationApplet::new(1);
        my_applet.refresh(&my_inv);

        assert_eq!(
            my_applet.loc,
            inventory::Location {
                id: 1,
                name: "location1".into(),
                comment: Some("comment1".into())
            }
        );

        my_applet.loc.comment = None;
        assert_eq!(
            my_applet.loc,
            inventory::Location {
                id: 1,
                name: "location1".into(),
                comment: None
            }
        );
        my_applet.refresh(&my_inv);
        assert_eq!(
            my_applet.loc,
            inventory::Location {
                id: 1,
                name: "location1".into(),
                comment: None
            }
        );
    }

    #[test]
    fn test_save_parsing() {
        let my_inv = inventory::Inventory::open_in_memory().unwrap();
        fill_db(&my_inv);
        let mut my_applet = EditLocationApplet::new(1);
        my_applet.refresh(&my_inv);

        my_applet.loc.name = "".to_string();
        assert!(my_applet.save_location(&my_inv).is_err());
        my_applet.loc.name = "Something else".to_string();
        assert!(my_applet.save_location(&my_inv).is_ok());
        my_applet.loc.comment = None;
        assert!(my_applet.save_location(&my_inv).is_ok());
        my_applet.loc.comment = Some("Other Comment".into());
        assert!(my_applet.save_location(&my_inv).is_ok());
    }

    #[test]
    fn test_save() {
        let my_inv = inventory::Inventory::open_in_memory().unwrap();
        fill_db(&my_inv);
        let mut my_applet = EditLocationApplet::new(1);
        my_applet.refresh(&my_inv);
        assert_eq!(
            my_inv.search_location_id(my_applet.loc.id),
            Some(inventory::Location {
                id: 1,
                name: "location1".to_string(),
                comment: Some("comment1".to_string()),
            })
        );

        my_applet.loc.name = "newname".into();
        assert!(my_applet.save_location(&my_inv).is_ok());
        assert_eq!(
            my_inv.search_location_id(my_applet.loc.id),
            Some(inventory::Location {
                id: 1,
                name: "newname".to_string(),
                comment: Some("comment1".to_string()),
            })
        );

        my_applet.loc.name = "".into();
        assert!(my_applet.save_location(&my_inv).is_err());
        assert_eq!(
            my_inv.search_location_id(my_applet.loc.id),
            Some(inventory::Location {
                id: 1,
                name: "newname".to_string(),
                comment: Some("comment1".to_string()),
            })
        );
        my_applet.loc.name = "newername".into();
        assert!(my_applet.save_location(&my_inv).is_ok());
        assert_eq!(
            my_inv.search_location_id(my_applet.loc.id),
            Some(inventory::Location {
                id: 1,
                name: "newername".to_string(),
                comment: Some("comment1".to_string()),
            })
        );

        my_applet.loc.comment = Some("".into());
        assert!(my_applet.save_location(&my_inv).is_ok());
        assert_eq!(
            my_inv.search_location_id(my_applet.loc.id),
            Some(inventory::Location {
                id: 1,
                name: "newername".to_string(),
                comment: None,
            })
        );

        my_applet.loc.comment = Some("Other Comment".into());
        assert!(my_applet.save_location(&my_inv).is_ok());
        assert_eq!(
            my_inv.search_location_id(my_applet.loc.id),
            Some(inventory::Location {
                id: 1,
                name: "newername".to_string(),
                comment: Some("Other Comment".into()),
            })
        );

        my_applet.loc.comment = None;
        assert!(my_applet.save_location(&my_inv).is_ok());
        assert_eq!(
            my_inv.search_location_id(my_applet.loc.id),
            Some(inventory::Location {
                id: 1,
                name: "newername".to_string(),
                comment: None,
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
            assert!(my_inv.add_location(&loc).is_ok());
        }
    }
}
