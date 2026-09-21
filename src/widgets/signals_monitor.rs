use crate::gnss::{GalileoSignal, GlonassSignal, Gnss, GnssSignal, GpsSignal};

use std::collections::{BTreeMap, HashMap};

use ratatui::text::Text;
use ratatui::widgets::{BorderType, Borders, Paragraph};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, BlockExt, Widget},
};

struct SignalInfoSet(Vec<SignalInfo>);

pub struct SignalInfo {
    signal: GnssSignal,
    svid: u8,
    cn0: u8,
    is_used: bool,
}

pub struct SignalsMonitor<'a> {
    signals: SignalInfoSet,
    block: Option<Block<'a>>,
    style: Style,
}

impl SignalInfo {
    pub fn new(signal: GnssSignal, svid: u8, cn0: u8, is_used: bool) -> Self {
        Self {
            signal,
            svid,
            cn0,
            is_used,
        }
    }

    pub fn dummy() -> Vec<Self> {
        vec![
            SignalInfo::new(GnssSignal::Gps(GpsSignal::L1CA), 1, 44, true),
            SignalInfo::new(GnssSignal::Gps(GpsSignal::L2C), 1, 41, true),
            SignalInfo::new(GnssSignal::Galileo(GalileoSignal::E1BC), 5, 29, false),
            SignalInfo::new(GnssSignal::Galileo(GalileoSignal::E5AIQ), 5, 30, false),
            SignalInfo::new(GnssSignal::Gps(GpsSignal::L1CA), 3, 48, true),
            SignalInfo::new(GnssSignal::Gps(GpsSignal::L1CA), 28, 39, true),
            SignalInfo::new(GnssSignal::Gps(GpsSignal::L2C), 28, 39, true),
            SignalInfo::new(GnssSignal::Gps(GpsSignal::L1C), 28, 32, false),
            SignalInfo::new(GnssSignal::Gps(GpsSignal::L5IQ), 28, 30, false),
            SignalInfo::new(GnssSignal::Gps(GpsSignal::L1CA), 24, 41, true),
            SignalInfo::new(GnssSignal::Galileo(GalileoSignal::E1BC), 3, 38, true),
            SignalInfo::new(GnssSignal::Galileo(GalileoSignal::E1BC), 36, 31, false),
            SignalInfo::new(GnssSignal::Glonass(GlonassSignal::L1OF), 22, 48, true),
        ]
    }
}

impl<'a> SignalsMonitor<'a> {
    /// Creates a new [`SignalsMonitor`] widget that displays signals tracking info as a
    /// [`BarChart`] with an additional summary panel
    pub fn new(signals: Vec<SignalInfo>) -> Self {
        Self {
            signals: SignalInfoSet(signals),
            block: None,
            style: Style::default(),
        }
    }

    /// Surrounds the [`SignalsMonitor`] widget with a [`Block`].
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    fn split_area(area: Rect) -> (Rect, Rect) {
        let layout = Layout::horizontal([Constraint::Fill(1), Constraint::Length(25)]);
        let [left, right] = area.layout(&layout);

        (left.inner(Margin::new(1, 0)), right)
    }

    fn count_satellites(&self) -> (usize, usize) {
        let mut svs: HashMap<(Gnss, u8), bool> = HashMap::new();

        self.signals.0.iter().for_each(|signal| {
            svs.entry((Gnss::from(signal.signal), signal.svid))
                .or_insert(signal.is_used);
        });

        let tracked = svs.len();
        let used = svs.values().filter(|&used| *used).count();

        (tracked, used)
    }

    fn count_signals(&self) -> (usize, usize) {
        let tracked = self.signals.0.len();
        let used = self
            .signals
            .0
            .iter()
            .filter(|signal| signal.is_used)
            .count();

        (tracked, used)
    }
}

impl<'a> Widget for SignalsMonitor<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        const MAX_CN0: u64 = 55;
        let widget_area = self.block.inner_if_some(area);

        self.block.as_ref().render(area, buf);

        let (barchart_area, info_area) = Self::split_area(widget_area);

        let (svs_tracked, svs_used) = self.count_satellites();
        let (signals_tracked, signals_used) = self.count_signals();

        let barchart = BarChart::grouped(self.signals)
            .bar_style(Style::default().fg(Color::Cyan))
            .bar_width(4)
            .bar_gap(1)
            .group_gap(1)
            .max(MAX_CN0);

        let info_panel = Paragraph::new(Text::from(vec![
            Line::from(format!(" Satellites tracked: {:2} ", svs_tracked)),
            Line::from(format!(" Satellites used:    {:2} ", svs_used)),
            Line::default(),
            Line::from(format!(" Signals tracked:    {:2} ", signals_tracked)),
            Line::from(format!(" Signals used:       {:2} ", signals_used)),
        ]))
        .block(
            Block::new()
                .border_type(BorderType::LightDoubleDashed)
                .borders(Borders::LEFT),
        );

        barchart.render(barchart_area, buf);
        info_panel.render(info_area, buf);
    }
}

impl<'a> Into<Vec<BarGroup<'a>>> for SignalInfoSet {
    fn into(self) -> Vec<BarGroup<'a>> {
        let mut bars_per_sv: BTreeMap<(Gnss, u8), Vec<Bar>> = BTreeMap::new();

        for item in self.0 {
            bars_per_sv
                .entry((Gnss::from(item.signal), item.svid))
                .or_insert(vec![])
                .push(Bar::from(item));
        }

        bars_per_sv
            .into_iter()
            .map(|(sv, signal_bars)| {
                BarGroup::with_label(
                    Line::from(format!("{}{:02}", sv.0.as_char(), sv.1))
                        .centered()
                        .style(Style::default().bold()),
                    signal_bars,
                )
            })
            .collect()
    }
}

impl<'a> From<SignalInfo> for Bar<'a> {
    fn from(signal_info: SignalInfo) -> Self {
        Bar::default()
            .value(signal_info.cn0 as u64)
            .label(signal_info.signal.as_signal_code_str())
            .style(if signal_info.is_used {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            })
    }
}
