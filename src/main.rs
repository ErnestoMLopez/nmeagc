use crate::app::App;

pub mod app;
pub mod event;
pub mod gnss;
pub mod nmea;
pub mod theme;
pub mod ui;
pub mod widgets;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}
