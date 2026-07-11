use crate::{
    app::{AppEvent, RawNmeaLog, RawNmeaStatus},
    gnss::ParsedMessagedExt,
};

use color_eyre::{Result, eyre::WrapErr};
use crossterm::event::{self, Event as CrosstermEvent};
use nmea_parser::NmeaParser;
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

/// Frequency at which tick events are emitted.
const TICK_FPS: f64 = 30.0;

/// Representation of all possible events.
#[derive(Clone, Debug)]
pub enum Event {
    /// Event emitted on a regular schedule.
    Tick,
    /// Crossterm event emitted by the terminal.
    Crossterm(CrosstermEvent),
    /// Application events.
    App(AppEvent),
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    sender: mpsc::Sender<Event>,
    /// Event receiver channel.
    receiver: mpsc::Receiver<Event>,
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`] and spawns a new thread to handle events.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let tui_handler = EventThread::new(sender.clone());
        let nmea_handler = EventThread::new(sender.clone());
        thread::spawn(|| run_tui_handler(tui_handler));
        thread::spawn(|| run_nmea_handler(nmea_handler));
        Self { sender, receiver }
    }

    /// Receives an event from the sender.
    ///
    /// This function blocks until an event is received.
    ///
    /// # Errors
    ///
    /// This function returns an error if the sender channel is disconnected. This can happen if an
    /// error occurs in the event thread. In practice, this should not happen unless there is a
    /// problem with the underlying terminal.
    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }

    /// Queue an app event to be sent to the event receiver.
    ///
    /// This is useful for sending events to the event handler which will be processed by the next
    /// iteration of the application's event loop.
    pub fn send(&mut self, app_event: AppEvent) {
        let _ = self.sender.send(Event::App(app_event));
    }
}

/// A thread that handles reading crossterm events and emitting tick events on a regular schedule.
struct EventThread {
    /// Event sender channel.
    sender: mpsc::Sender<Event>,
}

impl EventThread {
    /// Constructs a new instance of [`EventThread`].
    fn new(sender: mpsc::Sender<Event>) -> Self {
        Self { sender }
    }

    /// Sends an event to the receiver.
    fn send(&self, event: Event) {
        let _ = self.sender.send(event);
    }
}

/// Runs the terminal events thread.
///
/// This function emits tick events at a fixed rate and polls for crossterm events in between.
fn run_tui_handler(actor: EventThread) -> Result<()> {
    let tick_interval = Duration::from_secs_f64(1.0 / TICK_FPS);
    let mut last_tick = Instant::now();
    loop {
        // Emit tick events at a fixed rate
        let timeout = tick_interval.saturating_sub(last_tick.elapsed());
        if timeout == Duration::ZERO {
            last_tick = Instant::now();
            actor.send(Event::Tick);
        }
        // Poll for crossterm events, ensuring that we don't block the tick interval
        if event::poll(timeout).wrap_err("failed to poll for crossterm events")? {
            let event = event::read().wrap_err("failed to read crossterm event")?;
            actor.send(Event::Crossterm(event));
        }
    }
}

/// Runs the NMEA event thread.
///
/// This function emits NMEA events.
fn run_nmea_handler(actor: EventThread) -> Result<()> {
    let mut parser = NmeaParser::new();

    // TODO: Reemplazar por la conexión y recepción de mensajes
    let sentences = vec![
        "!AIVDM,1,1,,A,H42O55i18tMET00000000000000,2*6D",
        "!AIVDM,1,1,,A,H42O55lti4hhhilD3nink000?050,0*40",
        "$GAGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*56",
    ];

    for sentence in sentences {
        thread::sleep(Duration::from_millis(1000));

        let raw_status: RawNmeaStatus;

        match parser.parse_sentence(sentence) {
            Ok(msg) => {
                raw_status = if msg.is_gnss_sentence() {
                    RawNmeaStatus::Gnss
                } else {
                    RawNmeaStatus::Other
                };
                actor.send(Event::App(AppEvent::NmeaMessage(Box::new(msg))))
            }
            Err(_) => raw_status = RawNmeaStatus::Error,
        }

        let nmea_log = RawNmeaLog {
            sentence: sentence.to_string(),
            status: raw_status,
        };

        let app_event = AppEvent::RawNmeaSentence(nmea_log);
        actor.send(Event::App(app_event));
    }

    Ok(())
}
