use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Sparkline},
};
use std::io;
use std::sync::{Arc, Mutex};

// Assuming Inventory struct is available and imported from db.rs
// use crate::db::Inventory;

// Placeholder for Inventory struct if not in scope
#[derive(Debug, Clone)]
pub struct Inventory {
    // Mock fields for TUI structure
    items: Vec<String>,
    locations: Vec<String>,
}

impl Inventory {
    // Mock methods for TUI structure
    pub fn new() -> Self {
        Inventory {
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
            locations: vec!["Location A".to_string(), "Location B".to_string()],
        }
    }

    pub fn get_all_items(&self) -> Vec<String> {
        self.items.clone()
    }

    pub fn get_all_locations(&self) -> Vec<String> {
        self.locations.clone()
    }
}

pub fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the terminal
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Mock Inventory data (replace with actual database interaction)
    let inventory = Arc::new(Mutex::new(Inventory::new()));

    // Main application loop
    loop {
        terminal.draw(|f| {
            let size = f.size();
            // Define layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(10), // Header
                        Constraint::Percentage(40), // Items List
                        Constraint::Percentage(40), // Locations List
                        Constraint::Percentage(10), // Footer/Status
                    ]
                    .as_ref(),
                )
                .split(size);

            // Header
            let header_block = Block::default().title("Inventory Manager").title_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
            f.render_widget(header_block, chunks[0]);

            // Items List
            let items_data = inventory.lock().unwrap().get_all_items();
            let items: Vec<ListItem> = items_data
                .iter()
                .map(|i| ListItem::new(Span::raw(i)))
                .collect();
            let items_list = List::new(items)
                .block(Block::default().title("Items").borders(Borders::ALL))
                .style(Style::default().fg(Color::Gray))
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(
                items_list,
                chunks[1],
                &mut ratatui::widgets::ListState::default(),
            );

            // Locations List
            let locations_data = inventory.lock().unwrap().get_all_locations();
            let locations: Vec<ListItem> = locations_data
                .iter()
                .map(|loc| ListItem::new(Span::raw(loc)))
                .collect();
            let locations_list = List::new(locations)
                .block(Block::default().title("Locations").borders(Borders::ALL))
                .style(Style::default().fg(Color::Gray))
                .highlight_style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(
                locations_list,
                chunks[2],
                &mut ratatui::widgets::ListState::default(),
            );

            // Footer/Status Bar
            let footer_text = vec![Line::from(Span::styled(
                "Press 'q' to quit. Use arrows to navigate. (WIP: Add/Edit functionality)",
                Style::default().fg(Color::DarkGray),
            ))];
            let footer_block = Block::default()
                .title("Status")
                .title_style(Style::default().fg(Color::Yellow))
                .borders(Borders::ALL);
            let footer_paragraph = Paragraph::new(Text::from(footer_text))
                .block(footer_block)
                .style(Style::default().fg(Color::White));
            f.render_widget(footer_paragraph, chunks[3]);
        })?;

        // Event handling
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        // Exit the TUI
                        break;
                    }
                    KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                        // Handle navigation (will need state for focused widget)
                        // For now, just acknowledge input
                    }
                    _ => {} // Ignore other keys for now
                }
            }
        }
    }

    Ok(())
}
