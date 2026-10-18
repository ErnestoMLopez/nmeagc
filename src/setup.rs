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

impl SourceSetup {
    /// Creates a new [`SourceSetup`] instance to implement the client interactive setup
    pub fn new() -> Self {
        Self {
            step: SetupStep::Source,
            source_input: ChoiceInput::new("Source"),
            config_tcp: Form::new()
                .field(TextInput::new("Host").required().with_input("127.0.0.1"))
                .field(TextInput::new("Port").required().with_input("23000"))
                .cancel_button("Back")
                .submit_button("Finish")
                .style(THEME.popups)
                .buttons_style(THEME.popups_button)
                .highlight_style(THEME.popups_highlight),
            config_serial: Form::new()
                .field(TextInput::new("Path").required())
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
                        .required(),
                )
                .field(
                    ChoiceInput::new("Data bits")
                        .options(vec![
                            ("Five", "5"),
                            ("Six", "6"),
                            ("Seven", "7"),
                            ("Eight", "8"),
                        ])
                        .required(),
                )
                .field(
                    ChoiceInput::new("Parity")
                        .options(vec![("None", "None"), ("Odd", "Odd"), ("Even", "Even")])
                        .required(),
                )
                .field(
                    ChoiceInput::new("Stop bits")
                        .options(vec![("One", "1"), ("Two", "2")])
                        .required(),
                )
                .field(
                    ChoiceInput::new("Flow control")
                        .options(vec![
                            ("None", "None"),
                            ("Sw", "Software"),
                            ("Hw", "Hardware"),
                        ])
                        .required(),
                )
                .field(TextInput::new("Timeout [ms]"))
                .field(CheckboxInput::new("Exclusive access"))
                .cancel_button("Back")
                .submit_button("Finish")
                .style(THEME.popups)
                .buttons_style(THEME.popups_button)
                .highlight_style(THEME.popups_highlight),
            config_file: Form::new()
                .field(TextInput::new("Path").required())
                .cancel_button("Back")
                .submit_button("Finish")
                .style(THEME.popups)
                .buttons_style(THEME.popups_button)
                .highlight_style(THEME.popups_highlight),
            error: None,
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<SetupAction, Error> {
        if key_event.kind != KeyEventKind::Press {
            return Ok(SetupAction::Continue);
        }

        if let KeyCode::Esc = key_event.code {
            return Ok(SetupAction::Cancel);
        }

        match self.step {
            SetupStep::Source => match key_event.code {
                KeyCode::Up => {
                    self.source_input.selection.select_previous();
                    return Ok(SetupAction::Continue);
                }
                KeyCode::Down => {
                    self.source_input.selection.select_next();
                    return Ok(SetupAction::Continue);
                }
                KeyCode::Enter => return self.advance(),
                _ => {
                    return Ok(SetupAction::Continue);
                }
            },
            SetupStep::Config(ConfigItem::Tcp(_)) => {
                self.config_tcp.handle_key_event(key_event);
                Ok(SetupAction::Continue)
            }
            SetupStep::Config(ConfigItem::Serial(_)) => {
                self.config_serial.handle_key_event(key_event);
                Ok(SetupAction::Continue)
            }
            SetupStep::Config(ConfigItem::File(_)) => {
                self.config_file.handle_key_event(key_event);
                Ok(SetupAction::Continue)
            }
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
