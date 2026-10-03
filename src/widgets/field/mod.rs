mod checkbox;
mod choice;
mod text;

use crossterm::event::KeyEvent;
use ratatui::widgets::WidgetRef;

pub use checkbox::CheckboxInput;
pub use choice::ChoiceInput;
pub use text::TextInput;

pub trait Field: WidgetRef + Send + Sync {
    /// Returns the display label for this field.
    fn label(&self) -> &str;

    /// Handles keyboard input.
    fn handle_key_event(&mut self, event: &KeyEvent);

    /// Makes the field value hidden, so it can be skipped during rendering.
    ///
    /// This is useful for fields that are not visible due to previous fields with variable height,
    /// such as a dropdown that is opened and hides the following fields.
    fn hidden(&mut self, hidden: bool);

    /// Validates the field and returns any errors.
    fn validate(&self) -> Result<(), Vec<String>>;

    /// Returns the height needed to render this field.
    ///
    /// Normally this is 1, but some fields can have variable height, such as a dropdown that is
    /// opened and hides the following fields.
    fn height(&self) -> u16 {
        1
    }

    /// Returns whether this field is required.
    fn is_required(&self) -> bool;
}
