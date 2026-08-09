use crate::event::{Event, EventHandler};
use crate::gnss::{NavigationData, SvData};
use crate::nmea::RawNmeaLog;
use crate::widgets::skyplot::SkyplotState;

use std::sync::{Arc, Mutex};

use circular_buffer::FixedCircularBuffer;
use color_eyre::Result;
use crossterm::event::{
    Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind,
};
use nmea::{Nmea, SentenceType};
use ratatui::DefaultTerminal;
use strum::{Display, EnumIter, FromRepr};

/// Maximum amount of NMEA sentences to store and render in the raw data tab
const MAX_RAW_NMEA_LOGS: usize = 1000;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Indicates if the application is running.
    pub running: bool,
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
    /// State of the skyplot widget (for rendering of hovered satellite info).
    pub skyplot_state: SkyplotState,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        let nmea_parser = Arc::new(Mutex::new(Nmea::default()));

        Self {
            running: true,
            event_handler: EventHandler::new(Arc::clone(&nmea_parser)),
            tab: AppTab::default(),
            nav_data: NavigationData::default(),
            sv_data: Vec::new(),
            raw_data: FixedCircularBuffer::<RawNmeaLog, MAX_RAW_NMEA_LOGS>::new(),
            nmea_data: nmea_parser,
            skyplot_state: SkyplotState::default(),
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_events()?;
        }
        Ok(())
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
                AppEvent::Quit => self.quit(),
                AppEvent::NmeaMessage(msg) => self.handle_nmea_msg(msg)?,
                AppEvent::RawNmeaSentence(raw) => self.handle_raw_nmea(raw)?,
            },
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        if key_event.kind != crossterm::event::KeyEventKind::Press {
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
                // Handle mouse move events here
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
        self.raw_data.push_back(raw);
        Ok(())
    }

    /// Handles the tick event of the terminal.
    fn tick(&mut self) {}

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

/// Application events.
#[derive(Clone, Debug)]
pub enum AppEvent {
    /// Quit the application.
    Quit,
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
        *self = Self::from_repr(next_index).unwrap_or(*self)
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
