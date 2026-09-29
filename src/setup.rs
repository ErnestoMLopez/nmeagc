use crate::cli::{CliDataBits, CliFlowControl, CliParity, CliStopBits, DataSource, TcpConfig};

use std::path::PathBuf;

use color_eyre::eyre::Error;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct SourceSetup {
    pub step: SetupStep,
    pub source_input: SelectableInput,
    pub config_tcp: ConfigTcp,
    pub config_serial: ConfigSerial,
    pub config_file: ConfigFile,
    pub config_input: ConfigInput,
    pub error: Option<&'static str>,
}

impl Default for SourceSetup {
    fn default() -> Self {
        Self {
            step: SetupStep::Source,
            source_input: SelectableInput::default(),
            config_tcp: ConfigTcp::default(),
            config_serial: ConfigSerial::default(),
            config_file: ConfigFile::default(),
            config_input: ConfigInput::Text(TextInput::default()),
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
            KeyCode::Tab => {
                if let SetupStep::Config(ref mut config_item) = self.step {
                    match config_item {
                        ConfigItem::Tcp(TcpOption::Host) => {
                            if let ConfigInput::Text(ref mut text) = self.config_input {
                                self.config_tcp.host = text.input.clone();
                                text.input.clear();
                            }
                            *config_item = ConfigItem::Tcp(TcpOption::Port);
                        }
                        ConfigItem::Tcp(TcpOption::Port) => {
                            if let ConfigInput::Text(ref mut text) = self.config_input {
                                self.config_tcp.port = text.input.parse().unwrap_or(23000);
                                text.input.clear();
                            }
                        }
                        _ => {}
                    }
                }
                Ok(SetupAction::Continue)
            }
            KeyCode::BackTab => {
                if let SetupStep::Config(ref mut config_item) = self.step {
                    match config_item {
                        ConfigItem::Tcp(TcpOption::Host) => {
                            if let ConfigInput::Text(ref mut text) = self.config_input {
                                self.config_tcp.host = text.input.clone();
                                text.input.clear();
                            }
                        }
                        ConfigItem::Tcp(TcpOption::Port) => {
                            if let ConfigInput::Text(ref mut text) = self.config_input {
                                self.config_tcp.port = text.input.parse().unwrap_or(23000);
                                text.input.clear();
                            }
                            *config_item = ConfigItem::Tcp(TcpOption::Host);
                        }
                        _ => {}
                    }
                }
                Ok(SetupAction::Continue)
            }
            KeyCode::Up => {
                match self.step {
                    SetupStep::Source => {
                        self.source_input.list_state.select_previous();
                    }
                    SetupStep::Config(_) => {}
                    SetupStep::Done => {}
                }
                Ok(SetupAction::Continue)
            }
            KeyCode::Down => {
                match self.step {
                    SetupStep::Source => {
                        self.source_input.list_state.select_next();
                    }
                    SetupStep::Config(_) => {}
                    SetupStep::Done => {}
                }
                Ok(SetupAction::Continue)
            }
            KeyCode::Backspace => {
                if let ConfigInput::Text(ref mut text_input) = self.config_input {
                    text_input.input.pop();
                }
                Ok(SetupAction::Continue)
            }
            KeyCode::Char(character) => {
                if let ConfigInput::Text(ref mut text_input) = self.config_input {
                    text_input.input.push(character);
                }
                Ok(SetupAction::Continue)
            }
            KeyCode::Enter => self.advance(),
            _ => Ok(SetupAction::Continue),
        }
    }

    fn advance(&mut self) -> Result<SetupAction, Error> {
        match self.step {
            SetupStep::Source => {
                let config_item = self.source_input.list_state.selected().into();
                self.step = SetupStep::Config(config_item);
                Ok(SetupAction::Continue)
            }
            SetupStep::Config(ConfigItem::Tcp(_)) => {
                let source = DataSource::Tcp(TcpConfig {
                    host: self.config_tcp.host.clone(),
                    port: self.config_tcp.port,
                });
                self.step = SetupStep::Done;
                Ok(SetupAction::Complete(source))
            }
            _ => Ok(SetupAction::Continue),
        }
    }
}

#[derive(Debug, Default)]
pub struct TextInput {
    pub input: String,
    pub cursor: usize,
}

#[derive(Debug)]
pub struct SelectableInput {
    pub list_state: ListState,
}

#[derive(Debug)]
pub enum ConfigInput {
    Text(TextInput),
    Selectable(SelectableInput),
}

impl Default for SelectableInput {
    fn default() -> Self {
        Self {
            list_state: ListState::default().with_selected(Some(0)),
        }
    }
}

#[derive(Debug)]
pub struct ConfigTcp {
    pub host: String,
    pub port: u16,
}

#[derive(Debug)]
pub struct ConfigSerial {
    pub path: String,
    pub baudrate: u32,
    pub data_bits: CliDataBits,
    pub parity: CliParity,
    pub stop_bits: CliStopBits,
    pub flow_control: CliFlowControl,
    pub timeout: Option<u64>,
}

#[derive(Debug)]
pub struct ConfigFile {
    pub path: PathBuf,
}

impl Default for ConfigTcp {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 23000,
        }
    }
}

impl Default for ConfigSerial {
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

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
        }
    }
}

#[derive(Debug)]
pub enum TcpOption {
    Host,
    Port,
}

#[derive(Debug)]
pub enum SerialOption {
    Path,
    Baudrate,
    DataBits,
    Parity,
    StopBits,
    FlowControl,
    Timeout,
}

#[derive(Debug)]
pub enum FileOption {
    Path,
}

#[derive(Debug)]
pub enum ConfigItem {
    Tcp(TcpOption),
    Serial(SerialOption),
    File(FileOption),
}

impl From<Option<usize>> for ConfigItem {
    fn from(value: Option<usize>) -> Self {
        match value {
            Some(0) => ConfigItem::Tcp(TcpOption::Host),
            Some(1) => ConfigItem::Serial(SerialOption::Path),
            Some(2) => ConfigItem::File(FileOption::Path),
            _ => ConfigItem::Tcp(TcpOption::Host),
        }
    }
}

#[derive(Debug)]
pub enum SetupStep {
    Source,
    Config(ConfigItem),
    Done,
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
