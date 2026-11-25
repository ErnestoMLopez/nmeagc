use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::canvas::{Canvas, Circle, Context, Line as CanvasLine, Painter, Shape};
use ratatui::widgets::{
    Block, BlockExt, BorderType, Borders, Cell, Paragraph, Row, Table, Widget, Wrap,
};
use strum::{EnumIter, IntoEnumIterator};

use crate::gnss::{Gnss, SvData};

pub struct Skyplot<'a> {
    satellites: Vec<SkyplotSatellite>,
    block: Option<Block<'a>>,
    style: Style,
    mouse_position: Option<(u16, u16)>,
}

pub struct SkyplotSatellite {
    gnss: Gnss,
    svid: u8,
    elevation: f64,
    azimuth: f64,
    state: SatelliteState,
}

impl TryFrom<&SvData> for SkyplotSatellite {
    type Error = ();

    fn try_from(sv: &SvData) -> Result<Self, Self::Error> {
        let elevation = sv.elevation.ok_or(())?;
        let azimuth = sv.azimuth.ok_or(())?;

        Ok(Self {
            gnss: sv.gnss,
            svid: sv.svid,
            elevation: elevation as f64,
            azimuth: azimuth as f64,
            state: SatelliteState::from(sv),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
enum SatelliteState {
    /// The satellite is in view, but no signal is being tracked.
    Visible,
    /// At least one signal is tracked, but none is used in the navigation solution.
    Tracked,
    /// At least one tracked signal is used in the navigation solution.
    Used,
}

impl SatelliteState {
    const fn marker(self) -> Marker {
        match self {
            Self::Visible => Marker::Dot,
            Self::Tracked => Marker::Custom('o'),
            Self::Used => Marker::HalfBlock,
        }
    }

    const fn glyph(self) -> &'static str {
        match self {
            Self::Visible => "•",
            Self::Tracked => "o",
            Self::Used => "▄",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Visible => "Visible",
            Self::Tracked => "Tracked",
            Self::Used => "Used",
        }
    }
}

impl From<&SvData> for SatelliteState {
    fn from(sv: &SvData) -> Self {
        if sv.signals().iter().any(|signal| signal.is_used) {
            Self::Used
        } else if !sv.signals().is_empty() {
            Self::Tracked
        } else {
            Self::Visible
        }
    }
}

struct PlotableSv {
    color: Color,
    x: f64,
    y: f64,
    state: SatelliteState,
}

impl PlotableSv {
    fn new(color: Color, x: f64, y: f64, state: SatelliteState) -> Self {
        PlotableSv { color, x, y, state }
    }
}

impl<'a> Widget for Skyplot<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let widget_area = self.block.inner_if_some(area);

        self.block.as_ref().render(area, buf);

        let (plot_area, info_area) = Self::split_area(widget_area);

        if plot_area.width.min(plot_area.height) < 5 {
            Paragraph::new("Not enough space")
                .centered()
                .wrap(Wrap { trim: true })
                .render(widget_area, buf);
            return;
        }

        // Convert satellites to the plotable elements representing them. This step allows detection
        // of mouse hovering to display its info.
        let svs: Vec<_> = self
            .satellites
            .iter()
            .map(|satellite| {
                let elevation_rad = satellite.elevation.to_radians();
                let azimuth_rad = satellite.azimuth.to_radians();
                let radius = 1.0 - (elevation_rad / std::f64::consts::FRAC_PI_2);
                let x = radius * azimuth_rad.sin();
                let y = radius * azimuth_rad.cos();

                PlotableSv::new(Color::from(satellite.gnss), x, y, satellite.state)
            })
            .collect();

        let skyplot = Canvas::default()
            .background_color(self.style.bg.unwrap_or(Color::Reset))
            .marker(Marker::Braille)
            .x_bounds([-1.0, 1.0])
            .y_bounds([-1.0, 1.0])
            .paint(|ctx| {
                Self::draw_grid(ctx);
                ctx.layer();
                Self::draw_svs(ctx, &svs);
            });

        skyplot.render(plot_area, buf);

        // If we enabled mouse support print satellite info if a satellite is hovered
        if let Some(satellite) = self.detect_hovered_satellite(&svs, plot_area) {
            let rows = [
                Row::new([
                    Cell::from(format!("GNSS: {}", satellite.gnss.as_str())),
                    Cell::from(format!("Elevation: {}º", satellite.elevation)),
                ]),
                Row::new([
                    Cell::from(format!("SVID: {:02}", satellite.svid)),
                    Cell::from(format!("Azimuth:   {}º", satellite.azimuth)),
                ]),
            ];
            let widths = [Constraint::Percentage(40), Constraint::Percentage(60)];
            let table = Table::new(rows, widths)
                .block(
                    Block::new()
                        .border_type(BorderType::LightDoubleDashed)
                        .borders(Borders::TOP),
                )
                .column_spacing(1)
                .style(self.style);
            Widget::render(table, info_area, buf);
        } else {
            self.render_legend(info_area, buf);
        }
    }
}

impl<'a> Skyplot<'a> {
    pub fn new(satellites: Vec<SkyplotSatellite>) -> Self {
        Self {
            satellites,
            block: None,
            style: Style::default(),
            mouse_position: None,
        }
    }

    /// Surrounds the [`Skyplot`] widget with a [`Block`].
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

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn with_hover(mut self, mouse_position: Option<(u16, u16)>) -> Self {
        self.mouse_position = mouse_position;
        self
    }

    fn detect_hovered_satellite(
        &self,
        svs: &[PlotableSv],
        plot_area: Rect,
    ) -> Option<&SkyplotSatellite> {
        let ((mouse_x, mouse_y), threshold) = self.get_mouse_position_and_radius(plot_area)?;

        svs.iter()
            .enumerate()
            .map(|(index, sv)| {
                let distance = (sv.x - mouse_x).hypot(sv.y - mouse_y);
                (index, distance)
            })
            .filter(|(_, distance)| *distance <= threshold)
            .min_by(|(_, left), (_, right)| left.total_cmp(right))
            .and_then(|(index, _)| self.satellites.get(index))
    }

    fn draw_grid(ctx: &mut Context) {
        for radius in [1.0, 0.67, 0.33] {
            ctx.draw(&Circle::new(0.0, 0.0, radius, Color::DarkGray));
        }
        ctx.draw(&CanvasLine::new(-1.0, 0.0, 1.0, 0.0, Color::DarkGray));
        ctx.draw(&CanvasLine::new(0.0, -1.0, 0.0, 1.0, Color::DarkGray));
        ctx.print(0.0, 1.0, "N".green());
        ctx.print(1.0, 0.0, "E".green());
        ctx.print(0.0, -1.0, "S".green());
        ctx.print(-1.0, 0.0, "W".green());
        ctx.print(0.9, -0.01, "0º".green());
        ctx.print(0.6, -0.01, "30º".green());
        ctx.print(0.3, -0.01, "60º".green());
    }

    fn draw_svs(ctx: &mut Context, svs: &[PlotableSv]) {
        for state in SatelliteState::iter() {
            let svs_per_state = svs.iter().filter(|sv| sv.state == state);

            ctx.marker(state.marker());

            for sv in svs_per_state {
                ctx.draw(sv);
            }
        }
    }

    fn render_legend(&self, area: Rect, buf: &mut Buffer) {
        let states = Line::from_iter(SatelliteState::iter().enumerate().flat_map(
            |(index, state)| {
                let separator = (index > 0).then_some(Span::raw("    "));
                let symbol = Span::styled(state.glyph(), self.style);
                let label = Span::styled(format!(" {}", state.label()), self.style);

                [separator, Some(symbol), Some(label)].into_iter().flatten()
            },
        ))
        .centered();

        let constellations = Line::from_iter(Gnss::iter().enumerate().flat_map(|(index, gnss)| {
            let separator = (index > 0).then_some(Span::raw("    "));
            let symbol = Span::styled("■", self.style.fg(Color::from(gnss)));
            let label = Span::styled(format!(" {}", gnss.as_code_str()), self.style);

            [separator, Some(symbol), Some(label)].into_iter().flatten()
        }))
        .centered();

        let legend = Paragraph::new(Text::from(vec![constellations, states]))
            .block(
                Block::new()
                    .border_type(BorderType::LightDoubleDashed)
                    .borders(Borders::TOP),
            )
            .style(self.style);

        legend.render(area, buf);
    }

    fn split_area(area: Rect) -> (Rect, Rect) {
        let layout = Layout::vertical([Constraint::Fill(1), Constraint::Max(3)]);
        let [top, bottom] = area.layout(&layout);

        let plot_height = top.height.min(top.width / 2);
        let plot_width = plot_height * 2;

        let top = top.centered(
            Constraint::Length(plot_width),
            Constraint::Length(plot_height),
        );

        (top, bottom)
    }

    /// Calculates the mouse position in the [`Canvas`] coordinates and a threshold radius for mouse
    /// hovering detection
    fn get_mouse_position_and_radius(&self, plot_area: Rect) -> Option<((f64, f64), f64)> {
        let mouse = Position::from(self.mouse_position?);

        if !plot_area.contains(mouse) {
            return None;
        }

        // Calculate the size of a terminal cell in Canvas units
        let cell_width = 2.0 / plot_area.width as f64;
        let cell_height = 2.0 / plot_area.height as f64;

        // Define a detection radius based on the distance from the cell's center to a vertex of the
        // cell, with a margin for a more relaxed feeling while hovering
        let threshold = 1.5 * cell_height.hypot(cell_width) / 2.0;

        // Normalize terminal coordinates (cell center point) relative to canvas origin
        let canvas_cell_x = (mouse.x - plot_area.x) as f64 + cell_width;
        let canvas_cell_y = (mouse.y - plot_area.y) as f64 + cell_height;

        // Interpolate to Canvas bounds, inverting Y coordinate due to the Canvas reference frame
        let canvas_x = -1.0 + (canvas_cell_x / plot_area.width as f64) * 2.0;
        let canvas_y = 1.0 - (canvas_cell_y / plot_area.height as f64) * 2.0;

        Some(((canvas_x, canvas_y), threshold))
    }
}

impl Shape for PlotableSv {
    fn draw(&self, painter: &mut Painter) {
        if let Some((x, y)) = painter.get_point(self.x, self.y) {
            painter.paint(x, y, self.color);
        }
    }
}

impl From<Gnss> for Color {
    fn from(gnss: Gnss) -> Self {
        match gnss {
            Gnss::Gps => Color::Cyan,
            Gnss::Galileo => Color::Blue,
            Gnss::Glonass => Color::Red,
            Gnss::Beidou => Color::Yellow,
            Gnss::Other => Color::Gray,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gnss::{GnssSignal, GpsSignal};

    fn sv() -> SvData {
        SvData::new(Gnss::Gps, 2)
    }

    #[test]
    fn satellite_state_is_visible_without_tracked_signals() {
        assert_eq!(SatelliteState::from(&sv()), SatelliteState::Visible);
    }

    #[test]
    fn satellite_state_is_tracked_when_no_signal_is_used() {
        let mut sv = sv();
        sv.update_signal(GnssSignal::Gps(GpsSignal::L1CA), Some(45.0), Some(false));

        assert_eq!(SatelliteState::from(&sv), SatelliteState::Tracked);
    }

    #[test]
    fn satellite_state_is_used_when_any_signal_is_used() {
        let mut sv = sv();
        sv.update_signal(GnssSignal::Gps(GpsSignal::L1CA), Some(45.0), Some(false));
        sv.update_signal(GnssSignal::Gps(GpsSignal::L5I), Some(30.0), Some(true));

        assert_eq!(SatelliteState::from(&sv), SatelliteState::Used);
    }

    #[test]
    fn skyplot_satellite_requires_position() {
        assert!(SkyplotSatellite::try_from(&sv()).is_err());
    }

    #[test]
    fn skyplot_satellite_is_built_from_sv_data() {
        let mut sv = sv();
        sv.update_geometry(Some(45.0), Some(120.0));
        sv.update_signal(GnssSignal::Gps(GpsSignal::L1CA), Some(45.0), Some(true));

        let satellite = SkyplotSatellite::try_from(&sv).unwrap();

        assert_eq!(satellite.gnss, Gnss::Gps);
        assert_eq!(satellite.svid, 2);
        assert_eq!(satellite.elevation, 45.0);
        assert_eq!(satellite.azimuth, 120.0);
        assert_eq!(satellite.state, SatelliteState::Used);
    }
}
