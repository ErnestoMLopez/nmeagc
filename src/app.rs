use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::TcpStream;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use circular_buffer::FixedCircularBuffer;
use color_eyre::Result;
use color_eyre::eyre::{Context, Report, eyre};
use crossterm::event::{
    Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind,
};
use nmea::{Nmea, SentenceType};
use ratatui::DefaultTerminal;
use strum::{Display, EnumIter, FromRepr};

use crate::cli::{Cli, DataSource};
use crate::event::{Event, EventHandler, EventThread, run_tui_handler};
use crate::gnss::{NavigationData, SvData};
use crate::nmea::{RawNmeaLog, run_nmea_handler};
use crate::setup::{AppSetup, SetupAction};

/// Maximum amount of NMEA sentences to store and render in the raw data tab
const MAX_RAW_NMEA_LOGS: usize = 1000;

/// Application.
pub struct App {
    /// Indicates if the application is running.
    pub running: Arc<AtomicBool>,
    /// Indicates if the client was started in interactive mode for the input/output configuration
    pub interactive_setup: bool,
    /// State for the interactive setup process
    pub setup: AppSetup,
    /// Last recorded mouse position (for hovering detection)
    pub mouse_position: Option<(u16, u16)>,
    /// Event handler.
    pub event_handler: EventHandler,
    /// Current tab.
    pub tab: AppTab,
    /// Last navigation solution available
    pub nav_data: NavigationData,
    /// GNSS data table.
    pub sv_data: Vec<SvData>,
    /// Raw NMEA data logs.
    pub raw_data: FixedCircularBuffer<RawNmeaLog, MAX_RAW_NMEA_LOGS>,
    /// NMEA parser and data (shared between the event handler and the application).
    pub nmea_data: Arc<Mutex<Nmea>>,
    /// Selected NMEA data source.
    input_source: DataSource,
    /// Output file to write validated received messages
    output_file: Option<PathBuf>,
    /// Writer for the output file
    output_writer: Option<BufWriter<File>>,
    /// Handles of each spawned thread for graceful shutdown
    workers: Vec<(&'static str, JoinHandle<Result<()>>)>,
    /// First fatal error reported by a worker thread
    worker_error: Option<Report>,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(args: Cli) -> Self {
        let app = Self {
            running: Arc::new(AtomicBool::new(true)),
            interactive_setup: args.interactive,
            setup: AppSetup::new(),
            mouse_position: None,
            event_handler: EventHandler::new(),
            tab: AppTab::default(),
            nav_data: NavigationData::default(),
            sv_data: Vec::new(),
            raw_data: FixedCircularBuffer::<RawNmeaLog, MAX_RAW_NMEA_LOGS>::new(),
            nmea_data: Arc::new(Mutex::new(Nmea::default())),
            input_source: args.source.unwrap_or_default(),
            output_file: args.output,
            output_writer: None,
            workers: Vec::new(),
            worker_error: None,
        };

        // If interactive mode wasn't invoked, we enqueue a message to setup the reader immediatly
        // after the main loop started. Otherwise, the interactive setup would run and send this
        // message after completion
        if !app.interactive_setup {
            app.event_handler.send(AppEvent::SetupReady);
        }

        app
    }

    /// Performs the application setup to process NMEA data
    ///
    /// Creates an event runner which handles the conection, reception and parsing of NMEA data. The
    /// parsing and event generation is done in a thread loop. If an output file is specified, it
    /// will be opened and the validated NMEA sentences will be written to it.
    pub fn setup(&mut self) -> Result<()> {
        if let (DataSource::File(config), Some(output)) =
            (&self.input_source, self.output_file.as_ref())
            && let (Ok(input), Ok(output)) = (config.path.canonicalize(), output.canonicalize())
            && input == output
        {
            return Err(eyre!(
                "Input and output files must be different: {}",
                input.display()
            ));
        }

        let input: Box<dyn Read + Send> = match &self.input_source {
            DataSource::Tcp(config) => {
                let address = (&*config.host, config.port);
                let stream = TcpStream::connect(address).wrap_err("Error opening TCP stream")?;
                stream
                    .set_read_timeout(Some(config.read_timeout()))
                    .wrap_err("Error setting TCP read timeout")?;
                Box::new(stream)
            }
            DataSource::Serial(config) => {
                let builder = serialport::new(&config.serial_path, config.baudrate)
                    .data_bits(config.data_bits.into())
                    .flow_control(config.flow_control.into())
                    .parity(config.parity.into())
                    .stop_bits(config.stop_bits.into())
                    .timeout(config.read_timeout());
                // Exclusive access is only available on Unix platforms
                #[cfg(unix)]
                let builder = builder.exclusive(config.exclusive.unwrap_or(true));
                let serial_port = builder.open().wrap_err("Error opening serial port")?;
                Box::new(serial_port)
            }
            DataSource::File(config) => {
                Box::new(File::open(&config.path).wrap_err("Error opening file")?)
            }
        };

        if let Some(output) = self.output_file.as_ref() {
            let output_file = File::create(output).wrap_err("Error creating output file")?;
            self.output_writer = Some(BufWriter::new(output_file));
        }

        let reader = BufReader::new(input);
        let parser = Arc::clone(&self.nmea_data);

        self.spawn_worker("NMEA reader", move |actor, running| {
            run_nmea_handler(actor, running, reader, parser)
        });

        Ok(())
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.start();

        let loop_result: Result<()> = (|| {
            while self.running.load(Ordering::Relaxed) {
                terminal.draw(|frame| self.render(frame))?;
                self.handle_events()?;
            }
            Ok(())
        })();

        let shutdown_result = self.shutdown();

        self.worker_error
            .take()
            .map_or(loop_result, Err)
            .and(shutdown_result)
    }

    /// Handles all the events emitted by the event handler.
    fn handle_events(&mut self) -> Result<()> {
        match self.event_handler.next()? {
            Event::Tick => self.tick(),
            Event::Crossterm(event) => match event {
                CrosstermEvent::Key(key_event) => self.handle_key_event(key_event)?,
                CrosstermEvent::Mouse(mouse_event) => self.handle_mouse_event(mouse_event)?,
                _ => {}
            },
            Event::App(app_event) => match app_event {
                AppEvent::Quit => self.quit()?,
                AppEvent::SetupReady => self.setup()?,
                AppEvent::NmeaMessage(msg) => self.handle_nmea_msg(msg)?,
                AppEvent::RawNmeaSentence(raw) => self.handle_raw_nmea(raw)?,
            },
            Event::WorkerError { worker, error } => {
                self.worker_error
                    .get_or_insert_with(|| eyre!("{worker} failed: {error}"));
                self.quit()?;
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        if key_event.kind != crossterm::event::KeyEventKind::Press {
            return Ok(());
        }

        if self.interactive_setup {
            match self.setup.handle_key_event(key_event)? {
                SetupAction::Continue => {}
                SetupAction::Cancel => {
                    self.event_handler.send(AppEvent::Quit);
                }
                SetupAction::Complete(input, output) => {
                    self.interactive_setup = false;
                    self.input_source = input;
                    self.output_file = output;
                    self.event_handler.send(AppEvent::SetupReady)
                }
            }
            return Ok(());
        }

        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.event_handler.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.event_handler.send(AppEvent::Quit)
            }
            KeyCode::Tab => self.tab.next(),
            KeyCode::Right => self.tab.next(),
            KeyCode::Left => self.tab.prev(),
            _ => {}
        }
        Ok(())
    }

    /// Handles the mouse events and updates the state of [`App`].
    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> Result<()> {
        match mouse_event.kind {
            MouseEventKind::Down(_) => {
                // Handle mouse click events here
            }
            MouseEventKind::Up(_) => {
                // Handle mouse release events here
            }
            MouseEventKind::Drag(_) => {
                // Handle mouse drag events here
            }
            MouseEventKind::Moved => {
                self.mouse_position = Some((mouse_event.column, mouse_event.row))
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_nmea_msg(&mut self, nmea_msg: SentenceType) -> Result<()> {
        match nmea_msg {
            SentenceType::GGA => self.update_from_gga(),
            SentenceType::RMC => self.update_from_rmc(),
            SentenceType::GNS => {}
            SentenceType::GSA => {}
            SentenceType::GSV => self.update_from_gsv(),
            SentenceType::GLL => {}
            _ => {}
        }
        Ok(())
    }

    fn handle_raw_nmea(&mut self, raw: RawNmeaLog) -> Result<()> {
        if let Some(writer) = self.output_writer.as_mut() {
            writeln!(writer, "{}", raw.sentence.as_str())
                .wrap_err("Error writing to output file")?;
        }
        self.raw_data.push_back(raw);
        Ok(())
    }

    /// Handles the tick event of the terminal.
    fn tick(&mut self) {}

    /// Set running to false to quit the application and wait for each spawned thread.
    fn quit(&mut self) -> Result<()> {
        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }

    /// Spawns a worker thread that reports its error through the event channel.
    fn spawn_worker<F>(&mut self, name: &'static str, worker: F)
    where
        F: FnOnce(EventThread, Arc<AtomicBool>) -> Result<()> + Send + 'static,
    {
        let actor = self.event_handler.create_actor();
        let error_actor = actor.clone();
        let running = Arc::clone(&self.running);

        let handle = thread::spawn(move || {
            let result = catch_unwind(AssertUnwindSafe(|| worker(actor, running)))
                .unwrap_or_else(|_| Err(eyre!("panicked")));
            if let Err(error) = &result {
                error_actor.send(Event::WorkerError {
                    worker: name,
                    error: format!("{error:#}"),
                });
            }
            result
        });

        self.workers.push((name, handle));
    }

    fn start(&mut self) {
        self.spawn_worker("event handler", |actor, running| {
            run_tui_handler(actor, running)
        });
    }

    fn shutdown(&mut self) -> Result<()> {
        self.running.store(false, Ordering::Relaxed);

        let mut first_error = None;

        if let Some(mut writer) = self.output_writer.take()
            && let Err(error) = writer.flush().wrap_err("Error flushing output file")
        {
            first_error = Some(error);
        }

        for (name, handle) in self.workers.drain(..) {
            match handle.join() {
                Err(_) => {
                    first_error.get_or_insert_with(|| eyre!("{name} thread panicked"));
                }
                Ok(Err(error)) => {
                    first_error.get_or_insert(error);
                }
                Ok(Ok(())) => {}
            }
        }

        first_error.map_or(Ok(()), Err)
    }

    /// Returns the key bindings.
    pub fn get_key_bindings(&self) -> Vec<(&str, &str)> {
        if self.interactive_setup {
            return self.setup.get_key_bindings();
        }

        // TODO: Add key bindings for each tab
        match self.tab {
            AppTab::Monitor => vec![],
            AppTab::Map => vec![],
            AppTab::Raw => vec![],
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

/// Application events.
#[derive(Clone, Debug)]
pub enum AppEvent {
    /// Quit the application.
    Quit,
    /// Application setup finished (via CLI or interactive setup).
    SetupReady,
    /// NMEA message received.
    NmeaMessage(SentenceType),
    /// Raw NMEA sentence for raw data logging.
    RawNmeaSentence(RawNmeaLog),
}

#[derive(Debug, Clone, Copy, Default, Display, EnumIter, FromRepr, PartialEq, Eq)]
pub enum AppTab {
    #[default]
    Monitor,
    Map,
    Raw,
}

impl AppTab {
    fn next(&mut self) {
        let current_index = *self as usize;
        let next_index = current_index.saturating_add(1);
        *self = Self::from_repr(next_index).unwrap_or_default()
    }

    fn prev(&mut self) {
        let current_index = *self as usize;
        let prev_index = current_index.saturating_sub(1);
        *self = Self::from_repr(prev_index).unwrap_or(*self)
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Monitor => " Monitor ",
            Self::Map => "   Map   ",
            Self::Raw => "   Raw   ",
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use clap::Parser;

    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("nmeagc-{}-{name}", std::process::id()))
    }

    fn app_with_paths(input: &Path, output: &Path) -> App {
        let args = Cli::try_parse_from([
            "nmeagc",
            "-f",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .unwrap();

        App::new(args)
    }

    #[test]
    fn setup_rejects_same_input_and_output_path() {
        let path = temp_path("same.nmea");
        fs::write(&path, b"data").unwrap();
        let mut app = app_with_paths(&path, &path);

        assert!(app.setup().is_err());
        assert_eq!(fs::read(&path).unwrap(), b"data");

        fs::remove_file(&path).ok();
    }

    #[test]
    fn setup_accepts_different_input_and_output_paths() {
        let input = temp_path("input.nmea");
        let output = temp_path("output.nmea");
        fs::write(&input, b"data").unwrap();
        fs::remove_file(&output).ok();
        let mut app = app_with_paths(&input, &output);

        assert!(app.setup().is_ok());
        assert_eq!(fs::read(&input).unwrap(), b"data");
        assert!(output.exists());
        app.shutdown().unwrap();

        fs::remove_file(&input).ok();
        fs::remove_file(&output).ok();
    }

    #[test]
    fn shutdown_returns_first_worker_error() {
        let input = temp_path("shutdown-input.nmea");
        let output = temp_path("shutdown-output.nmea");
        fs::write(&input, b"data").unwrap();
        fs::remove_file(&output).ok();
        let mut app = app_with_paths(&input, &output);

        app.workers.push(("ok", thread::spawn(|| Ok(()))));
        app.workers.push((
            "boom",
            thread::spawn(|| -> Result<()> { Err(eyre!("boom")) }),
        ));

        let error = app.shutdown().unwrap_err();
        assert_eq!(error.to_string(), "boom");

        fs::remove_file(&input).ok();
        fs::remove_file(&output).ok();
    }

    #[test]
    fn worker_panic_is_reported() {
        let input = temp_path("panic-input.nmea");
        let output = temp_path("panic-output.nmea");
        fs::write(&input, b"data").unwrap();
        fs::remove_file(&output).ok();
        let mut app = app_with_paths(&input, &output);

        app.spawn_worker("boom", |_, _| panic!("boom"));

        let (worker, error) = loop {
            if let Event::WorkerError { worker, error } = app.event_handler.next().unwrap() {
                break (worker, error);
            }
        };

        assert_eq!(worker, "boom");
        assert_eq!(error, "panicked");

        fs::remove_file(&input).ok();
        fs::remove_file(&output).ok();
    }
}
