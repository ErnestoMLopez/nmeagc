use crate::{
    cli::{DataSource, TcpConfig},
    theme::THEME,
    widgets::field::{CheckboxInput, ChoiceInput, TextInput},
};

use color_eyre::eyre::Error;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

#[derive(Debug)]
pub struct SourceSetup {
    pub step: SetupStep,
    pub source_input: ChoiceInput,
    pub config_tcp: ConfigTcp,
    pub config_serial: ConfigSerial,
    pub config_file: ConfigFile,
    pub error: Option<&'static str>,
}

impl Default for SourceSetup {
    fn default() -> Self {
        Self {
            step: SetupStep::Source,
            source_input: ChoiceInput::new("Source"),
            config_tcp: ConfigTcp::default(),
            config_serial: ConfigSerial::default(),
            config_file: ConfigFile::default(),
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
                            *config_item = ConfigItem::Tcp(TcpOption::Port);
                        }
                        ConfigItem::Tcp(TcpOption::Port) => {}
                        _ => {}
                    }
                }
                Ok(SetupAction::Continue)
            }
            KeyCode::BackTab => {
                if let SetupStep::Config(ref mut config_item) = self.step {
                    match config_item {
                        ConfigItem::Tcp(TcpOption::Host) => {}
                        ConfigItem::Tcp(TcpOption::Port) => {
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
                        self.source_input.selection.select_previous();
                    }
                    SetupStep::Config(_) => {}
                    SetupStep::Done => {}
                }
                Ok(SetupAction::Continue)
            }
            KeyCode::Down => {
                match self.step {
                    SetupStep::Source => {
                        self.source_input.selection.select_next();
                    }
                    SetupStep::Config(_) => {}
                    SetupStep::Done => {}
                }
                Ok(SetupAction::Continue)
            }
            KeyCode::Backspace => {
                if let Some(ref mut text_input) = self.get_text_input_mut() {
                    text_input.input.pop();
                }
                Ok(SetupAction::Continue)
            }
            KeyCode::Char(character) => {
                if let Some(ref mut text_input) = self.get_text_input_mut() {
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
                let config_item = self.source_input.selection.selected().into();
                self.step = SetupStep::Config(config_item);
                Ok(SetupAction::Continue)
            }
            SetupStep::Config(ConfigItem::Tcp(_)) => {
                let source = DataSource::Tcp(TcpConfig {
                    host: self.config_tcp.host.input.clone(),
                    port: self.config_tcp.port.input.parse()?,
                });
                self.step = SetupStep::Done;
                Ok(SetupAction::Complete(source))
            }
            _ => Ok(SetupAction::Continue),
        }
    }

    pub fn get_text_input(&self) -> Option<&TextInput> {
        match self.step {
            SetupStep::Config(ConfigItem::Tcp(TcpOption::Host)) => Some(&self.config_tcp.host),
            SetupStep::Config(ConfigItem::Tcp(TcpOption::Port)) => Some(&self.config_tcp.port),
            SetupStep::Config(ConfigItem::Serial(SerialOption::Path)) => {
                Some(&self.config_serial.path)
            }
            SetupStep::Config(ConfigItem::Serial(SerialOption::Timeout)) => {
                Some(&self.config_serial.timeout)
            }
            SetupStep::Config(ConfigItem::File(FileOption::Path)) => Some(&self.config_file.path),
            _ => None,
        }
    }

    pub fn get_text_input_mut(&mut self) -> Option<&mut TextInput> {
        match self.step {
            SetupStep::Config(ConfigItem::Tcp(TcpOption::Host)) => Some(&mut self.config_tcp.host),
            SetupStep::Config(ConfigItem::Tcp(TcpOption::Port)) => Some(&mut self.config_tcp.port),
            SetupStep::Config(ConfigItem::Serial(SerialOption::Path)) => {
                Some(&mut self.config_serial.path)
            }
            SetupStep::Config(ConfigItem::Serial(SerialOption::Timeout)) => {
                Some(&mut self.config_serial.timeout)
            }
            SetupStep::Config(ConfigItem::File(FileOption::Path)) => {
                Some(&mut self.config_file.path)
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct ConfigTcp {
    pub host: TextInput,
    pub port: TextInput,
}

#[derive(Debug)]
pub struct ConfigSerial {
    pub path: TextInput,
    pub baudrate: ChoiceInput,
    pub data_bits: ChoiceInput,
    pub parity: ChoiceInput,
    pub stop_bits: ChoiceInput,
    pub flow_control: ChoiceInput,
    pub timeout: TextInput,
    pub exclusive: CheckboxInput,
}

#[derive(Debug)]
pub struct ConfigFile {
    pub path: TextInput,
}

impl Default for ConfigTcp {
    fn default() -> Self {
        Self {
            host: TextInput::new("Host")
                .with_input("127.0.0.1")
                .required()
                .style(THEME.popups),
            port: TextInput::new("Port")
                .with_input("23000")
                .required()
                .style(THEME.popups),
        }
    }
}

impl Default for ConfigSerial {
    fn default() -> Self {
        Self {
            path: TextInput::new("Path").required().style(THEME.popups),
            baudrate: ChoiceInput::new("Baudrate"),
            data_bits: ChoiceInput::new("Data bits"),
            parity: ChoiceInput::new("Parity"),
            stop_bits: ChoiceInput::new("Stop bits"),
            flow_control: ChoiceInput::new("Flow control"),
            timeout: TextInput::new("Timeout").style(THEME.popups),
            exclusive: CheckboxInput::new("Exclusive"),
        }
    }
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            path: TextInput::new("Path").required().style(THEME.popups),
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
