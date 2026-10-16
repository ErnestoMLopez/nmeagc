use crate::widgets::field::Field;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Widget,
};

pub struct Form {
    fields: Vec<Box<dyn Field>>,
    focus: FormFocus,
    style: Style,
    highlight_style: Style,
    validation_errors: Vec<String>,
}

impl Form {
    /// Creates a new [`Form`] with no fields and default style
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            focus: FormFocus::None,
            style: Style::default(),
            highlight_style: Style::default(),
            validation_errors: Vec::new(),
        }
    }

    /// Adds a field to the [`Form`]
    pub fn field<T: Field + 'static>(mut self, field: T) -> Self {
        self.fields.push(Box::new(field));
        self
    }

    /// Sets the base style for the [`Form`] and every [`Field`] added before to it
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;

        for field in &mut self.fields {
            field.style(style);
        }

        self
    }

    /// Sets the style for any highlighted item of the [`Form`] and every [`Field`] added before to it
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;

        for field in &mut self.fields {
            field.highlight_style(style);
        }

        self
    }

    /// Handles keyboard events
    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        // If a field is focused we pass the event to it and return if it was used
        if let FormFocus::Field(index) = self.focus {
            if let Some(field) = self.fields.get_mut(index) {
                if field.handle_key_event(&key_event) {
                    return;
                }
            }
        }

        match key_event.code {
            KeyCode::Tab | KeyCode::Down => {
                self.focus_next();
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.focus_previous();
            }
            _ => {}
        }

        if let FormFocus::Field(index) = self.focus {
            if let Some(field) = self.fields.get_mut(index) {
                field.handle_key_event(&key_event);
            }
        }
    }

    /// Moves focus to the next field or button.
    fn focus_next(&mut self) {
        self.focus = match self.focus {
            FormFocus::Field(index) => {
                if let Some(field) = self.fields.get_mut(index) {
                    field.focus(false);
                }

                if index < self.fields.len() - 1 {
                    FormFocus::Field(index + 1)
                } else {
                    FormFocus::CancelButton
                }
            }
            FormFocus::CancelButton => FormFocus::SubmitButton,
            FormFocus::SubmitButton => FormFocus::Field(0),
            FormFocus::None => FormFocus::Field(0),
        };

        if let FormFocus::Field(index) = self.focus {
            if let Some(field) = self.fields.get_mut(index) {
                field.focus(true);
            }
        }
    }

    /// Moves focus to the previous field or button.
    fn focus_previous(&mut self) {
        self.focus = match self.focus {
            FormFocus::Field(index) => {
                if let Some(field) = self.fields.get_mut(index) {
                    field.focus(false);
                }

                if index > 0 {
                    FormFocus::Field(index - 1)
                } else {
                    FormFocus::SubmitButton
                }
            }
            FormFocus::CancelButton => FormFocus::Field(self.fields.len().saturating_sub(1)),
            FormFocus::SubmitButton => FormFocus::CancelButton,
            FormFocus::None => FormFocus::SubmitButton,
        };

        if let FormFocus::Field(index) = self.focus {
            if let Some(field) = self.fields.get_mut(index) {
                field.focus(true);
            }
        }
    }
}

impl Widget for &Form {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Layout for fields and submit button. Selectable fields can have variable height, and they
        // hide following fields values when opened.
        let mut fields_area = area;

        for field in &self.fields {
            let field_height = field.height();
            let layout = Layout::vertical([Constraint::Length(field_height), Constraint::Min(0)]);
            let [top, bottom] = fields_area.layout(&layout);
            field.render_ref(top, buf);
            fields_area = bottom;
        }

        // Render submit button
        // let layout = Layout::vertical([
        //     Constraint::Length(1),
        //     Constraint::Length(1),
        //     Constraint::Min(0),
        // ]);
        // let [_, button, _] = fields_area.layout(&layout);
        // self.render_submit_button(button, buf);

        // Render validation errors summary if any
        if !self.validation_errors.is_empty() {
            let error_count = self.validation_errors.len();
            let error_msg = format!(
                "{} validation error{}",
                error_count,
                if error_count == 1 { "" } else { "s" }
            );

            let error_area = Rect {
                x: area.x,
                y: area.y + area.height.saturating_sub(1),
                width: area.width,
                height: 1,
            };

            let error_line = Line::from(Span::styled(error_msg, self.style.red()));
            error_line.render(error_area, buf);
        }
    }
}

enum FormFocus {
    Field(usize),
    SubmitButton,
    CancelButton,
    None,
}
