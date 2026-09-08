use crate::gnss::Gnss;
use crate::theme::THEME;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    symbols::Marker,
    widgets::{
        Block, BlockExt, Paragraph, StatefulWidget, Widget, Wrap,
        canvas::{Canvas, Circle, Context, Line, Shape},
    },
};

pub struct Skyplot<'a> {
    pub satellites: Vec<SkyplotSatellite>,
    block: Option<Block<'a>>,
}

pub struct SkyplotSatellite {
    pub gnss: Gnss,
    pub elevation: f64,
    pub azimuth: f64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SkyplotState {
    pub plot_area: Rect,
}

impl<'a> StatefulWidget for Skyplot<'a> {
    type State = SkyplotState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let widget_area = self.block.inner_if_some(area);

        self.block.as_ref().render(area, buf);

        state.plot_area = Self::top_centered_square(widget_area);

        if state.plot_area.width.min(state.plot_area.height) < 5 {
            Paragraph::new("Not enough space")
                .centered()
                .wrap(Wrap { trim: true })
                .render(widget_area, buf);
            return;
        }

        let skyplot = Canvas::default()
            .background_color(THEME.root.bg.unwrap_or(Color::Reset))
            .marker(Marker::Braille)
            .x_bounds([-1.0, 1.0])
            .y_bounds([-1.0, 1.0])
            .paint(|ctx| {
                Self::draw_grid(ctx);
                ctx.layer();
                self.draw_satellites(ctx);
            });

        skyplot.render(state.plot_area, buf);
    }
}

impl<'a> Skyplot<'a> {
    pub fn new(satellites: Vec<SkyplotSatellite>) -> Self {
        Self {
            block: None,
            satellites,
        }
    }

    /// Surrounds the [`Skyplot`] widget with a [`Block`].
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
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
    }

    fn draw_satellites(&self, ctx: &mut Context) {
        ctx.marker(Marker::HalfBlock);
        for sv in self.satellites.iter() {
            ctx.draw(sv);
        }
    }

    fn top_centered_square(area: Rect) -> Rect {
        let width = area.width.min(area.height * 2);
        let height = width / 2;
        Rect {
            x: area.x + (area.width - width) / 2,
            y: area.y,
            width,
            height,
        }
    }
}

impl Shape for SkyplotSatellite {
    fn draw(&self, painter: &mut ratatui::widgets::canvas::Painter) {
        let elevation_rad = self.elevation.to_radians();
        let azimuth_rad = self.azimuth.to_radians();

        let radius = 1.0 - (elevation_rad / std::f64::consts::FRAC_PI_2);
        let x = radius * azimuth_rad.sin();
        let y = radius * azimuth_rad.cos();

        if let Some((x, y)) = painter.get_point(x, y) {
            painter.paint(x, y, Color::from(self.gnss));
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
