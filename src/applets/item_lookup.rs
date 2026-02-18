use super::applet::Applet;
use crate::db::inventory;
use crate::{App, AppState};
use crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::Style;
use ratatui::widgets::{Block, Padding, Paragraph};
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
}

impl Applet for ItemLookupApplet {
    fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        _db: &inventory::Inventory,
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
            frame.render_widget(border, frame.area());
            frame.render_widget(line1, l1_area);
            frame.render_widget(
                id_widget,
                id_area.centered_horizontally(Constraint::Length(20)),
            );
        })?;

        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                    self.next_state = AppState::Exit
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
mod item_lookup_tests {
    use super::*;
    #[test]
    fn test_creation() {
        let my_applet = ItemLookupApplet::new();
        assert_eq!(my_applet.next_state, AppState::NoChange);
        assert_eq!(my_applet.cursor_position, 0);
        assert!(my_applet.id.is_empty());
    }
}
