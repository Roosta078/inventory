use super::applet::Applet;
use crate::AppState;
use crate::db::inventory::Inventory;
use crossterm::event::{self, KeyCode};
use ratatui::DefaultTerminal;
use ratatui::style::Style;
use ratatui::widgets::{Block, List, ListDirection, ListState};
pub struct TopMenuApplet {
    exit_applet: bool,
    list_state: ListState,
    next_state: AppState,
}

impl Default for TopMenuApplet {
    fn default() -> Self {
        Self {
            exit_applet: false,
            list_state: ListState::default().with_selected(Some(0)),
            next_state: AppState::NoChange,
        }
    }
}

impl Applet for TopMenuApplet {
    fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        _db: &Inventory,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.exit_applet = false;
        self.next_state = AppState::NoChange;
        let list = List::new(["List Locations", "List Items", "Create Location", "Exit"])
            .block(
                Block::bordered()
                    .title("Inventory Manager")
                    .title_bottom("Press 'q' or Esc to exit"),
            )
            .style(Style::new().white())
            .highlight_style(Style::new().bold())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        terminal
            .draw(|frame| frame.render_stateful_widget(list, frame.area(), &mut self.list_state))?;
        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => self.next_state = AppState::Exit,
                KeyCode::Down => self.list_state.select_next(),
                KeyCode::Up => self.list_state.select_previous(),
                KeyCode::Enter => match self.list_state.selected().unwrap_or(3) {
                    0 => self.next_state = AppState::ListLocations,
                    1 => self.next_state = AppState::ListItems,
                    2 => self.next_state = AppState::CreateLocation,
                    3 => self.next_state = AppState::Exit,
                    _ => (),
                },
                _ => {}
            }
        }
        Ok(())
    }

    fn get_name(&self) -> String {
        "Top Menu".to_string()
    }

    fn get_next_state(&self) -> AppState {
        self.next_state.clone()
    }
}
