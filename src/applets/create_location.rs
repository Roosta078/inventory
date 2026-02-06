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

pub struct CreateLocationApplet {
    next_state: AppState,
    id: String,
    name: String,
    comment: String,
    selection: CreateLocationSelection,
    cursor_position: usize,
}

#[derive(Debug)]
struct CreateLocationError {
    error_text: String,
}

impl fmt::Display for CreateLocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Create Item Error: {}", self.error_text)
    }
}

impl error::Error for CreateLocationError {}

impl CreateLocationError {
    fn new(msg: &str) -> Box<CreateLocationError> {
        Box::new(CreateLocationError {
            error_text: msg.to_string(),
        })
    }
}
#[derive(Debug, PartialEq)]
enum CreateLocationSelection {
    Id,
    Name,
    Comment,
    Cancel,
    Save,
}

impl CreateLocationSelection {
    fn next(&self) -> Self {
        match self {
            CreateLocationSelection::Id => CreateLocationSelection::Name,
            CreateLocationSelection::Name => CreateLocationSelection::Comment,
            CreateLocationSelection::Comment => CreateLocationSelection::Cancel,
            CreateLocationSelection::Cancel => CreateLocationSelection::Save,
            CreateLocationSelection::Save => CreateLocationSelection::Id,
        }
    }
    fn previous(&self) -> Self {
        match self {
            CreateLocationSelection::Id => CreateLocationSelection::Save,
            CreateLocationSelection::Name => CreateLocationSelection::Id,
            CreateLocationSelection::Comment => CreateLocationSelection::Name,
            CreateLocationSelection::Cancel => CreateLocationSelection::Comment,
            CreateLocationSelection::Save => CreateLocationSelection::Cancel,
        }
    }
}

impl CreateLocationApplet {
    pub fn new() -> Self {
        Self {
            next_state: AppState::NoChange,
            id: String::new(),
            name: String::new(),
            comment: String::new(),
            selection: CreateLocationSelection::Id,
            cursor_position: 0,
        }
    }
    fn save_location(&self, db: &inventory::Inventory) -> Result<(), Box<dyn std::error::Error>> {
        //Check ID
        let id = self
            .id
            .parse::<i64>()
            .map_err(|_| CreateLocationError::new("Failed to parse Location ID"))?;
        if db.item_exists(id) {
            return Err(CreateLocationError::new("Location ID already exists"));
        }

        //check Name
        if self.name.is_empty() {
            return Err(CreateLocationError::new("Name cannot be empty"));
        }

        //check comment
        let comment_opt = if self.comment.is_empty() {
            None
        } else {
            Some(self.comment.clone())
        };

        let new_location = inventory::Location {
            id,
            name: self.name.clone(),
            comment: comment_opt,
        };
        db.add_location(&new_location)?;
        Ok(())
    }
}

impl Applet for CreateLocationApplet {
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
            .title_top("Inventory Manager - Create Location")
            .title_bottom("Press 'q' or Esc to exit")
            .border_type(ratatui::widgets::BorderType::Thick)
            .padding(Padding::horizontal(1));
        let id_widget = Paragraph::new(self.id.as_str())
            .style(if self.selection == CreateLocationSelection::Id {
                Style::default().yellow()
            } else {
                Style::default()
            })
            .block(Block::bordered().title("Location ID"));
        let name_widget = Paragraph::new(self.name.as_str())
            .style(if self.selection == CreateLocationSelection::Name {
                Style::default().yellow()
            } else {
                Style::default()
            })
            .block(Block::bordered().title("Name"));
        let comment_widget = Paragraph::new(self.comment.as_str())
            .style(if self.selection == CreateLocationSelection::Comment {
                Style::default().yellow()
            } else {
                Style::default()
            })
            .block(Block::bordered().title("Comment"));
        let cancel_button = Paragraph::new("Cancel".to_string())
            .style(if self.selection == CreateLocationSelection::Cancel {
                Style::default().yellow()
            } else {
                Style::default()
            })
            .block(Block::bordered());
        let save_button = Paragraph::new("Save Changes".to_string())
            .style(if self.selection == CreateLocationSelection::Save {
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
                CreateLocationSelection::Id => {
                    frame.set_cursor_position(Position::new(
                        id_area.x + self.cursor_position as u16 + 1,
                        id_area.y + 1,
                    ));
                }
                CreateLocationSelection::Name => {
                    frame.set_cursor_position(Position::new(
                        name_area.x + self.cursor_position as u16 + 1,
                        name_area.y + 1,
                    ));
                }
                CreateLocationSelection::Comment => {
                    frame.set_cursor_position(Position::new(
                        comment_area.x + self.cursor_position as u16 + 1,
                        comment_area.y + 1,
                    ));
                }
                _ => (),
            }
        })?;

        //Handle Input
        if let Some(key) = event::read()?.as_key_press_event() {
            match self.selection {
                CreateLocationSelection::Id => match key.code {
                    KeyCode::Char(c) => {
                        self.id.insert(self.cursor_position, c);
                        self.cursor_position += 1
                    }
                    KeyCode::Backspace => {
                        if self.cursor_position != 0 {
                            self.cursor_position -= 1;
                            self.id.remove(self.cursor_position);
                        }
                    }
                    KeyCode::Delete => {
                        if self.cursor_position != self.id.len() {
                            self.id.remove(self.cursor_position);
                        }
                    }
                    KeyCode::Left => self.cursor_position = self.cursor_position.saturating_sub(1),
                    KeyCode::Right => {
                        self.cursor_position =
                            self.cursor_position.saturating_add(1).min(self.id.len())
                    }
                    _ => {}
                },
                CreateLocationSelection::Name => match key.code {
                    KeyCode::Char(c) => {
                        self.name.insert(self.cursor_position, c);
                        self.cursor_position += 1
                    }
                    KeyCode::Backspace => {
                        if self.cursor_position != 0 {
                            self.cursor_position -= 1;
                            self.name.remove(self.cursor_position);
                        }
                    }
                    KeyCode::Delete => {
                        if self.cursor_position != self.name.len() {
                            self.name.remove(self.cursor_position);
                        }
                    }
                    KeyCode::Left => self.cursor_position = self.cursor_position.saturating_sub(1),
                    KeyCode::Right => {
                        self.cursor_position =
                            self.cursor_position.saturating_add(1).min(self.name.len())
                    }
                    _ => {}
                },

                CreateLocationSelection::Comment => match key.code {
                    KeyCode::Char(c) => {
                        self.comment.insert(self.cursor_position, c);
                        self.cursor_position += 1
                    }
                    KeyCode::Backspace => {
                        if self.cursor_position != 0 {
                            self.cursor_position -= 1;
                            self.comment.remove(self.cursor_position);
                        }
                    }
                    KeyCode::Delete => {
                        if self.cursor_position != self.comment.len() {
                            self.comment.remove(self.cursor_position);
                        }
                    }
                    KeyCode::Left => self.cursor_position = self.cursor_position.saturating_sub(1),
                    KeyCode::Right => {
                        self.cursor_position = self
                            .cursor_position
                            .saturating_add(1)
                            .min(self.comment.len())
                    }
                    _ => {}
                },
                CreateLocationSelection::Cancel => match key.code {
                    KeyCode::Enter => self.next_state = AppState::Exit,
                    _ => (),
                },
                CreateLocationSelection::Save => match key.code {
                    KeyCode::Enter => {
                        self.next_state = if self.save_location(db).is_ok() {
                            AppState::Exit
                        } else {
                            AppState::NoChange
                        }
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
mod create_location_tests {
    use super::*;
    #[test]
    fn test_new() {
        let my_applet = CreateLocationApplet::new();
        assert_eq!(my_applet.next_state, AppState::NoChange);
        assert!(my_applet.id.is_empty());
        assert!(my_applet.name.is_empty());
        assert!(my_applet.comment.is_empty());
        assert_eq!(my_applet.cursor_position, 0);
        assert_eq!(my_applet.selection, CreateLocationSelection::Id);
    }

    #[test]
    fn test_save_parsing() {
        let my_inv = inventory::Inventory::open_in_memory().unwrap();

        let mut my_applet = CreateLocationApplet::new();

        my_applet.name = "Some_name".into();
        my_applet.id = "1".into();
        assert!(my_applet.save_location(&my_inv).is_ok());
        my_applet.id = "nan".into();
        assert!(my_applet.save_location(&my_inv).is_err());
        my_applet.id = "0xff".into();
        assert!(my_applet.save_location(&my_inv).is_err());
        my_applet.id = "1".into();
        assert!(my_applet.save_location(&my_inv).is_err());
        my_applet.id = "2".into();
        assert!(my_applet.save_location(&my_inv).is_ok());

        my_applet.id = "3".into();
        my_applet.name = "".into();
        assert!(my_applet.save_location(&my_inv).is_err());
        my_applet.name = "43".into();
        assert!(my_applet.save_location(&my_inv).is_ok());
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
