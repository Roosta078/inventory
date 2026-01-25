use super::applet::Applet;
use crate::AppState;
use crate::db::inventory;
use crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph};

pub struct EditLocationApplet {
    next_state: AppState,
    loc: inventory::Location,
    id: i64,
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
    fn save_location(&self, db: &inventory::Inventory) -> Result<(), Box<dyn std::error::Error>> {
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
        if self.id != self.loc.id {
            if let Some(loc) = db.search_location_id(self.id) {
                self.loc = loc;
                if self.loc.comment.is_none() {
                    self.loc.comment = Some("".to_string());
                }
            }
        }
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
