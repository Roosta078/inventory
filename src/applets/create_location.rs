use super::applet::Applet;
use crate::AppState;
use crate::db::inventory;
use crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph};

pub struct CreateLocationApplet {
    next_state: AppState,
    id: String,
    name: String,
    comment: String,
    selection: CreateLocationSelection,
    cursor_position: usize,
}

#[derive(PartialEq)]
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
        let comment_opt = if self.comment.is_empty() {
            None
        } else {
            Some(self.comment.clone())
        };
        let new_location = inventory::Location {
            id: self.id.parse()?,
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
        // if self.id != self.loc.id {
        //     if let Some(loc) = db.search_location_id(self.id) {
        //         self.loc = loc;
        //         if self.loc.comment.is_none() {
        //             self.loc.comment = Some("".to_string());
        //         }
        //     }
        // }
        //Prepare Draw
        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(3),
        ]);
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
            let [id_area, name_area, comment_area, cancel_area, save_area] =
                vertical.areas(frame.area());
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
    fn get_name(&self) -> String {
        "Create Location".to_string()
    }
    fn get_next_state(&self) -> AppState {
        self.next_state.clone()
    }
}
