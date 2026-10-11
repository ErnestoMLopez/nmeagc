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
    focused_field: Option<usize>,
    style: Style,
    validation_errors: Vec<String>,
}

impl Form {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            focused_field: None,
            style: Style::default(),
            validation_errors: Vec::new(),
        }
    }

    pub fn field<T: Field + 'static>(mut self, field: T) -> Self {
        self.fields.push(Box::new(field));
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Tab => {
                self.focus_next();
            }
            KeyCode::BackTab => {
                self.focus_previous();
            }
            _ => {}
        }

        if let Some(index) = self.focused_field {
            if let Some(field) = self.fields.get_mut(index) {
                field.handle_key_event(&key_event);
            }
        }
    }

    /// Moves focus to the next field or make the form out of focus.
    fn focus_next(&mut self) {
        self.focused_field = if let Some(index) = self.focused_field {
            if let Some(field) = self.fields.get_mut(index) {
                field.focus(false);
            }

            if index < self.fields.len() - 1 {
                let index = index + 1;
                if let Some(field) = self.fields.get_mut(index) {
                    field.focus(true);
                }
                Some(index)
            } else {
                None
            }
        } else {
            if let Some(field) = self.fields.get_mut(0) {
                field.focus(true);
            }
            Some(0)
        };
    }

    /// Moves focus to the previous field or make the form out of focus.
    fn focus_previous(&mut self) {
        self.focused_field = if let Some(index) = self.focused_field {
            if let Some(field) = self.fields.get_mut(index) {
                field.focus(false);
            }
            if index > 0 {
                let index = index - 1;
                if let Some(field) = self.fields.get_mut(index) {
                    field.focus(true);
                }
                Some(index)
            } else {
                None
            }
        } else {
            let index = self.fields.len().saturating_sub(1);
            if let Some(field) = self.fields.get_mut(index) {
                field.focus(true);
            }
            Some(index)
        };
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
