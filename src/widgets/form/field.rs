mod checkbox;
mod choice;
mod text;
mod validation;

pub use checkbox::CheckboxInput;
pub use choice::ChoiceInput;
use crossterm::event::KeyEvent;
use ratatui::style::Style;
use ratatui::widgets::WidgetRef;
pub use text::TextInput;
pub use validation::{ValidationError, Validator};

pub trait Field: WidgetRef + Send + Sync {
    /// Returns the display label for this field.
    fn label(&self) -> &str;

    /// Returns the final input value
    fn value(&self) -> FieldValue;

    /// Sets the style for the field in a non focused state
    fn style(&mut self, style: Style);

    /// Sets the style for a highlighted part of the field
    fn highlight_style(&mut self, style: Style);

    /// Handles keyboard input.
    ///
    /// Returns true if the field consumes the event.
    fn handle_key_event(&mut self, event: &KeyEvent) -> bool;

    /// Focuses the field, allowing it to receive input and highlight it during rendering.
    fn focus(&mut self, focused: bool);

    /// Validates the field and returns any errors.
    fn validate(&self) -> Result<(), Vec<String>>;

    /// Returns the height needed to render this field.
    ///
    /// Normally this is 1, but some fields can have variable height, such as a
    /// dropdown that covers and skips the fields rendered below it while open.
    fn height(&self) -> u16 {
        1
    }

    /// Returns whether this field is required.
    fn is_required(&self) -> bool;
}

#[derive(PartialEq)]
pub enum FieldValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
}

pub struct FieldResult {
    field: String,
    value: FieldValue,
}

impl FieldResult {
    pub fn new(field: impl Into<String>, value: FieldValue) -> Self {
        Self {
            field: field.into(),
            value,
        }
    }
    /// Decompose the [`FieldResult`] in a reference to its label and its final value
    pub fn decompose(&self) -> (&str, &FieldValue) {
        (self.field.as_str(), &self.value)
    }
}
