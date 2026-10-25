use crate::{
    cli::{DataSource, TcpConfig},
    theme::THEME,
    widgets::{
        field::{CheckboxInput, ChoiceInput, FieldValue, TextInput, Validator},
        form::Form,
    },
};

use std::path::Path;

use color_eyre::eyre::Error;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

pub struct SourceSetup {
    pub step: SetupStep,
    pub source_input: ChoiceInput,
    pub config_tcp: Form,
    pub config_serial: Form,
    pub config_file: Form,
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
                .field(
                    TextInput::new("Path")
                        .required()
                        .validator(Box::new(PathString)),
                )
                .field(
                    ChoiceInput::new("Baud rate")
                        .options(vec![
                            (FieldValue::Int(1200), "1200"),
                            (FieldValue::Int(2400), "2400"),
                            (FieldValue::Int(9600), "9600"),
                            (FieldValue::Int(19200), "19200"),
                            (FieldValue::Int(38400), "38400"),
                            (FieldValue::Int(57600), "57600"),
                            (FieldValue::Int(115200), "115200"),
                            (FieldValue::Int(230400), "230400"),
                            (FieldValue::Int(460800), "460800"),
                            (FieldValue::Int(921600), "921600"),
                        ])
                        .with_selected(FieldValue::Int(9600))
                        .required(),
                )
                .field(
                    ChoiceInput::new("Data bits")
                        .options(vec![
                            (FieldValue::Int(5), "5"),
                            (FieldValue::Int(6), "6"),
                            (FieldValue::Int(7), "7"),
                            (FieldValue::Int(8), "8"),
                        ])
                        .required(),
                )
                .field(
                    ChoiceInput::new("Parity")
                        .options(vec![
                            (FieldValue::Text("None".to_string()), "None"),
                            (FieldValue::Text("Odd".to_string()), "Odd"),
                            (FieldValue::Text("Even".to_string()), "Even"),
                        ])
                        .required(),
                )
                .field(
                    ChoiceInput::new("Stop bits")
                        .options(vec![(FieldValue::Int(1), "1"), (FieldValue::Int(2), "2")])
                        .required(),
                )
                .field(
                    ChoiceInput::new("Flow control")
                        .options(vec![
                            (FieldValue::Text("None".to_string()), "None"),
                            (FieldValue::Text("SW".to_string()), "Software"),
                            (FieldValue::Text("HW".to_string()), "Hardware"),
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
                .field(
                    TextInput::new("Path")
                        .required()
                        .validator(Box::new(PathString)),
                )
                .cancel_button("Back")
                .submit_button("Finish")
                .style(THEME.popups)
                .buttons_style(THEME.popups_button)
                .highlight_style(THEME.popups_highlight),
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
                if let Some(results) = self.config_tcp.results() {
                    // TODO Replace for form parsing
                    // let config = DataSource::from(results);
                    let source = DataSource::Tcp(TcpConfig {
                        host: "127.0.0.1".to_string(),
                        port: 23000,
                    });
                    Ok(SetupAction::Complete(source))
                } else {
                    Ok(SetupAction::Continue)
                }
            }
            SetupStep::Config(ConfigItem::Serial(_)) => {
                self.config_serial.handle_key_event(key_event);
                if let Some(results) = self.config_serial.results() {
                    // TODO Replace for form parsing
                    // let config = DataSource::from(results);
                    let source = DataSource::Tcp(TcpConfig {
                        host: "127.0.0.1".to_string(),
                        port: 23000,
                    });
                    Ok(SetupAction::Complete(source))
                } else {
                    Ok(SetupAction::Continue)
                }
            }
            SetupStep::Config(ConfigItem::File(_)) => {
                self.config_file.handle_key_event(key_event);
                if let Some(results) = self.config_file.results() {
                    // TODO Replace for form parsing
                    // let config = DataSource::from(results);
                    let source = DataSource::Tcp(TcpConfig {
                        host: "127.0.0.1".to_string(),
                        port: 23000,
                    });
                    Ok(SetupAction::Complete(source))
                } else {
                    Ok(SetupAction::Continue)
                }
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

pub struct ConfigTcp {
    pub host: TextInput,
    pub port: TextInput,
}

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

struct PathString;

impl Validator for PathString {
    fn validate(&self, value: &str) -> Result<(), String> {
        let path = Path::new(value);
        path.exists()
            .then_some(())
            .ok_or("Path does not exists".to_string())
    }
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
