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
    cancel_button: Option<String>,
    submit_button: Option<String>,
    style: Style,
    buttons_style: Style,
    highlight_style: Style,
    validation_errors: Vec<String>,
}

impl Form {
    /// Creates a new [`Form`] with no fields and default style
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            focus: FormFocus::None,
            cancel_button: None,
            submit_button: None,
            style: Style::default(),
            buttons_style: Style::default(),
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

    /// Sets the base style for the [`Form`] buttons
    pub fn buttons_style(mut self, style: Style) -> Self {
        self.buttons_style = style;
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

    /// Sets a custom label for the cancel button
    pub fn cancel_button(mut self, label: impl Into<String>) -> Self {
        self.cancel_button = Some(label.into());
        self
    }

    /// Sets a custom label for the submit button
    pub fn submit_button(mut self, label: impl Into<String>) -> Self {
        self.submit_button = Some(label.into());
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

    fn render_cancel_button(&self, area: Rect, buf: &mut Buffer) {
        let buttom_text = if let Some(ref label) = self.cancel_button {
            label
        } else {
            "Cancel"
        };

        let (button_label, button_style) = if self.focus == FormFocus::CancelButton {
            (
                format!(
                    "[ {:^width$} ]",
                    buttom_text,
                    width = (area.width as usize).saturating_sub(4)
                ),
                self.buttons_style.reversed(),
            )
        } else {
            (
                format!(
                    "  {:^width$}  ",
                    buttom_text,
                    width = (area.width as usize).saturating_sub(4)
                ),
                self.buttons_style,
            )
        };

        // Center the button
        let button_width = button_label.len() as u16;
        let x = area.x + (area.width.saturating_sub(button_width)) / 2;

        for (i, c) in button_label.chars().enumerate() {
            if x + (i as u16) < area.x + area.width {
                buf[(x + i as u16, area.y)].set_char(c);
                buf[(x + i as u16, area.y)].set_style(button_style);
            }
        }
    }

    fn render_submit_button(&self, area: Rect, buf: &mut Buffer) {
        let buttom_text = if let Some(ref label) = self.submit_button {
            label
        } else {
            "Submit"
        };

        let (button_label, button_style) = if self.focus == FormFocus::SubmitButton {
            (
                format!(
                    "[ {:^width$} ]",
                    buttom_text,
                    width = (area.width as usize).saturating_sub(4)
                ),
                self.buttons_style.reversed(),
            )
        } else {
            (
                format!(
                    "  {:^width$}  ",
                    buttom_text,
                    width = (area.width as usize).saturating_sub(4)
                ),
                self.buttons_style,
            )
        };

        // Center the button
        let button_width = button_label.len() as u16;
        let x = area.x + (area.width.saturating_sub(button_width)) / 2;

        for (i, c) in button_label.chars().enumerate() {
            if x + (i as u16) < area.x + area.width {
                buf[(x + i as u16, area.y)].set_char(c);
                buf[(x + i as u16, area.y)].set_style(button_style);
            }
        }
    }
}

impl Widget for &Form {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Layout for fields, validation error message and cancel and submit buttons
        let mut constraints = Vec::with_capacity(self.fields.len() + 5);

        for field in &self.fields {
            constraints.push(Constraint::Length(field.height()));
        }
        constraints.push(Constraint::Fill(1));
        constraints.extend([Constraint::Length(1); 3]);
        constraints.push(Constraint::Fill(0));

        let areas = Layout::vertical(constraints).split(area);

        for (i, field) in self.fields.iter().enumerate() {
            field.render_ref(areas[i], buf);
        }

        // Render submit button
        if let Some(row_area) = areas.get(self.fields.len() + 3) {
            let layout = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Percentage(30),
                Constraint::Fill(1),
                Constraint::Percentage(30),
                Constraint::Fill(1),
            ]);
            let [_, cancel_button_area, _, submit_button_area, _] = row_area.layout(&layout);
            self.render_cancel_button(cancel_button_area, buf);
            self.render_submit_button(submit_button_area, buf);
        }

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

#[derive(PartialEq)]
enum FormFocus {
    Field(usize),
    SubmitButton,
    CancelButton,
    None,
}
