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
    hidden: bool,
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
            hidden: false,
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

    fn handle_key_event(&mut self, event: &KeyEvent) {
        match event.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle();
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
        if self.required && !self.checked {
            Err(vec![format!("Field '{}' must be checked", self.label)])
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
            Constraint::Min(1),
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

        // If the field is hidden we don't render the label. This could be hidding more characters
        // than really needed, but we don't know the width of the rest of the fields, so this is the
        // best we can do.
        if self.hidden {
            return;
        }

        let required_marker = if self.required { "*" } else { "" };
        let label_text = format!(" {}{}", self.label, required_marker);
        let label = Span::styled(&label_text, self.style);

        label.render(label_area, buf);
    }
}
