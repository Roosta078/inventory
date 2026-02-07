use super::applet::Applet;
use crate::AppState;
use crate::db::inventory;
use crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph};

pub struct ErrorApplet {
    next_state: AppState,
    error_text: String,
    selection: ErrorSelection,
}

#[derive(PartialEq, Debug)]
enum ErrorSelection {
    Accept,
}

impl ErrorApplet {
    pub fn new(text: String) -> Self {
        Self {
            next_state: AppState::NoChange,
            error_text: text,
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
        let border = Block::bordered()
            .title_top("Inventory Manager")
            .title_bottom("Press 'q' or Esc to exit");
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
        let line2 = Paragraph::new(self.error_text.clone())
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
            let inner_area = border.inner(frame.area());
            let [_, l1_area, l2_area, accept_area, _] = vertical.areas(inner_area);
            frame.render_widget(border, frame.area());
            frame.render_widget(line1, l1_area);
            frame.render_widget(line2, l2_area);
            frame.render_widget(
                accept_button,
                accept_area.centered_horizontally(Constraint::Length(20)),
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
mod error_tests {
    use super::*;
    #[test]
    fn test_creation() {
        let my_applet = ErrorApplet::new("msg".into());
        assert_eq!(my_applet.next_state, AppState::NoChange);
        assert_eq!(my_applet.error_text, "msg".to_string());
        assert_eq!(my_applet.selection, ErrorSelection::Accept);
    }
    #[test]
    fn test_trait_functions() {
        let mut my_applet = ErrorApplet::new("msg".into());
        let my_inv = inventory::Inventory::open_in_memory().unwrap();
        my_applet.refresh(&my_inv); //ensure it doesn't panic

        assert_eq!(my_applet.next_state, AppState::NoChange);

        //would like to test run, but not sure how
    }
}
