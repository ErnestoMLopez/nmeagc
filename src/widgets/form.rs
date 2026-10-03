use crate::widgets::field::Field;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BlockExt, Widget},
};

pub struct Form<'a> {
    fields: Vec<Box<dyn Field>>,
    block: Option<Block<'a>>,
    style: Style,
    validation_errors: Vec<String>,
}

impl<'a> Form<'a> {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            block: None,
            style: Style::default(),
            validation_errors: Vec::new(),
        }
    }
}

impl<'a> Widget for Form<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner_area = self.block.inner_if_some(area);

        self.block.as_ref().render(area, buf);

        if inner_area.height < 2 || inner_area.width < 10 {
            return;
        }

        // Layout for fields and submit button. Selectable fields can have variable height, and they
        // hide following fields values when opened.
        let mut fields_area = inner_area;

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
                x: inner_area.x,
                y: inner_area.y + inner_area.height.saturating_sub(1),
                width: inner_area.width,
                height: 1,
            };

            let error_line = Line::from(Span::styled(error_msg, self.style.red()));
            error_line.render(error_area, buf);
        }
    }
}
