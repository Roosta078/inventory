use std::io::Error;

use super::applet::Applet;
use crate::AppState;
use crate::db::inventory;
use crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Layout, Position};
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph};

pub struct ErrorApplet {
    next_state: AppState,
    error_text: String,
    selection: ErrorSelection,
}

#[derive(PartialEq)]
enum ErrorSelection {
    Accept,
}

impl ErrorApplet {
    pub fn new(text: &str) -> Self {
        Self {
            next_state: AppState::NoChange,
            error_text: text.to_string(),
            selection: ErrorSelection::Accept,
        }
    }
}

impl Applet for ErrorApplet {
    fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        _db: &inventory::Inventory,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.next_state = AppState::NoChange;

        let vertical = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Fill(1),
        ]);
        let line1 = Paragraph::new("Inventory Manager Encountered an Error")
            .style(Style::default())
            .centered();
        let line2 = Paragraph::new(self.error_text.to_string())
            .style(Style::default().red())
            .centered();
        let accept_button = Paragraph::new("Ok")
            .style(if self.selection == ErrorSelection::Accept {
                Style::default().yellow()
            } else {
                Style::default()
            })
            .centered()
            .block(Block::bordered());
        terminal.draw(|frame| {
            let [_, l1_area, l2_area, accept_area, _] = vertical.areas(frame.area());
            frame.render_widget(line1, l1_area);
            frame.render_widget(line2, l2_area);
            frame.render_widget(accept_button, accept_area);
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
