use std::{io, panic, thread};

use color_eyre::Result;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::{DefaultTerminal, Terminal};

/// Initializes the terminal interface.
///
/// It sets a custom panic hook, enables the raw mode and sets terminal properties.
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

/// Set custom panic hook to reset the terminal interface.
///
/// Only a panic in the main thread will trigger the terminal restoration. Panics in worker threads
/// are caught during unwinding, and send an event to the main thread to do a proper shutdown, which
/// includes the terminal restoration. This is done to avoid restoring the terminal from a worker
/// thread panic and then having the main thread rendering on the restored terminal.
fn set_panic_hook() {
    let main_thread_id = thread::current().id();
    let panic_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        if thread::current().id() == main_thread_id {
            restore();
        }
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
