use crate::{
    cli::{
        CliDataBits, CliFlowControl, CliParity, CliStopBits, DataSource, FileConfig, SerialConfig,
        TcpConfig,
    },
    theme::THEME,
    widgets::{
        field::{CheckboxInput, ChoiceInput, FieldValue, TextInput, Validator},
        form::Form,
    },
};

use std::path::{Path, PathBuf};

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
    const BAUD_RATES: [u32; 10] = [
        1200, 2400, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600,
    ];

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
                        .options(
                            SourceSetup::BAUD_RATES
                                .into_iter()
                                .map(|b| (FieldValue::Int(b as i64), b.to_string()))
                                .collect(),
                        )
                        .with_selected(FieldValue::Int(9600))
                        .required(),
                )
                .field(
                    ChoiceInput::new("Data bits")
                        .options(
                            (5..=8)
                                .into_iter()
                                .map(|d| (FieldValue::Int(d as i64), d.to_string()))
                                .collect(),
                        )
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
            SetupStep::Config(SourceKind::Tcp) => {
                self.config_tcp.handle_key_event(key_event);
                match self.config_tcp.results() {
                    None => Ok(SetupAction::Continue),
                    Some(results) => {
                        let source = DataSource::Tcp(TcpConfig::from(results));
                        Ok(SetupAction::Complete(source))
                    }
                }
            }
            SetupStep::Config(SourceKind::Serial) => {
                self.config_serial.handle_key_event(key_event);
                match self.config_serial.results() {
                    None => Ok(SetupAction::Continue),
                    Some(results) => {
                        let source = DataSource::Serial(SerialConfig::from(results));
                        Ok(SetupAction::Complete(source))
                    }
                }
            }
            SetupStep::Config(SourceKind::File) => {
                self.config_file.handle_key_event(key_event);
                match self.config_file.results() {
                    None => Ok(SetupAction::Continue),
                    Some(results) => {
                        let source = DataSource::File(FileConfig::from(results));
                        Ok(SetupAction::Complete(source))
                    }
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
            SetupStep::Config(SourceKind::Tcp) => {
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

impl From<Vec<(String, FieldValue)>> for TcpConfig {
    fn from(input: Vec<(String, FieldValue)>) -> Self {
        let mut tcp_config = TcpConfig::default();
        for (field, value) in input.iter() {
            match (field.as_str(), value) {
                ("Host", FieldValue::Text(v)) => {
                    tcp_config.host = v.clone();
                }
                ("Port", FieldValue::Text(v)) => {
                    tcp_config.port = v.parse::<u16>().unwrap_or_default();
                }
                _ => {}
            }
        }
        tcp_config
    }
}

impl From<Vec<(String, FieldValue)>> for SerialConfig {
    fn from(input: Vec<(String, FieldValue)>) -> Self {
        let mut serial_config = SerialConfig::default();

        for (field, value) in input.iter() {
            match (field.as_str(), value) {
                ("Path", FieldValue::Text(v)) => {
                    serial_config.serial_path = v.clone();
                }
                ("Baud rate", FieldValue::Int(v)) => {
                    serial_config.baudrate = *v as u32;
                }
                ("Data bits", FieldValue::Int(v)) => {
                    serial_config.data_bits = CliDataBits::from(*v as u32);
                }
                ("Parity", FieldValue::Text(v)) => {
                    serial_config.parity = CliParity::from(v.as_str());
                }
                ("Stop bits", FieldValue::Int(v)) => {
                    serial_config.stop_bits = CliStopBits::from(*v as u32);
                }
                ("Flow control", FieldValue::Text(v)) => {
                    serial_config.flow_control = CliFlowControl::from(v.as_str());
                }
                ("Timeout [ms]", FieldValue::Text(v)) => {
                    serial_config.timeout = v.parse::<u64>().ok();
                }
                ("Exclusive access", FieldValue::Bool(v)) => {
                    serial_config.exclusive = Some(*v);
                }
                _ => {}
            }
        }
        serial_config
    }
}

impl From<Vec<(String, FieldValue)>> for FileConfig {
    fn from(input: Vec<(String, FieldValue)>) -> Self {
        let mut file_config = FileConfig::default();
        for (field, value) in input.iter() {
            match (field.as_str(), value) {
                ("Path", FieldValue::Text(v)) => {
                    file_config.path = PathBuf::from(v);
                }
                _ => {}
            }
        }
        file_config
    }
}
