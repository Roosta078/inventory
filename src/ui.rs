use crossterm::{event::Event, execute, terminal};
use tui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
};

mod db;

struct App {
    inventory: db::inventory::Inventory,
}

impl App {
    fn new() -> Result<App, Box<dyn std::error::Error>> {
        let inventory = db::inventory::Inventory::open_in_file("my.db")?;
        Ok(App { inventory })
    }

    fn ui<B: tui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<(), tui::Error> {
        let backend = CrosstermBackend::new(std::io::stdout());
        let mut terminal = Terminal::new(backend)?;

        loop {
            crossterm::execute!(terminal.backend_mut(), terminal::EnableMouseCapture)?;
            let size = terminal.size()?;

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Min(1), Constraint::Max(size.height - 2)])
                .split(size);

            let table = self.get_table_data();

            terminal.draw(|mut f| {
                f.render_widget(
                    Block::new().borders(Borders::ALL).title("Inventory"),
                    chunks[0],
                );
                f.render_stateful_widget(
                    Table::new(vec![table.headers])
                        .block(Block::default().borders(Borders::ALL))
                        .highlight_style(
                            Style::default()
                                .bg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                    chunks[1],
                    &mut self.table_state,
                );
            })?;

            if let Event::Key(e) = crossterm::event::read()? {
                match e.code {
                    tui::event::KeyCode::Esc => return Ok(()),
                    _ => {}
                }
            }

            crossterm::execute!(terminal.backend_mut(), terminal::DisableMouseCapture)?;
        }
    }

    fn get_table_data(&self) -> TableData {
        let mut headers = vec![];
        for field in db::inventory::Item::fields() {
            headers.push(field.to_string());
        }
        headers.push("Location".to_string());

        let mut rows = vec![];
        for item in self.inventory.get_all_items()? {
            let mut row = vec![
                item.id.to_string(),
                item.name.clone(),
                item.comment.unwrap_or_default(),
                item.location_id.unwrap_or_default().to_string(),
            ];
            if let Some(location) = self
                .inventory
                .search_location_id(item.location_id.unwrap_or_default())
            {
                row.push(location.name);
            } else {
                row.push("Unknown".to_string());
            }
            rows.push(Row::new(row));
        }

        TableData { headers, rows }
    }
}

struct TableState {
    selected: usize,
    scroll_offset: usize,
}

impl Default for TableState {
    fn default() -> Self {
        Self {
            selected: 0,
            scroll_offset: 0,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new()?;
    execute!(std::io::stdout(), terminal::EnableMouseCapture)?;
    crossterm::execute!(std::io::stdout(), terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    app.ui(&mut terminal)
}
