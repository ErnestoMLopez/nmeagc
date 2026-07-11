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
        "$GPGGA,172400.00,3455.1280,S,05757.7830,W,1,08,0.9,25.0,M,10.0,M,,*54",
        "$GPRMC,172400.00,A,3455.1280,S,05757.7830,W,60.0,180.0,110726,,,A*6D",
        "$GNGNS,172400.00,3455.1280,S,05757.7830,W,AA,08,0.9,25.0,10.0,,0000*60",
        "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
        "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
        "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
        "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
        "$GPGLL,3455.1280,S,05757.7830,W,172400.00,A,A*66",
        "$GPGGA,172401.00,3455.1340,S,05757.7860,W,1,08,0.9,25.0,M,10.0,M,,*5D",
        "$GPRMC,172401.00,A,3455.1340,S,05757.7860,W,60.0,180.0,110726,,,A*64",
        "$GNGNS,172401.00,3455.1340,S,05757.7860,W,AA,08,0.9,25.0,10.0,,0000*69",
        "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
        "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
        "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
        "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
        "$GPGLL,3455.1340,S,05757.7860,W,172401.00,A,A*6F",
        "$GPGGA,172402.00,3455.1400,S,05757.7890,W,1,08,0.9,25.0,M,10.0,M,,*52",
        "$GPRMC,172402.00,A,3455.1400,S,05757.7890,W,60.0,180.0,110726,,,A*6B",
        "$GNGNS,172402.00,3455.1400,S,05757.7890,W,AA,08,0.9,25.0,10.0,,0000*66",
        "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
        "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
        "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
        "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
        "$GPGLL,3455.1400,S,05757.7890,W,172402.00,A,A*60",
        "$GPGGA,172403.00,3455.1460,S,05757.7920,W,1,08,0.9,25.0,M,10.0,M,,*5F",
        "$GPRMC,172403.00,A,3455.1460,S,05757.7920,W,60.0,180.0,110726,,,A*66",
        "$GNGNS,172403.00,3455.1460,S,05757.7920,W,AA,08,0.9,25.0,10.0,,0000*6B",
        "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
        "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
        "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
        "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
        "$GPGLL,3455.1460,S,05757.7920,W,172403.00,A,A*6D",
        "$GPGGA,172404.00,3455.1520,S,05757.7950,W,1,08,0.9,25.0,M,10.0,M,,*5A",
        "$GPRMC,172404.00,A,3455.1520,S,05757.7950,W,60.0,180.0,110726,,,A*63",
        "$GNGNS,172404.00,3455.1520,S,05757.7950,W,AA,08,0.9,25.0,10.0,,0000*6E",
        "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
        "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
        "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
        "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
        "$GPGLL,3455.1520,S,05757.7950,W,172404.00,A,A*68",
        "$GPRMC,172404.00,A,3455.1500,S,05757.5900,W,60.0,180.0,110726,,,A*00",
        "$GPGGA,172405.00,3455.1580,S,05757.7980,W,1,08,0.9,25.0,M,10.0,M,,*5C",
        "$GPRMC,172405.00,A,3455.1580,S,05757.7980,W,60.0,180.0,110726,,,A*65",
        "$GNGNS,172405.00,3455.1580,S,05757.7980,W,AA,08,0.9,25.0,10.0,,0000*68",
        "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
        "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
        "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
        "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
        "$GPGLL,3455.1580,S,05757.7980,W,172405.00,A,A*6E",
        "$GPGGA,172406.00,3455.1640,S,05757.8010,W,1,08,0.9,25.0,M,10.0,M,,*5F",
        "$GPRMC,172406.00,A,3455.1640,S,05757.8010,W,60.0,180.0,110726,,,A*66",
        "$GNGNS,172406.00,3455.1640,S,05757.8010,W,AA,08,0.9,25.0,10.0,,0000*6B",
        "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
        "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
        "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
        "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
        "$GPGLL,3455.1640,S,05757.8010,W,172406.00,A,A*6D",
        "$GPRMC,172406.00,A,3455.1500,S,05757.5900,W,60.0,180.0,110726,,,A*00",
        "$GPGGA,172407.00,3455.1700,S,05757.8040,W,1,08,0.9,25.0,M,10.0,M,,*5E",
        "$GPRMC,172407.00,A,3455.1700,S,05757.8040,W,60.0,180.0,110726,,,A*67",
        "$GNGNS,172407.00,3455.1700,S,05757.8040,W,AA,08,0.9,25.0,10.0,,0000*6A",
        "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
        "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
        "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
        "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
        "$GPGLL,3455.1700,S,05757.8040,W,172407.00,A,A*6C",
        "$GPGGA,172408.00,3455.1760,S,05757.8070,W,1,08,0.9,25.0,M,10.0,M,,*54",
        "$GPRMC,172408.00,A,3455.1760,S,05757.8070,W,60.0,180.0,110726,,,A*6D",
        "$GNGNS,172408.00,3455.1760,S,05757.8070,W,AA,08,0.9,25.0,10.0,,0000*60",
        "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
        "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
        "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
        "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
        "$GPGLL,3455.1760,S,05757.8070,W,172408.00,A,A*66",
        "$GPRMC,172408.00,A,3455.1500,S,05757.5900,W,60.0,180.0,110726,,,A*00",
        "$GPGGA,172409.00,3455.1820,S,05757.8100,W,1,08,0.9,25.0,M,10.0,M,,*58",
        "$GPRMC,172409.00,A,3455.1820,S,05757.8100,W,60.0,180.0,110726,,,A*61",
        "$GNGNS,172409.00,3455.1820,S,05757.8100,W,AA,08,0.9,25.0,10.0,,0000*6C",
        "$GPGSA,A,3,02,05,09,12,17,19,23,28,,,,,1.5,0.9,1.2*34",
        "$GPGSV,3,1,10,2,45,120,45,5,30,250,40,9,60,300,42,12,15,180,35*4C",
        "$GPGSV,3,2,10,17,25,45,38,19,50,210,44,23,10,315,30,28,40,90,41*7A",
        "$GPGSV,3,3,10,31,20,270,39,32,35,150,37*70",
        "$GPGLL,3455.1820,S,05757.8100,W,172409.00,A,A*6A",
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
