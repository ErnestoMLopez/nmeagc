use crate::widgets::field::Field;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Widget,
};

pub struct Form {
    fields: Vec<Box<dyn Field>>,
    style: Style,
    validation_errors: Vec<String>,
}

impl Form {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
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
}

impl Widget for Form {
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
