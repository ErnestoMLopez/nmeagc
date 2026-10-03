pub mod checkbox;
pub mod choice;
pub mod text;

use ratatui::widgets::WidgetRef;

use crossterm::event::KeyEvent;

pub trait Field: WidgetRef + Send + Sync {
    /// Returns the display label for this field.
    fn label(&self) -> &str;

    /// Handles keyboard input. Returns true if the input was consumed.
    fn handle_key_event(&mut self, event: &KeyEvent);

    /// Makes the field value masked.
    ///
    /// This is useful for password fields, where the value should not be displayed in plain text.
    fn masked(&mut self, masked: bool);

    /// Makes the field value hidden, so it can be skipped during rendering.
    ///
    /// This is useful for fields that are not visible due to previous fields with variable height,
    /// such as a dropdown that is opened and hides the following fields.
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
