use crate::AppState;
use crate::db::inventory::Inventory;
use ratatui::DefaultTerminal;

pub trait Applet {
    fn run(
        &mut self,
        _terminal: &mut DefaultTerminal,
        _db: &Inventory,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    fn get_next_state(&self) -> AppState;
    fn refresh(&mut self, _db: &Inventory) {}
}
