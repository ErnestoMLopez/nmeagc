use crate::widgets::field::Field;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Widget, WidgetRef},
};

#[derive(Debug)]
pub struct TextInput {
    label: String,
    style: Style,
    required: bool,
    hidden: bool,
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
            required: false,
            hidden: false,
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

    /// Sets the style (not highlighted) for the text input and label.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
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
}

impl Field for TextInput {
    fn label(&self) -> &str {
        &self.label
    }

    fn handle_key_event(&mut self, event: &KeyEvent) {
        match event.code {
            KeyCode::Char(c) => {
                self.input.insert(self.cursor, c);
                self.cursor += 1;
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.input.remove(self.cursor - 1);
                    self.cursor -= 1;
                }
            }
            KeyCode::Delete => {
                if self.cursor < self.input.chars().count() {
                    self.input.remove(self.cursor);
                }
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor < self.input.chars().count() {
                    self.cursor += 1;
                }
            }
            KeyCode::Home => {
                self.cursor = 0;
            }
            KeyCode::End => {
                self.cursor = self.input.chars().count() + 1;
            }
            _ => {}
        }
    }

    fn focus(&mut self) {
        self.focused = true;
    }

    fn hidden(&mut self, hidden: bool) {
        self.hidden = hidden;
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

        let label_style = if self.hidden {
            self.style.add_modifier(Modifier::DIM)
        } else {
            self.style
        };

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

        // If the field is hidden or the input is empty, we don't render the input.
        if self.hidden || self.input.is_empty() {
            return;
        }

        let input_text = if self.obfuscated {
            "*".repeat(self.input.len())
        } else {
            self.input.clone()
        };

        // Render the input as a Line. If the field is focused, we change the style and also render
        // the blinking cursor as an individual Span
        let input = if let (true, Some((start, c))) =
            (self.focused, input_text.char_indices().nth(self.cursor))
        {
            let cursor_span = Span::styled(
                c.to_string(),
                self.style.add_modifier(Modifier::RAPID_BLINK),
            );
            Line::from(vec![
                Span::styled(&input_text[..start], self.style.reversed()),
                cursor_span,
                Span::styled(&input_text[(start + c.len_utf8())..], self.style.reversed()),
            ])
        } else {
            Line::from(Span::styled(&input_text, self.style))
        };

        input.render(input_area, buf);
    }
}
