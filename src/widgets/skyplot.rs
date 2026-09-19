use crate::gnss::Gnss;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Style, Stylize},
    symbols::Marker,
    widgets::{
        Block, BlockExt, BorderType, Borders, Cell, Paragraph, Row, Table, Widget, Wrap,
        canvas::{Canvas, Circle, Context, Line, Painter, Shape},
    },
};

pub struct Skyplot<'a> {
    pub satellites: Vec<SkyplotSatellite>,
    block: Option<Block<'a>>,
    style: Style,
    mouse_position: Option<(u16, u16)>,
}

pub struct SkyplotSatellite {
    pub gnss: Gnss,
    pub svid: u8,
    pub elevation: f64,
    pub azimuth: f64,
}

struct PlotableSv {
    color: Color,
    x: f64,
    y: f64,
}

impl PlotableSv {
    fn new(color: Color, x: f64, y: f64) -> Self {
        PlotableSv { color, x, y }
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

                PlotableSv::new(Color::from(satellite.gnss), x, y)
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
                .style(Color::White);
            Widget::render(table, info_area, buf);
        } else {
            Block::new()
                .border_type(BorderType::LightDoubleDashed)
                .borders(Borders::TOP)
                .render(info_area, buf);
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
        ctx.draw(&Line::new(-1.0, 0.0, 1.0, 0.0, Color::DarkGray));
        ctx.draw(&Line::new(0.0, -1.0, 0.0, 1.0, Color::DarkGray));
        ctx.print(0.0, 1.0, "N".green());
        ctx.print(1.0, 0.0, "E".green());
        ctx.print(0.0, -1.0, "S".green());
        ctx.print(-1.0, 0.0, "W".green());
        ctx.print(0.9, -0.01, "0º".green());
        ctx.print(0.6, -0.01, "30º".green());
        ctx.print(0.3, -0.01, "60º".green());
    }

    fn draw_svs(ctx: &mut Context, svs: &[PlotableSv]) {
        ctx.marker(Marker::HalfBlock);
        for sv in svs.iter() {
            ctx.draw(sv);
        }
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

        // Calculate the size of a terminal cell
        let cell_width = 2.0 / plot_area.x as f64;
        let cell_height = 2.0 / plot_area.y as f64;

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
