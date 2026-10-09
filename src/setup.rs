use crate::{
    cli::{DataSource, TcpConfig},
    theme::THEME,
    widgets::{
        field::{CheckboxInput, ChoiceInput, TextInput},
        form::Form,
    },
};

use color_eyre::eyre::Error;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

pub struct SourceSetup {
    pub step: SetupStep,
    pub source_input: ChoiceInput,
    pub config_tcp: Form,
    pub config_serial: Form,
    pub config_file: Form,
    pub error: Option<&'static str>,
}

impl Default for SourceSetup {
    fn default() -> Self {
        Self {
            step: SetupStep::Source,
            source_input: ChoiceInput::new("Source"),
            config_tcp: Form::new()
                .style(THEME.popups)
                .field(TextInput::new("Host").required().with_input("127.0.0.1"))
                .field(
                    TextInput::new("Port")
                        .required()
                        .with_input("23000")
                        .style(THEME.popups),
                ),
            config_serial: Form::new()
                .style(THEME.popups)
                .field(TextInput::new("Path").required().style(THEME.popups))
                .field(
                    ChoiceInput::new("Baud rate")
                        .options(vec![
                            ("1200", "1200"),
                            ("2400", "2400"),
                            ("9600", "9600"),
                            ("19200", "19200"),
                            ("38400", "38400"),
                            ("57600", "57600"),
                            ("115200", "115200"),
                            ("230400", "230400"),
                            ("460800", "460800"),
                            ("921600", "921600"),
                        ])
                        .with_selected("9600")
                        .required()
                        .style(THEME.popups),
                )
                .field(
                    ChoiceInput::new("Data bits")
                        .options(vec![
                            ("Five", "5"),
                            ("Six", "6"),
                            ("Seven", "7"),
                            ("Eight", "8"),
                        ])
                        .with_selected("Eight")
                        .required()
                        .style(THEME.popups),
                )
                .field(
                    ChoiceInput::new("Parity")
                        .options(vec![("None", "None"), ("Odd", "Odd"), ("Even", "Even")])
                        .with_selected("None")
                        .required()
                        .style(THEME.popups),
                )
                .field(
                    ChoiceInput::new("Stop bits")
                        .options(vec![("One", "1"), ("Two", "2")])
                        .with_selected("One")
                        .required()
                        .style(THEME.popups),
                )
                .field(
                    ChoiceInput::new("Flow control")
                        .options(vec![
                            ("None", "None"),
                            ("Sw", "Software"),
                            ("Hw", "Hardware"),
                        ])
                        .with_selected("None")
                        .required()
                        .style(THEME.popups),
                )
                .field(TextInput::new("Timeout").style(THEME.popups))
                .field(CheckboxInput::new("Exclusive").style(THEME.popups)),
            config_file: Form::new()
                .style(THEME.popups)
                .field(TextInput::new("Path").required().style(THEME.popups)),
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
            KeyCode::Backspace => Ok(SetupAction::Continue),
            KeyCode::Char(_) => Ok(SetupAction::Continue),
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
                    host: "127.0.0.1".to_string(), // TODO Replace for form parsing
                    port: 23000,
                });
                self.step = SetupStep::Done;
                Ok(SetupAction::Complete(source))
            }
            _ => Ok(SetupAction::Continue),
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
