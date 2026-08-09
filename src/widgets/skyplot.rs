use crate::theme::THEME;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    symbols::Marker,
    widgets::{
        Block, Paragraph, StatefulWidget, Widget, Wrap,
        canvas::{Canvas, Circle, Context, Line},
    },
};

pub struct Skyplot {
    pub satellites: Vec<(f64, f64)>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SkyplotState {
    pub plot_area: Rect,
}

impl StatefulWidget for Skyplot {
    type State = SkyplotState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::bordered().title("Skyplot").style(THEME.borders);
        let inner_area = block.inner(area);

        block.render(area, buf);

        state.plot_area = Self::centered_square(inner_area);

        if state.plot_area.width.min(state.plot_area.height) < 5 {
            Paragraph::new("Not enough space")
                .centered()
                .wrap(Wrap { trim: true })
                .render(inner_area, buf);
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
                // self.draw_sky_track(ctx, &self.shared.ground_station.as_ref().unwrap().position);
            });

        skyplot.render(state.plot_area, buf);
    }
}

impl Skyplot {
    pub fn new(satellites: Vec<(f64, f64)>) -> Self {
        Self { satellites }
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

    fn centered_square(area: Rect) -> Rect {
        let width = area.width.min(area.height * 2);
        let height = width / 2;
        Rect {
            x: area.x + (area.width - width) / 2,
            y: area.y + (area.height - height) / 2,
            width,
            height,
        }
    }
}
