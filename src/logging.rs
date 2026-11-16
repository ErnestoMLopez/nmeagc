use color_eyre::Result;
use tui_logger::{LevelFilter, TuiLoggerFile};

const LOG_FILE: &str = "log.txt";

pub fn init() -> Result<()> {
    tui_logger::init_logger(LevelFilter::Trace)?;
    #[cfg(debug_assertions)]
    tui_logger::set_default_level(LevelFilter::Debug);
    tui_logger::set_log_file(TuiLoggerFile::new(LOG_FILE));
    Ok(())
}
