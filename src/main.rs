use clap::Parser;

use crate::{app::App, cli::Cli};

pub mod app;
pub mod cli;
pub mod event;
pub mod gnss;
pub mod nmea;
pub mod terminal;
pub mod theme;
pub mod ui;
pub mod widgets;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    dbg!(&cli);
    let source = cli.into_data_source();
    dbg!(&source);
    let terminal = terminal::init();
    let result = App::new(source).run(terminal);
    terminal::restore();
    result
}
