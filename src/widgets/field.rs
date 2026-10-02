use ratatui::widgets::WidgetRef;

use crossterm::event::KeyEvent;

pub trait Field: WidgetRef + Send + Sync {
    /// Returns the display label for this field.
    fn label(&self) -> &str;

    /// Handles keyboard input. Returns true if the input was consumed.
    fn handle_key_event(&mut self, event: &KeyEvent);

    /// Makes the field hidden, so it can be skipped during rendering.
    fn hidden(&mut self, hidden: bool);

    /// Validates the field and returns any errors.
    fn validate(&self) -> Result<(), Vec<String>>;

    /// Returns the height needed to render this field.
    fn height(&self) -> u16 {
        1
    }

    /// Returns whether this field is required.
    fn is_required(&self) -> bool;
}
