use crossterm::event::{self, KeyCode, KeyEvent};
use ratatui::DefaultTerminal;
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, List, ListDirection, ListItem, ListState};
mod db;

struct App {
    state: AppState,
    top_menu_options: Vec<String>,
    list_state: ListState,
}

enum AppState {
    TopMenu,
    Exit,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: AppState::TopMenu,
            top_menu_options: vec![
                "Item 1".to_string(),
                "Item 2".to_string(),
                "Quit".to_string(),
            ],
            list_state: ListState::default(),
        }
    }
}

impl App {
    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
        self.list_state.select_first();
        loop {
            match self.state {
                AppState::Exit => break,
                AppState::TopMenu => self.run_top_menu(terminal)?,
            }
        }
        Ok(())
    }

    fn run_top_menu(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let list = List::new(self.top_menu_options.clone())
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
                KeyCode::Char('q') | KeyCode::Esc => self.state = AppState::Exit,
                KeyCode::Down => self.list_state.select_next(),
                KeyCode::Up => self.list_state.select_previous(),
                KeyCode::Enter => {
                    if self.list_state.selected().unwrap_or(0) == 2 {
                        self.state = AppState::Exit;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let myapp = App::default();
    ratatui::run(|terminal| myapp.run(terminal))
}
