use crate::widgets::field::Field;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Widget, WidgetRef},
};

#[derive(Debug)]
pub struct TextInput {
    label: String,
    style: Style,
    highlighted_style: Style,
    required: bool,
    focused: bool,
    obfuscated: bool,
    pub input: String, // TODO: Remove pub after refactoring the rest of the app
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
            input: String::new(),
            cursor: 0,
        }
    }

    /// Sets an initial value for the text input.
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

    fn enter_char(&mut self, character: char) {
        let index = self
            .input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor)
            .unwrap_or(self.input.len());
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
}

impl Field for TextInput {
    fn label(&self) -> &str {
        &self.label
    }

    fn style(&mut self, style: Style) {
        self.style = style;
    }

    fn highlight_style(&mut self, style: Style) {
        self.highlighted_style = style;
    }

    fn handle_key_event(&mut self, event: &KeyEvent) -> bool {
        match event.code {
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
            KeyCode::Left => {
                self.move_cursor_left();
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
        if self.required && self.input.is_empty() {
            Err(vec![format!("{} is required", self.label)])
        } else {
            Ok(())
        }
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

        let label_style = self.style;

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
