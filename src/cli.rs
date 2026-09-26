use std::path::PathBuf;

use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum, builder::Styles};
use serialport::{DataBits, FlowControl, Parity, StopBits};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, disable_help_flag = true, flatten_help = true, arg_required_else_help = true, styles = Styles::default())]
pub struct Cli {
    /// Execute the client in interactive setup mode
    ///
    /// Interactive mode allows configuring the source of the NMEA data to setup a reader directly from the TUI.
    #[arg(short, long)]
    pub interactive: bool,

    /// Show this help message
    #[arg(short = 'H', long, action = ArgAction::Help)]
    help: Option<bool>,

    /// Specifies the source and configuration of the NMEA data.
    #[command(subcommand)]
    pub source: DataSource,
}

#[derive(Subcommand, Debug)]
pub enum DataSource {
    /// Establish a TCP connection to a host to use it as NMEA data source.
    #[command(short_flag = 't')]
    Tcp(TcpConfig),
    /// Open a serial port to read NMEA data from it.
    #[command(short_flag = 's')]
    Serial(SerialConfig),
    /// Open a file to read NMEA data from it.
    #[command(short_flag = 'f')]
    File(FileConfig),
}

#[derive(Args, Debug)]
pub struct TcpConfig {
    /// Host name for the TCP data source
    #[arg(short, long, default_value_t = "127.0.0.1".to_string(), value_name = "HOST")]
    pub host: String,
    /// Port number for the TCP data source
    #[arg(short, long, default_value_t = 23000)]
    pub port: u16,
}

#[derive(Args, Debug)]
pub struct SerialConfig {
    /// Serial port path
    #[arg(display_order = 0, value_name = "PATH")]
    pub serial_path: String,
    /// Baudrate of the serial data stream
    #[arg(short, long, default_value_t = 9600, display_order = 1)]
    pub baudrate: u32,
    /// Number of data bits
    #[arg(long, value_enum, default_value_t = CliDataBits::Eight, display_order = 2)]
    pub data_bits: CliDataBits,
    /// Parity mode
    #[arg(long, value_enum, default_value_t = CliParity::None, display_order = 2)]
    pub parity: CliParity,
    /// Number of stop bits
    #[arg(long, value_enum, default_value_t = CliStopBits::One, display_order = 2)]
    pub stop_bits: CliStopBits,
    /// Flow control setting
    #[arg(long, value_enum, default_value_t = CliFlowControl::None, display_order = 2)]
    pub flow_control: CliFlowControl,
    /// Read timeout in milliseconds
    #[arg(long)]
    pub timeout: Option<u64>,
}

#[derive(Args, Debug)]
pub struct FileConfig {
    /// File path
    pub path: PathBuf,
}

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
    fn help_is_printed_without_arguments() {
        let result = Cli::try_parse_from(["nmeagc"]);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(
            error.kind(),
            clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        );
        assert!(error.to_string().contains("Usage:"));
    }

    #[test]
    fn parse_serial_source_with_defaults() {
        let source = Cli::try_parse_from(["nmeagc", "-s", "/dev/ttyUSB0"])
            .unwrap()
            .source;

        assert!(matches!(
            source,
            DataSource::Serial(SerialConfig {
                serial_path: path,
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
    fn parse_file_source() {
        let source = Cli::try_parse_from(["nmeagc", "-f", "track.nmea"])
            .unwrap()
            .source;

        assert!(matches!(
            source,
            DataSource::File(FileConfig { path }) if path == PathBuf::from("track.nmea")
        ));
    }

    #[test]
    fn error_sources_are_mutually_exclusive() {
        assert!(Cli::try_parse_from(["nmeagc", "-h", "localhost", "-f", "track.nmea"]).is_err());
        assert!(Cli::try_parse_from(["nmeagc", "-s", "/dev/ttyUSB0", "-f", "track.nmea"]).is_err());
        assert!(Cli::try_parse_from(["nmeagc", "-h", "localhost", "-s", "/dev/ttyUSB0"]).is_err());
    }
}
