use crate::gnss::{GalileoSignal, GlonassSignal, Gnss, GnssSignal, GpsSignal};
use crate::theme::THEME;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Widget},
};
use std::collections::BTreeMap;

struct SignalInfoSet(Vec<SignalInfo>);

pub struct SignalInfo {
    signal: GnssSignal,
    svid: u8,
    cn0: u64,
    is_used: bool,
}

pub struct SignalsMonitor {
    signals: SignalInfoSet,
}

impl SignalInfo {
    pub fn new(signal: GnssSignal, svid: u8, cn0: u64, is_used: bool) -> Self {
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

impl SignalsMonitor {
    pub fn new(signals: Vec<SignalInfo>) -> Self {
        Self {
            signals: SignalInfoSet(signals),
        }
    }
}

impl Widget for SignalsMonitor {
    fn render(self, area: Rect, buf: &mut Buffer) {
        const MAX_CN0: u64 = 55;
        let block_barchart = Block::bordered()
            .title("Signals monitor")
            .style(THEME.borders);
        let barchart = BarChart::grouped(self.signals)
            .block(block_barchart)
            .bar_style(Style::default().fg(Color::Cyan))
            .bar_width(4)
            .bar_gap(1)
            .group_gap(1)
            .max(MAX_CN0);

        barchart.render(area, buf);
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
            .value(signal_info.cn0)
            .label(signal_info.signal.as_signal_code_str())
            .style(if signal_info.is_used {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            })
    }
}
