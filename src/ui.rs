use crate::app::{App, AppTab};
use crate::theme::THEME;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Paragraph, Tabs},
};
use strum::IntoEnumIterator;

impl App {
    pub fn render(&mut self, frame: &mut Frame) {
        let screen = Block::new().style(THEME.root);
        frame.render_widget(screen, frame.area());

        let layout = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
        let [titlebar_area, tabcontent_area] = frame.area().layout(&layout);

        render_tabs_title(self, frame, titlebar_area);

        match self.tab {
            AppTab::Monitor => render_monitor_tab(self, frame, tabcontent_area),
            AppTab::Map => render_map_tab(self, frame, tabcontent_area),
            AppTab::Raw => render_raw_tab(self, frame, tabcontent_area),
        }
    }
}

fn render_tabs_title(app: &App, frame: &mut Frame, area: Rect) {
    let layout = Layout::horizontal([Constraint::Fill(1), Constraint::Length(6)]);
    let [tabs_area, title_area] = area.layout(&layout);

    let title = Paragraph::new("nmeagc")
        .right_aligned()
        .style(THEME.app_title);
    let tab_titles = AppTab::iter().map(|tab| tab.as_str());
    let tabs = Tabs::new(tab_titles)
        .style(THEME.tabs)
        .highlight_style(THEME.tabs_selected)
        .select(app.tab as usize)
        .divider("")
        .padding(" ", " ");

    frame.render_widget(title, title_area);
    frame.render_widget(tabs, tabs_area);
}

fn render_monitor_tab(_app: &App, _frame: &mut Frame, _area: Rect) {}

fn render_map_tab(_app: &App, _frame: &mut Frame, _area: Rect) {}

fn render_raw_tab(_app: &App, _frame: &mut Frame, _area: Rect) {}
