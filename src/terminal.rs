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
pub fn init() -> Result<DefaultTerminal> {
    set_panic_hook()?;

    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;
    terminal.clear()?;

    Ok(terminal)
}

/// Resets the terminal interface.
///
/// This function is also used for the panic hook to revert the terminal properties if unexpected
/// errors occur.
pub fn reset() -> Result<()> {
    crossterm::execute!(io::stdout(), DisableMouseCapture, LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

/// Set panic hook to reset the terminal interface on panic.
fn set_panic_hook() -> Result<()> {
    let panic_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic| {
        reset().expect("failed to reset the terminal");
        panic_hook(panic);
    }));
    Ok(())
}
