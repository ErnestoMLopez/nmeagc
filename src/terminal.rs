use std::{io, panic};

use color_eyre::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{DefaultTerminal, Terminal, backend::CrosstermBackend};

/// Initializes the terminal interface.
///
/// It enables the raw mode and sets terminal properties.
pub fn init() -> DefaultTerminal {
    set_panic_hook();
    try_init().expect("failed to initialize the terminal")
}

/// Resets the terminal interface.
///
/// This function is also used for the panic hook to revert the terminal properties if unexpected
/// errors occur.
pub fn restore() {
    // There's not much we can do if restoring the terminal fails, so we just print the error
    if let Err(err) = try_restore() {
        std::eprintln!("Failed to restore terminal: {err}");
    }
}

/// Set panic hook to reset the terminal interface on panic.
fn set_panic_hook() {
    let panic_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        restore();
        panic_hook(panic_info);
    }));
}

fn try_init() -> Result<DefaultTerminal> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;
    terminal.clear()?;

    Ok(terminal)
}

fn try_restore() -> Result<()> {
    crossterm::execute!(io::stdout(), DisableMouseCapture, LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
