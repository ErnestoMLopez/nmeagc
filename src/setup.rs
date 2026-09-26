use crate::cli::{CliDataBits, CliFlowControl, CliParity, CliStopBits, DataSource, TcpConfig};

use std::path::PathBuf;

use color_eyre::eyre::{Error, Ok};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct SourceSetup {
    pub step: SetupStep,
    pub source_selection: ListState,
    pub input: String,
    pub error: Option<&'static str>,
    pub tcp_host: String,
    pub tcp_port: u16,
    pub serial_path: String,
    pub baudrate: u32,
    pub data_bits: CliDataBits,
    pub parity: CliParity,
    pub stop_bits: CliStopBits,
    pub flow_control: CliFlowControl,
    pub timeout: Option<u64>,
    pub file_path: PathBuf,
}

impl Default for SourceSetup {
    fn default() -> Self {
        Self {
            step: SetupStep::Source,
            source_selection: ListState::default().with_selected(Some(0)),
            input: String::new(),
            error: None,
            tcp_host: "127.0.0.1".to_string(),
            tcp_port: 23000,
            serial_path: String::new(),
            baudrate: 9600,
            data_bits: CliDataBits::Eight,
            parity: CliParity::None,
            stop_bits: CliStopBits::One,
            flow_control: CliFlowControl::None,
            timeout: None,
            file_path: PathBuf::new(),
        }
    }
}

impl SourceSetup {
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<SetupAction, Error> {
        if key.kind != KeyEventKind::Press {
            return Ok(SetupAction::Continue);
        }

        match key.code {
            KeyCode::Esc => Ok(SetupAction::Cancel),
            KeyCode::Up => {
                match self.step {
                    SetupStep::Source => {
                        self.source_selection.select_previous();
                    }
                    SetupStep::Config => {}
                    SetupStep::Done => {}
                }
                Ok(SetupAction::Continue)
            }
            KeyCode::Down => {
                match self.step {
                    SetupStep::Source => {
                        self.source_selection.select_next();
                    }
                    SetupStep::Config => {}
                    SetupStep::Done => {}
                }
                Ok(SetupAction::Continue)
            }
            KeyCode::Backspace if !matches!(self.step, SetupStep::Source | SetupStep::Config) => {
                self.input.pop();
                Ok(SetupAction::Continue)
            }
            KeyCode::Char(character)
                if !matches!(self.step, SetupStep::Source | SetupStep::Config) =>
            {
                self.input.push(character);
                Ok(SetupAction::Continue)
            }
            KeyCode::Enter => self.advance(),
            _ => Ok(SetupAction::Continue),
        }
    }

    fn advance(&mut self) -> Result<SetupAction, Error> {
        match self.step {
            SetupStep::Source => {
                self.step = self.step.next();
                Ok(SetupAction::Continue)
            }
            SetupStep::Config => Ok(SetupAction::Continue),
            SetupStep::Done => {
                // TODO: Create DataSource from configured values
                let source = DataSource::Tcp(TcpConfig {
                    host: "127.0.0.1".to_string(),
                    port: 23000,
                });
                Ok(SetupAction::Complete(source))
            }
        }
    }
}

#[derive(Debug)]
pub enum SetupStep {
    Source,
    Config,
    Done,
}

impl SetupStep {
    fn next(&self) -> Self {
        match self {
            Self::Source => Self::Config,
            Self::Config => Self::Done,
            Self::Done => Self::Done,
        }
    }

    fn prev(&self) -> Self {
        match self {
            Self::Source => Self::Source,
            Self::Config => Self::Source,
            Self::Done => Self::Config,
        }
    }
}

pub enum SetupAction {
    Continue,
    Cancel,
    Complete(DataSource),
}

#[derive(Clone, Copy, Debug)]
pub enum SourceKind {
    Tcp,
    Serial,
    File,
}

impl SourceKind {
    pub const ALL: [Self; 3] = [Self::Tcp, Self::Serial, Self::File];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tcp => "TCP stream",
            Self::Serial => "Serial port",
            Self::File => "File",
        }
    }
}
