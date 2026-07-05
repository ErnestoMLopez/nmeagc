use crate::app::{App, AppTab};
use crate::theme::THEME;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Color,
    symbols::Marker,
    widgets::{
        Block, Paragraph, Tabs,
        canvas::{Canvas, Circle, Map, MapResolution, Rectangle},
    },
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
        .divider("");

    frame.render_widget(title, title_area);
    frame.render_widget(tabs, tabs_area);
}

fn render_monitor_tab(_app: &App, frame: &mut Frame, area: Rect) {
    let layout = Layout::horizontal([Constraint::Length(35), Constraint::Fill(1)]);
    let [left_area, right_area] = area.layout(&layout);
    let layout = Layout::vertical([Constraint::Max(12), Constraint::Fill(1)]);
    let [upper_area, lower_area] = right_area.layout(&layout);

    let nav_area = left_area;
    let cn0_area = upper_area;
    let graphs_area = lower_area;

    let block_nav = Block::bordered().title("Navigation").style(THEME.borders);
    let block_cn0 = Block::bordered().title("Tracking").style(THEME.borders);
    let block_graphs = Block::bordered().title("Graphs").style(THEME.borders);

    frame.render_widget(block_nav, nav_area);
    frame.render_widget(block_cn0, cn0_area);
    frame.render_widget(block_graphs, graphs_area);
}

fn render_map_tab(_app: &App, frame: &mut Frame, area: Rect) {
    let layout = Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]);
    let [nav_area, map_area] = area.layout(&layout);

    let block = Block::bordered().title("Navigation").style(THEME.borders);

    let map = Canvas::default()
        .block(Block::bordered().title("Worldmap").style(THEME.borders))
        .background_color(THEME.root.bg.unwrap_or(Color::Reset))
        .marker(Marker::Braille)
        .x_bounds([-180.0, 180.0])
        .y_bounds([-90.0, 90.0])
        .paint(|ctx| {
            let coordinates = (-57.942, -34.906);
            ctx.draw(&Map {
                color: Color::White,
                resolution: MapResolution::High,
            });
            ctx.layer();
            ctx.draw(&Rectangle {
                x: coordinates.0,
                y: coordinates.1,
                width: 1.0,
                height: 1.0,
                color: Color::Yellow,
            });
            ctx.draw(&Circle {
                x: coordinates.0,
                y: coordinates.1,
                radius: 10.0,
                color: Color::Green,
            });
        });

    frame.render_widget(block, nav_area);
    frame.render_widget(map, map_area);
}

fn render_raw_tab(_app: &App, frame: &mut Frame, area: Rect) {
    let layout = Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]);
    let [left_area, right_area] = area.layout(&layout);
    let layout = Layout::vertical([Constraint::Min(0), Constraint::Max(3)]);
    let [upper_area, lower_area] = right_area.layout(&layout);

    let msg_area = left_area;
    let raw_area = upper_area;
    let search_area = lower_area;

    let block_msg = Block::bordered().title("Messages").style(THEME.borders);
    let block_raw = Block::bordered().title("Raw data").style(THEME.borders);
    let block_search = Block::bordered().title("Search").style(THEME.borders);

    frame.render_widget(block_msg, msg_area);
    frame.render_widget(block_raw, raw_area);
    frame.render_widget(block_search, search_area);
}
