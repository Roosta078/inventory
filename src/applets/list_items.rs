use super::applet::Applet;
use crate::AppState;
use crate::db::inventory::{Inventory, Location};
use crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;
use ratatui::style::Style;
use ratatui::text::Text;

pub struct ListItemsApplet {
    exit_applet: bool,
}

impl Default for ListItemsApplet {
    fn default() -> Self {
        Self { exit_applet: false }
    }
}

impl Applet for ListItemsApplet {
    fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        db: &Inventory,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.exit_applet = false;
        let text = Text::raw("TODO");
        terminal.draw(|frame| frame.render_widget(text, frame.area()))?;
        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.exit_applet = true,
                _ => {}
            }
        }
        Ok(())
    }
    fn get_name(&self) -> String {
        "List Items".to_string()
    }
    fn get_next_state(&self) -> AppState {
        if self.exit_applet {
            AppState::Exit
        } else {
            AppState::NoChange
        }
    }
}
