use crate::gnss::{GalileoSignal, GlonassSignal, Gnss, GnssSignal, GpsSignal};
use crate::theme::THEME;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Widget},
};

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
            SignalInfo::new(GnssSignal::Gps(GpsSignal::L1CA), 28, 39, true),
            SignalInfo::new(GnssSignal::Gps(GpsSignal::L2C), 28, 39, true),
            SignalInfo::new(GnssSignal::Gps(GpsSignal::L1CA), 28, 32, false),
            SignalInfo::new(GnssSignal::Gps(GpsSignal::L2C), 28, 30, false),
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
            .bar_width(3)
            .bar_gap(0)
            .group_gap(1)
            .max(MAX_CN0);

        barchart.render(area, buf);
    }
}

impl<'a> Into<Vec<BarGroup<'a>>> for SignalInfoSet {
    fn into(self) -> Vec<BarGroup<'a>> {
        let gps_signals = self
            .0
            .iter()
            .filter(|s| matches!(s.signal, GnssSignal::Gps(_)))
            .map(|s| {
                let gnss: Gnss = s.signal.into();
                Bar::default()
                    .value(s.cn0)
                    .label(format!("{}{:02}", gnss.as_char(), s.svid))
                    .style(if s.is_used {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Red)
                    })
            })
            .collect::<Vec<Bar>>();

        let glonass_signals = self
            .0
            .iter()
            .filter(|s| matches!(s.signal, GnssSignal::Glonass(_)))
            .map(|s| {
                let gnss: Gnss = s.signal.into();
                Bar::default()
                    .value(s.cn0)
                    .label(format!("{}{:02}", gnss.as_char(), s.svid))
                    .style(if s.is_used {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Red)
                    })
            })
            .collect::<Vec<Bar>>();

        let galileo_signals = self
            .0
            .iter()
            .filter(|s| matches!(s.signal, GnssSignal::Galileo(_)))
            .map(|s| {
                let gnss: Gnss = s.signal.into();
                Bar::default()
                    .value(s.cn0)
                    .label(format!("{}{:02}", gnss.as_char(), s.svid))
                    .style(if s.is_used {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Red)
                    })
            })
            .collect::<Vec<Bar>>();

        vec![
            BarGroup::with_label(Line::from("GPS").centered(), gps_signals),
            BarGroup::with_label(Line::from("GLO").centered(), glonass_signals),
            BarGroup::with_label(Line::from("GAL").centered(), galileo_signals),
        ]
    }
}
