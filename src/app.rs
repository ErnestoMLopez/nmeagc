use crate::event::{Event, EventHandler};

use color_eyre::Result;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;
use strum::{Display, EnumIter, FromRepr};

/// Application.
#[derive(Debug)]
pub struct App {
    /// Indicates if the application is running.
    pub running: bool,
    /// Event handler.
    pub event_handler: EventHandler,
    /// Current tab.
    pub tab: AppTab,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            event_handler: EventHandler::new(),
            tab: AppTab::default(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
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
                CrosstermEvent::Mouse(_) => {}
                _ => {}
            },
            Event::App(app_event) => match app_event {
                AppEvent::Quit => self.quit(),
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

    /// Handles the tick event of the terminal.
    fn tick(&self) {}

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
