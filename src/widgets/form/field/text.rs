use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Widget, WidgetRef};

use crate::widgets::form::field::validation::Validator;
use crate::widgets::form::field::{Field, FieldValue};

pub struct TextInput {
    label: String,
    style: Style,
    highlighted_style: Style,
    required: bool,
    focused: bool,
    obfuscated: bool,
    validators: Vec<Box<dyn Validator>>,
    input: String,
    cursor: usize,
}

impl TextInput {
    /// Creates a new [`TextInput`] with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            style: Style::default(),
            highlighted_style: Style::default(),
            required: false,
            focused: false,
            obfuscated: false,
            validators: Vec::new(),
            input: String::new(),
            cursor: 0,
        }
    }

    /// Sets an initial value for the [`TextInput`].
    pub fn with_input(mut self, input: impl Into<String>) -> Self {
        self.input = input.into();
        self
    }

    /// Marks this field as required (must be filled).
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Makes the field value obfuscated, so it can be used for password input.
    pub fn obfuscated(mut self) -> Self {
        self.obfuscated = true;
        self
    }

    /// Adds a [`Validator`] to the field
    pub fn validator(mut self, validator: Box<dyn Validator>) -> Self {
        self.validators.push(validator);
        self
    }

    fn move_cursor_left(&mut self) {
        self.cursor = self
            .cursor
            .saturating_sub(1)
            .clamp(0, self.input.chars().count());
    }

    fn move_cursor_right(&mut self) {
        self.cursor = self
            .cursor
            .saturating_add(1)
            .clamp(0, self.input.chars().count());
    }

    /// Moves the cursor to the start of the previous word.
    fn move_cursor_word_left(&mut self) {
        let prefix = &self.input[..self.cursor_char_index()];
        let mut index = prefix.chars().count();
        let mut chars = prefix.chars().rev().peekable();

        while chars.next_if(|&c| is_word_separator(c)).is_some() {
            index -= 1;
        }
        while chars.next_if(|&c| !is_word_separator(c)).is_some() {
            index -= 1;
        }

        self.cursor = index;
    }

    /// Moves the cursor to the start of the next word.
    fn move_cursor_word_right(&mut self) {
        let mut index = self.cursor;
        let mut chars = self.input.chars().skip(self.cursor).peekable();

        while chars.next_if(|&c| !is_word_separator(c)).is_some() {
            index += 1;
        }
        while chars.next_if(|&c| is_word_separator(c)).is_some() {
            index += 1;
        }

        self.cursor = index;
    }

    fn enter_char(&mut self, character: char) {
        let index = self.cursor_char_index();
        self.input.insert(index, character);
        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        if self.cursor > 0 {
            let prev_chars = self.input.chars().take(self.cursor - 1);
            let next_chars = self.input.chars().skip(self.cursor);

            self.input = prev_chars.chain(next_chars).collect();
            self.move_cursor_left();
        }
    }

    fn cursor_char_index(&self) -> usize {
        self.input
            .char_indices()
            .nth(self.cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.input.len())
    }
}

impl Field for TextInput {
    fn label(&self) -> &str {
        &self.label
    }

    fn value(&self) -> FieldValue {
        FieldValue::Text(self.input.clone())
    }

    fn style(&mut self, style: Style) {
        self.style = style;
    }

    fn highlight_style(&mut self, style: Style) {
        self.highlighted_style = style;
    }

    fn handle_key_event(&mut self, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Char(_) if event.modifiers == KeyModifiers::CONTROL => false,
            KeyCode::Char(c) => {
                self.enter_char(c);
                true
            }
            KeyCode::Backspace => {
                self.delete_char();
                true
            }
            KeyCode::Delete => {
                self.move_cursor_right();
                self.delete_char();
                true
            }
            KeyCode::Left if event.modifiers == KeyModifiers::CONTROL => {
                self.move_cursor_word_left();
                true
            }
            KeyCode::Left => {
                self.move_cursor_left();
                true
            }
            KeyCode::Right if event.modifiers == KeyModifiers::CONTROL => {
                self.move_cursor_word_right();
                true
            }
            KeyCode::Right => {
                self.move_cursor_right();
                true
            }
            KeyCode::Home => {
                self.cursor = 0;
                true
            }
            KeyCode::End => {
                self.cursor = self.input.chars().count();
                true
            }
            _ => false,
        }
    }

    fn focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.required && self.input.is_empty() {
            errors.push("Field is required".to_string());
        }

        for validator in &self.validators {
            if let Err(msg) = validator.validate(&self.input) {
                errors.push(msg);
            }
        }

        errors.is_empty().then_some(()).ok_or(errors)
    }

    fn is_required(&self) -> bool {
        self.required
    }
}

impl WidgetRef for TextInput {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 || area.width < 1 {
            return;
        }

        let label_style = self.style.bold();

        let required_marker = if self.required { "*" } else { "" };
        let label_text = format!("{}{}:", self.label, required_marker);
        let label_span = Span::styled(&label_text, label_style);

        let layout = Layout::horizontal([
            Constraint::Length(label_text.len() as u16),
            Constraint::Max(1),
            Constraint::Fill(1),
        ]);
        let [label_area, _, input_area] = area.layout(&layout);

        label_span.render(label_area, buf);

        let input_style = if self.focused {
            self.style.reversed()
        } else {
            self.style
        };

        // We set the style for all the area so the focus is visible even if the input is empty
        buf.set_style(input_area, input_style);

        let input_text = if self.obfuscated {
            "*".repeat(self.input.len())
        } else {
            self.input.clone()
        };

        // Render the input as a Line. If the field is focused, we change the style and also render
        // the blinking cursor as an individual Span
        let input = match (self.focused, input_text.char_indices().nth(self.cursor)) {
            (false, _) => Line::from(Span::styled(&input_text, input_style)),
            (true, None) => Line::from(vec![
                Span::styled(&input_text, input_style),
                Span::styled(" ", self.highlighted_style),
            ]),
            (true, Some((cursor_index, c))) => Line::from(vec![
                Span::styled(&input_text[..cursor_index], input_style),
                Span::styled(c.to_string(), self.highlighted_style),
                Span::styled(&input_text[(cursor_index + c.len_utf8())..], input_style),
            ]),
        };

        input.render(input_area, buf);
    }
}

fn is_word_separator(character: char) -> bool {
    character.is_whitespace() || character == '/' || character == '.'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctrl_c_is_not_inserted_as_text() {
        let mut input = TextInput::new("Host");

        let consumed =
            input.handle_key_event(&KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));

        assert!(!consumed);
        assert_eq!(input.value(), FieldValue::Text(String::new()));
    }

    #[test]
    fn plain_char_is_inserted() {
        let mut input = TextInput::new("Host");

        let consumed =
            input.handle_key_event(&KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));

        assert!(consumed);
        assert_eq!(input.value(), FieldValue::Text("c".to_string()));
    }

    fn input_with_cursor_at_end(value: &str) -> TextInput {
        let mut input = TextInput::new("Path").with_input(value);
        input.handle_key_event(&KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
        input
    }

    #[test]
    fn ctrl_left_jumps_to_previous_word_boundary() {
        let mut input = input_with_cursor_at_end("foo bar/baz.rs");

        let consumed = input.handle_key_event(&KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL));

        assert!(consumed);
        assert_eq!(input.cursor, 12);

        input.handle_key_event(&KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL));
        assert_eq!(input.cursor, 8);

        input.handle_key_event(&KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL));
        assert_eq!(input.cursor, 4);

        input.handle_key_event(&KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL));
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn ctrl_right_jumps_to_next_word_boundary() {
        let mut input = TextInput::new("Path").with_input("foo bar/baz.rs");

        let consumed =
            input.handle_key_event(&KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL));

        assert!(consumed);
        assert_eq!(input.cursor, 4);

        input.handle_key_event(&KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL));
        assert_eq!(input.cursor, 8);

        input.handle_key_event(&KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL));
        assert_eq!(input.cursor, 12);

        input.handle_key_event(&KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL));
        assert_eq!(input.cursor, 14);
    }
}
