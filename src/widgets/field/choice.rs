use crate::widgets::field::{Field, FieldValue};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::Span,
    widgets::{List, ListState, StatefulWidget, Widget, WidgetRef},
};

pub struct ChoiceInput {
    label: String,
    style: Style,
    highlighted_style: Style,
    required: bool,
    focused: bool,
    options: Vec<(FieldValue, String)>,
    selected: Option<usize>,
    highlighted: usize,
    pub selection: ListState, // TODO: Remove after refactor
    is_open: bool,
}

impl ChoiceInput {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            style: Style::default(),
            highlighted_style: Style::default(),
            required: false,
            focused: false,
            options: Vec::new(),
            selected: None,
            highlighted: 0,
            selection: ListState::default().with_selected(Some(0)),
            is_open: false,
        }
    }

    /// Sets the initial selected choice.
    ///
    /// This is a fluent setter method, but must be called after the options were setted through the
    /// [`ChoiceInput::option()`] or [`ChoiceInput::options()`] methods, to correctly set the index
    /// to the selected option.
    pub fn with_selected(mut self, value: FieldValue) -> Self {
        for (i, (v, _)) in self.options.iter().enumerate() {
            if *v == value {
                self.selected = Some(i);
                self.selection.select(Some(i)); // TODO: Remove after refactor
                break;
            }
        }
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

    /// Adds an option to the select.
    pub fn option(mut self, value: FieldValue, display: impl Into<String>) -> Self {
        self.options.push((value, display.into()));
        self
    }

    /// Adds multiple options at once.
    pub fn options(mut self, options: Vec<(FieldValue, impl Into<String>)>) -> Self {
        for (value, display) in options {
            self.options.push((value, display.into()));
        }
        self
    }

    fn toggle_open(&mut self) {
        self.is_open = !self.is_open;
        if self.is_open {
            if let Some(index) = self.selected {
                self.highlighted = index;
            }
        }
    }

    fn select_highlighted(&mut self) {
        if !self.options.is_empty() {
            self.selected = Some(self.highlighted);
        }
        self.is_open = false;
    }

    fn move_highlight_up(&mut self) {
        self.highlighted = self.highlighted.saturating_sub(1);
    }

    fn move_highlight_down(&mut self) {
        if self.highlighted < self.options.len().saturating_sub(1) {
            self.highlighted += 1;
        }
    }
}

impl Field for ChoiceInput {
    fn label(&self) -> &str {
        &self.label
    }

    fn value(&self) -> FieldValue {
        let field_value = self
            .selected
            .and_then(|i| self.options.get(i))
            .map(|(_, display)| display);

        field_value
            .map(|s| s.parse::<i64>())
            .and_then(|d| d.ok())
            .map(|n| FieldValue::Int(n))
            .unwrap_or(FieldValue::Text(
                field_value.cloned().unwrap_or("".to_string()),
            ))
    }

    fn style(&mut self, style: Style) {
        self.style = style;
    }

    fn highlight_style(&mut self, style: Style) {
        self.highlighted_style = style;
    }

    fn handle_key_event(&mut self, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                if self.is_open {
                    self.select_highlighted();
                } else {
                    self.toggle_open();
                }
                true
            }
            KeyCode::Esc if self.is_open => {
                self.is_open = false;
                true
            }
            KeyCode::Up if self.is_open => {
                self.move_highlight_up();
                true
            }
            KeyCode::Down if self.is_open => {
                self.move_highlight_down();
                true
            }
            _ => false,
        }
    }

    fn focus(&mut self, focused: bool) {
        self.focused = focused;

        if !focused {
            self.is_open = false;
        }
    }

    fn validate(&self) -> Result<(), Vec<String>> {
        if self.required && self.selected.is_none() {
            Err(vec![format!("Field is required")])
        } else {
            Ok(())
        }
    }

    fn height(&self) -> u16 {
        if self.is_open {
            1 + self.options.len() as u16
        } else {
            1
        }
    }

    fn is_required(&self) -> bool {
        self.required
    }
}

impl WidgetRef for ChoiceInput {
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
            self.style.add_modifier(Modifier::REVERSED)
        } else {
            self.style
        };

        let layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]);
        let [choice_area, dropdown_area] = input_area.layout(&layout);

        let layout = Layout::horizontal([Constraint::Fill(1), Constraint::Length(3)]);
        let [selected_area, arrow_area] = choice_area.layout(&layout);

        let input_text = self
            .selected
            .and_then(|i| self.options.get(i))
            .map(|(_, display)| display.clone())
            .unwrap_or("-- Select --".to_string());

        // We set the style for all the area so the focus is visible even if the input is empty
        buf.set_style(choice_area, input_style);

        let input = Span::styled(input_text, input_style);
        let arrow = Span::styled(if self.is_open { "▲" } else { "▼" }, input_style);

        input.render(selected_area, buf);
        arrow.render(arrow_area, buf);

        // Render dropdown if open
        if self.is_open && dropdown_area.height > 0 {
            let dropdown = List::new(self.options.iter().map(|(_, option)| option.as_str()))
                .style(input_style)
                .highlight_style(self.highlighted_style);

            let mut state = ListState::default().with_selected(Some(self.highlighted));

            StatefulWidget::render(dropdown, dropdown_area, buf, &mut state);
        }
    }
}
