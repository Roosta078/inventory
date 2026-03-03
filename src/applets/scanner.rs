use super::applet::Applet;
use crate::AppState;
use crate::db::inventory;
use crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph};

pub struct ScannerApplet {
    next_state: AppState,
    input_text: String,
}

impl ScannerApplet {
    pub fn new() -> Self {
        Self {
            next_state: AppState::NoChange,
            input_text: String::new(),
        }
    }
    pub fn get_input(&self) -> String {
        self.input_text.clone()
    }
}

impl Applet for ScannerApplet {
    fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        _db: &inventory::Inventory,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.next_state = AppState::NoChange;
        let border = Block::bordered()
            .title_top("Inventory Manager")
            .title_bottom("Press Esc to exit");
        let vertical = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ]);
        let line1 = Paragraph::new("SCANNER MODE")
            .style(Style::default())
            .centered();
        let line2 = Paragraph::new("If you got here by accident, press 'Esc' to go back")
            .style(Style::default().red())
            .centered();
        terminal.draw(|frame| {
            let inner_area = border.inner(frame.area());
            let [_, l1_area, l2_area, _] = vertical.areas(inner_area);
            frame.render_widget(border, frame.area());
            frame.render_widget(line1, l1_area);
            frame.render_widget(line2, l2_area);
        })?;
        while let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Esc => {
                    self.next_state = AppState::Exit;
                    break;
                }
                KeyCode::Char(c) => self.input_text.push(c),
                KeyCode::Enter => break,
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
}
