use crate::cli::{CliDataBits, CliFlowControl, CliParity, CliStopBits, DataSource, TcpConfig};

use std::path::PathBuf;

use color_eyre::eyre::{Error, Ok};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct SourceSetup {
    pub step: SetupStep,
    pub source_state: SourceState,
    pub config_tcp_state: ConfigTcpState,
    pub config_serial_state: ConfigSerialState,
    pub config_file_state: ConfigFileState,
    pub input: String,
    pub error: Option<&'static str>,
}

#[derive(Debug)]
pub struct SourceState {
    pub source_list_state: ListState,
}

#[derive(Debug)]
pub struct ConfigTcpState {
    pub host: String,
    pub port: u16,
}

#[derive(Debug)]
pub struct ConfigSerialState {
    pub path: String,
    pub baudrate: u32,
    pub data_bits: CliDataBits,
    pub parity: CliParity,
    pub stop_bits: CliStopBits,
    pub flow_control: CliFlowControl,
    pub timeout: Option<u64>,
}

#[derive(Debug)]
pub struct ConfigFileState {
    pub path: PathBuf,
}

impl Default for SourceState {
    fn default() -> Self {
        Self {
            source_list_state: ListState::default().with_selected(Some(0)),
        }
    }
}

impl Default for ConfigTcpState {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 23000,
        }
    }
}

impl Default for ConfigSerialState {
    fn default() -> Self {
        Self {
            path: String::new(),
            baudrate: 9600,
            data_bits: CliDataBits::Eight,
            parity: CliParity::Odd,
            stop_bits: CliStopBits::One,
            flow_control: CliFlowControl::None,
            timeout: None,
        }
    }
}

impl Default for ConfigFileState {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
        }
    }
}

impl Default for SourceSetup {
    fn default() -> Self {
        Self {
            step: SetupStep::Source,
            source_state: SourceState::default(),
            config_tcp_state: ConfigTcpState::default(),
            config_serial_state: ConfigSerialState::default(),
            config_file_state: ConfigFileState::default(),
            input: String::new(),
            error: None,
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
                        self.source_state.source_list_state.select_previous();
                    }
                    SetupStep::Config(_) => {}
                    SetupStep::Done => {}
                }
                Ok(SetupAction::Continue)
            }
            KeyCode::Down => {
                match self.step {
                    SetupStep::Source => {
                        self.source_state.source_list_state.select_next();
                    }
                    SetupStep::Config(_) => {}
                    SetupStep::Done => {}
                }
                Ok(SetupAction::Continue)
            }
            KeyCode::Backspace if matches!(self.step, SetupStep::Config(_)) => {
                self.input.pop();
                Ok(SetupAction::Continue)
            }
            KeyCode::Char(character) if matches!(self.step, SetupStep::Config(_)) => {
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
                let source_kind = self.source_state.source_list_state.selected().into();
                self.step = SetupStep::Config(source_kind);
                Ok(SetupAction::Continue)
            }
            SetupStep::Config(_) => Ok(SetupAction::Continue),
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

#[derive(Debug, PartialEq)]
pub enum SetupStep {
    Source,
    Config(SourceKind),
    Done,
}

pub enum SetupAction {
    Continue,
    Cancel,
    Complete(DataSource),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SourceKind {
    Tcp,
    Serial,
    File,
}

impl From<Option<usize>> for SourceKind {
    fn from(value: Option<usize>) -> Self {
        match value {
            Some(0) => SourceKind::Tcp,
            Some(1) => SourceKind::Serial,
            Some(2) => SourceKind::File,
            _ => SourceKind::Tcp,
        }
    }
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
