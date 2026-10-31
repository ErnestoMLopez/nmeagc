use crate::{
    cli::{
        CliDataBits, CliFlowControl, CliParity, CliStopBits, DataSource, FileConfig, SerialConfig,
        TcpConfig,
    },
    theme::THEME,
    widgets::{
        field::{CheckboxInput, ChoiceInput, FieldValue, TextInput, Validator},
        form::{Form, FormResults},
    },
};

use std::path::{Path, PathBuf};

use color_eyre::eyre::Error;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

pub struct SourceSetup {
    pub step: SetupStep,
    pub config_io: Form,
    pub config_tcp: Form,
    pub config_serial: Form,
    pub config_file: Form,
    pub output_file: Option<PathBuf>,
}

impl SourceSetup {
    const BAUD_RATES: [u32; 10] = [
        1200, 2400, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600,
    ];

    /// Creates a new [`SourceSetup`] instance to implement the client interactive setup
    pub fn new() -> Self {
        Self {
            step: SetupStep::IoConfig,
            config_io: Form::new()
                .field(
                    ChoiceInput::new("Input source")
                        .options(
                            SourceKind::ALL
                                .into_iter()
                                .map(|s| (FieldValue::Text(s.as_str().to_string()), s.as_str()))
                                .collect(),
                        )
                        .required(),
                )
                .field(TextInput::new("Output file"))
                .cancel_button("Cancel")
                .submit_button("Next")
                .style(THEME.popups)
                .buttons_style(THEME.popups_button)
                .highlight_style(THEME.popups_highlight),
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
            output_file: None,
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
            SetupStep::IoConfig => {
                self.config_io.handle_key_event(key_event);
                match self.config_io.results() {
                    None => Ok(SetupAction::Continue),
                    Some(results) => {
                        let source_kind = SourceKind::from(&results);
                        self.output_file = PathBuf::try_from(&results).ok();
                        self.step = SetupStep::SourceConfig(source_kind);
                        Ok(SetupAction::Continue)
                    }
                }
            }
            SetupStep::SourceConfig(SourceKind::Tcp) => {
                self.config_tcp.handle_key_event(key_event);
                match self.config_tcp.results() {
                    None => Ok(SetupAction::Continue),
                    Some(results) => {
                        let source = DataSource::Tcp(TcpConfig::from(&results));
                        Ok(SetupAction::Complete(source))
                    }
                }
            }
            SetupStep::SourceConfig(SourceKind::Serial) => {
                self.config_serial.handle_key_event(key_event);
                match self.config_serial.results() {
                    None => Ok(SetupAction::Continue),
                    Some(results) => {
                        let source = DataSource::Serial(SerialConfig::from(&results));
                        Ok(SetupAction::Complete(source))
                    }
                }
            }
            SetupStep::SourceConfig(SourceKind::File) => {
                self.config_file.handle_key_event(key_event);
                match self.config_file.results() {
                    None => Ok(SetupAction::Continue),
                    Some(results) => {
                        let source = DataSource::File(FileConfig::from(&results));
                        Ok(SetupAction::Complete(source))
                    }
                }
            }
            _ => Ok(SetupAction::Continue),
        }
    }
}

#[derive(Debug)]
pub enum SetupStep {
    IoConfig,
    SourceConfig(SourceKind),
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

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tcp => "TCP stream",
            Self::Serial => "Serial port",
            Self::File => "File",
        }
    }
}

impl From<&FormResults> for SourceKind {
    fn from(results: &FormResults) -> Self {
        let mut source_kind = SourceKind::Tcp;

        for field_results in results {
            match field_results.decompose() {
                ("Input source", FieldValue::Text(v)) => match v.as_str() {
                    v if v == SourceKind::Tcp.as_str() => {
                        source_kind = SourceKind::Tcp;
                    }
                    v if v == SourceKind::Serial.as_str() => {
                        source_kind = SourceKind::Serial;
                    }
                    v if v == SourceKind::File.as_str() => {
                        source_kind = SourceKind::File;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        source_kind
    }
}

impl TryFrom<&FormResults> for PathBuf {
    type Error = ();
    fn try_from(results: &FormResults) -> Result<Self, Self::Error> {
        for field_results in results {
            match field_results.decompose() {
                ("Output file", FieldValue::Text(v)) => {
                    return Ok(v.into());
                }
                _ => {}
            }
        }
        Err(())
    }
}

impl From<&FormResults> for TcpConfig {
    fn from(results: &FormResults) -> Self {
        let mut tcp_config = TcpConfig::default();
        for field_results in results {
            match field_results.decompose() {
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

impl From<&FormResults> for SerialConfig {
    fn from(results: &FormResults) -> Self {
        let mut serial_config = SerialConfig::default();

        for field_results in results {
            match field_results.decompose() {
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

impl From<&FormResults> for FileConfig {
    fn from(results: &FormResults) -> Self {
        let mut file_config = FileConfig::default();
        for field_results in results {
            match field_results.decompose() {
                ("Path", FieldValue::Text(v)) => {
                    file_config.path = PathBuf::from(v);
                }
                _ => {}
            }
        }
        file_config
    }
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
