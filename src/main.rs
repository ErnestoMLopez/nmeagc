use clap::Parser;

use crate::{app::App, cli::Cli};

pub mod app;
pub mod cli;
pub mod event;
pub mod gnss;
pub mod nmea;
pub mod setup;
pub mod terminal;
pub mod theme;
pub mod ui;
pub mod widgets;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = Cli::parse();
    let terminal = terminal::init();
    let result = App::new(args).run(terminal);
    terminal::restore();
    result
}
