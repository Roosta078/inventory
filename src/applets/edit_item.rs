use super::applet::Applet;
use crate::AppState;
use crate::db::inventory;
use crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph};

pub struct EditItemApplet {
    next_state: AppState,
    item: inventory::Item,
    id: i64,
    cursor_position: u16,
    selection: EditItemSelection,
    loc_id_str: String,
}
#[derive(PartialEq)]
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
    fn save_location(&self, db: &inventory::Inventory) -> Result<(), Box<dyn std::error::Error>> {
        //TODO implement saving to db
        // db.edit_location(&self.loc)?;
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

    fn refresh(&mut self, db: &inventory::Inventory) {
        self.item = db.search_item_id(self.id).unwrap();
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
