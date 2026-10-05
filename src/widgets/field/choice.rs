use crate::widgets::field::Field;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{ListState, Widget, WidgetRef},
};

#[derive(Debug)]
pub struct ChoiceInput {
    label: String,
    style: Style,
    required: bool,
    hidden: bool,
    focused: bool,
    options: Vec<(String, String)>,
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
            required: false,
            hidden: false,
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
    pub fn with_selected(mut self, value: &str) -> Self {
        for (i, (v, _)) in self.options.iter().enumerate() {
            if v == value {
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
    pub fn option(mut self, value: impl Into<String>, display: impl Into<String>) -> Self {
        self.options.push((value.into(), display.into()));
        self
    }

    /// Adds multiple options at once.
    pub fn options(mut self, options: Vec<(impl Into<String>, impl Into<String>)>) -> Self {
        for (value, display) in options {
            self.options.push((value.into(), display.into()));
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

    fn handle_key_event(&mut self, event: &KeyEvent) {
        match event.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                if self.is_open {
                    self.select_highlighted();
                } else {
                    self.toggle_open();
                }
            }
            KeyCode::Esc if self.is_open => {
                self.is_open = false;
            }
            KeyCode::Up if self.is_open => {
                self.move_highlight_up();
            }
            KeyCode::Down => {
                if self.is_open {
                    self.move_highlight_down();
                } else {
                    self.toggle_open();
                }
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
        if self.required && self.selected.is_none() {
            Err(vec![format!("{} is required", self.label)])
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
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {}
}
