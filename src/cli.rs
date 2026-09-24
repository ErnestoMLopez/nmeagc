use std::path::PathBuf;

use clap::{ArgAction, Args, Parser, ValueEnum};
use serialport::{DataBits, FlowControl, Parity, StopBits};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    /// Executes the client in interactive mode
    ///
    /// Interactive mode allows configuring the source of the NMEA data to setup a reader directly from the TUI.
    #[arg(short, long)]
    interactive: bool,

    /// Show this help message
    #[arg(long, action = ArgAction::Help)]
    help: Option<bool>,

    /// Specifies the source and configuration of the NMEA data.
    #[command(flatten)]
    source: SourceArgs,
}

#[derive(Args, Debug)]
struct SourceArgs {
    #[command(flatten)]
    selector: SourceSelector,
    #[command(flatten)]
    options: SourceOptions,
}

#[derive(Args, Debug)]
#[group(id = "data-source", multiple = false)]
struct SourceSelector {
    /// Host name for the TCP data source
    #[arg(
        short = 'h',
        long,
        help_heading = "Data source: TCP conection",
        value_name = "HOST"
    )]
    host: Option<String>,
    /// Serial port path
    #[arg(
        short = 's',
        long = "serial",
        help_heading = "Data source: Serial port",
        value_name = "PATH"
    )]
    serial: Option<String>,
    /// File path for the NMEA data source
    #[arg(
        short = 'f',
        long = "file",
        help_heading = "Data source: File",
        value_name = "PATH"
    )]
    file: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct SourceOptions {
    /// Port number for the TCP data source
    #[arg(
        short,
        long,
        requires = "host",
        default_value_t = 23000,
        help_heading = "Data source: TCP conection"
    )]
    port: u16,

    /// Baudrate of the serial data stream
    #[arg(
        short,
        long,
        requires = "serial",
        default_value_t = 9600,
        help_heading = "Data source: Serial port"
    )]
    baudrate: u32,
    /// Number of data bits
    #[arg(long, value_enum, requires = "serial", default_value_t = CliDataBits::Eight, help_heading = "Data source: Serial port")]
    data_bits: CliDataBits,
    /// Parity mode
    #[arg(long, value_enum, requires = "serial", default_value_t = CliParity::None, help_heading = "Data source: Serial port")]
    parity: CliParity,
    /// Number of stop bits
    #[arg(long, value_enum, requires = "serial", default_value_t = CliStopBits::One, help_heading = "Data source: Serial port")]
    stop_bits: CliStopBits,
    /// Flow control setting
    #[arg(long, value_enum, requires = "serial", default_value_t = CliFlowControl::None, help_heading = "Data source: Serial port")]
    flow_control: CliFlowControl,
    /// Read timeout in milliseconds
    #[arg(long, requires = "serial", help_heading = "Data source: Serial port")]
    timeout: Option<u64>,
}

#[derive(Debug)]
pub enum DataSource {
    Tcp(TcpConfig),
    Serial(SerialConfig),
    File(FileConfig),
}

#[derive(Debug)]
pub struct TcpConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug)]
pub struct SerialConfig {
    pub path: String,
    pub baudrate: u32,
    pub data_bits: CliDataBits,
    pub parity: CliParity,
    pub stop_bits: CliStopBits,
    pub flow_control: CliFlowControl,
    pub timeout: Option<u64>,
}

#[derive(Debug)]
pub struct FileConfig {
    pub path: PathBuf,
}

impl Cli {
    pub fn into_data_source(self) -> DataSource {
        self.source.into_data_source()
    }
}

impl SourceArgs {
    fn into_data_source(self) -> DataSource {
        if let Some(path) = self.selector.serial {
            return DataSource::Serial(SerialConfig {
                path,
                baudrate: self.options.baudrate,
                data_bits: self.options.data_bits,
                parity: self.options.parity,
                stop_bits: self.options.stop_bits,
                flow_control: self.options.flow_control,
                timeout: self.options.timeout,
            });
        }

        if let Some(path) = self.selector.file {
            return DataSource::File(FileConfig { path });
        }

        DataSource::Tcp(TcpConfig {
            host: self
                .selector
                .host
                .unwrap_or_else(|| "127.0.0.1".to_string()),
            port: self.options.port,
        })
    }
}

// Local enums mapping string arguments to serialport crate types
#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum CliDataBits {
    Five,
    Six,
    Seven,
    Eight,
}

impl From<CliDataBits> for DataBits {
    fn from(bits: CliDataBits) -> Self {
        match bits {
            CliDataBits::Five => DataBits::Five,
            CliDataBits::Six => DataBits::Six,
            CliDataBits::Seven => DataBits::Seven,
            CliDataBits::Eight => DataBits::Eight,
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum CliParity {
    None,
    Odd,
    Even,
}

impl From<CliParity> for Parity {
    fn from(p: CliParity) -> Self {
        match p {
            CliParity::None => Parity::None,
            CliParity::Odd => Parity::Odd,
            CliParity::Even => Parity::Even,
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum CliStopBits {
    One,
    Two,
}

impl From<CliStopBits> for StopBits {
    fn from(s: CliStopBits) -> Self {
        match s {
            CliStopBits::One => StopBits::One,
            CliStopBits::Two => StopBits::Two,
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum CliFlowControl {
    None,
    Software,
    Hardware,
}

impl From<CliFlowControl> for FlowControl {
    fn from(f: CliFlowControl) -> Self {
        match f {
            CliFlowControl::None => FlowControl::None,
            CliFlowControl::Software => FlowControl::Software,
            CliFlowControl::Hardware => FlowControl::Hardware,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn defaults_to_tcp_source() {
        let source = Cli::try_parse_from(["nmeagc"]).unwrap().into_data_source();

        assert!(matches!(
            source,
            DataSource::Tcp(TcpConfig { host, port })
                if host == "127.0.0.1" && port == 23000
        ));
    }

    #[test]
    fn parses_serial_source_and_defaults() {
        let source = Cli::try_parse_from(["nmeagc", "-s", "/dev/ttyUSB0"])
            .unwrap()
            .into_data_source();

        assert!(matches!(
            source,
            DataSource::Serial(SerialConfig {
                path,
                baudrate: 9600,
                data_bits: CliDataBits::Eight,
                parity: CliParity::None,
                stop_bits: CliStopBits::One,
                flow_control: CliFlowControl::None,
                timeout: None,
            }) if path == "/dev/ttyUSB0"
        ));
    }

    #[test]
    fn parses_file_source() {
        let source = Cli::try_parse_from(["nmeagc", "-f", "track.nmea"])
            .unwrap()
            .into_data_source();

        assert!(matches!(
            source,
            DataSource::File(FileConfig { path }) if path == PathBuf::from("track.nmea")
        ));
    }

    #[test]
    fn source_selectors_are_mutually_exclusive() {
        assert!(Cli::try_parse_from(["nmeagc", "-h", "example.com", "-f", "track.nmea"]).is_err());
    }
}
