use crate::app::{App, AppTab};
use crate::nmea::{RawNmeaLog, RawNmeaStatus};
use crate::theme::THEME;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    symbols::Marker,
    text::{Line, Text},
    widgets::{
        Bar, BarChart, BarGroup, Block, Paragraph, Tabs,
        canvas::{Canvas, Circle, Map, MapResolution, Points, Rectangle},
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

fn render_monitor_tab(app: &App, frame: &mut Frame, area: Rect) {
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

    let nmea_data = app.nmea_data.lock().expect("mutex poisoned");

    let lines = vec![
        Line::from(format!(
            "Latitude:  {}",
            nmea_data
                .latitude()
                .map_or("-".to_string(), |lat| { format_latitude(lat) })
        )),
        Line::from(format!(
            "Longitude: {}",
            nmea_data
                .longitude()
                .map_or("-".to_string(), |lon| format_longitude(lon))
        )),
        Line::from(format!(
            "Altitude:  {} m (MSL)",
            nmea_data
                .altitude()
                .map(|alt| format!("{:.3}", alt))
                .unwrap_or("-".to_string())
        )),
    ];
    let nav_text = Paragraph::new(lines).block(block_nav);

    // TODO: Reemplazar por la obtención de datos del estado de la app
    // TODO: Modularizar creando widget que genere internamente el gráfico de barras por satélite y por señal de cada satélite
    let data = [
        ('G', 01, 44, true),
        ('G', 01, 32, true),
        ('G', 24, 41, true),
        ('E', 03, 38, true),
        ('E', 36, 31, false),
        ('R', 22, 48, true),
    ];
    let bars = create_bars(&data);
    let barchart = BarChart::default()
        .block(block_cn0)
        .bar_width(3)
        .bar_style(Style::default().fg(Color::Cyan))
        .data(bars)
        .max(55);

    frame.render_widget(nav_text, nav_area);
    frame.render_widget(barchart, cn0_area);
    frame.render_widget(block_graphs, graphs_area);
}

// TODO: Esto es solo un ejemplo basi para después reemplazar por un custom widget
fn create_bars<'a>(data: &[(char, u8, u64, bool)]) -> BarGroup<'a> {
    let bargroup = data
        .iter()
        .map(|(gnss, label, value, is_active)| {
            Bar::default()
                .value(*value)
                .label(format!("{}{}", gnss, label))
                .style(if *is_active {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                })
        })
        .collect::<Vec<Bar>>();
    BarGroup::new(bargroup)
}

fn render_map_tab(_app: &App, frame: &mut Frame, area: Rect) {
    let layout = Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]);
    let [nav_area, map_area] = area.layout(&layout);

    let block = Block::bordered().title("Navigation").style(THEME.borders);

    // TODO: Reemplazar por la obtención de datos de la estructura de estado de la app
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
            ctx.draw(&Points {
                coords: &[(coordinates)],
                color: Color::LightRed,
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

fn render_raw_tab(app: &App, frame: &mut Frame, area: Rect) {
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

    let logs = Text::from(
        app.raw_data
            .iter()
            .map(|log| Line::from(log))
            .collect::<Vec<Line>>(),
    );

    let scroll_offset = (app.raw_data.len() as u16).saturating_sub(raw_area.height - 2);

    let logs_paragraph = Paragraph::new(logs)
        .block(block_raw)
        .left_aligned()
        .scroll((scroll_offset, 0));

    frame.render_widget(block_msg, msg_area);
    frame.render_widget(logs_paragraph, raw_area);
    frame.render_widget(block_search, search_area);
}

impl<'a> From<&RawNmeaLog> for Line<'a> {
    fn from(log: &RawNmeaLog) -> Self {
        let sentence = log.sentence.clone();
        match log.status {
            RawNmeaStatus::Gnss => Line::styled(sentence, Style::new().gray()),
            RawNmeaStatus::Error => Line::styled(sentence, Style::new().red()),
            RawNmeaStatus::Other => Line::styled(sentence, Style::new().light_blue()),
        }
    }
}

// TODO: Mover a un módulo nuevo
fn format_latitude(lat: f64) -> String {
    let direction = if lat >= 0.0 { "N" } else { "S" };
    let abs_lat = lat.abs();
    let degrees = abs_lat.floor() as i32;
    let minutes_raw = (abs_lat - degrees as f64) * 60.0;
    let minutes = minutes_raw.floor() as i32;
    let seconds = (minutes_raw - minutes as f64) * 60.0;

    format!("{}° {}' {:.2}\" {}", degrees, minutes, seconds, direction)
}

fn format_longitude(lon: f64) -> String {
    let direction = if lon >= 0.0 { "E" } else { "W" };
    let abs_lon = lon.abs();
    let degrees = abs_lon.floor() as i32;
    let minutes_raw = (abs_lon - degrees as f64) * 60.0;
    let minutes = minutes_raw.floor() as i32;
    let seconds = (minutes_raw - minutes as f64) * 60.0;

    format!("{}° {}' {:.2}\" {}", degrees, minutes, seconds, direction)
}
