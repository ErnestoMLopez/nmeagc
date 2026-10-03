use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct ChoiceInput {
    pub list_state: ListState,
}

impl Default for ChoiceInput {
    fn default() -> Self {
        Self {
            list_state: ListState::default().with_selected(Some(0)),
        }
    }
}
