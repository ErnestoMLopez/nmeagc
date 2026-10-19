use crate::widgets::field::Field;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::Span,
    widgets::{Widget, WidgetRef},
};

#[derive(Debug)]
pub struct CheckboxInput {
    label: String,
    style: Style,
    marker: char,
    checked: bool,
    required: bool,
    focused: bool,
}

impl CheckboxInput {
    /// Creates a new [`CheckboxInput`] with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            style: Style::default(),
            marker: '✓',
            checked: false,
            required: false,
            focused: false,
        }
    }

    /// Sets the style (not highlighted) for the checkbox and label.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the marker character for the checkbox when checked.
    pub fn marker(mut self, marker: char) -> Self {
        self.marker = marker;
        self
    }

    /// Sets the initial state as checked.
    pub fn checked(mut self) -> Self {
        self.checked = true;
        self
    }

    /// Marks this field as required (must be checked).
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    fn toggle(&mut self) {
        self.checked = !self.checked;
    }
}

impl Field for CheckboxInput {
    fn label(&self) -> &str {
        &self.label
    }

    fn style(&mut self, style: Style) {
        self.style = style;
    }

    fn highlight_style(&mut self, _style: Style) {}

    fn handle_key_event(&mut self, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle();
                true
            }
            _ => false,
        }
    }

    fn focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn validate(&self) -> Result<(), Vec<String>> {
        if self.required && !self.checked {
            Err(vec![format!("Field must be checked")])
        } else {
            Ok(())
        }
    }

    fn height(&self) -> u16 {
        1
    }

    fn is_required(&self) -> bool {
        self.required
    }
}

impl WidgetRef for CheckboxInput {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 || area.width < 4 {
            return;
        }

        let checkbox_style = if self.focused {
            self.style.add_modifier(Modifier::REVERSED)
        } else {
            self.style
        };

        let layout = Layout::horizontal([
            Constraint::Length(3),
            Constraint::Max(1),
            Constraint::Fill(1),
        ]);
        let [check_area, _, label_area] = area.layout(&layout);

        // Render checkbox
        let checkbox = if self.checked {
            Span::styled(format!("[{}]", self.marker), checkbox_style)
        } else {
            Span::styled("[ ]", checkbox_style)
        };

        checkbox.render(check_area, buf);

        let required_marker = if self.required { "*" } else { "" };
        let label_text = format!(" {}{}", self.label, required_marker);
        let label = Span::styled(&label_text, self.style.bold());

        label.render(label_area, buf);
    }
}
