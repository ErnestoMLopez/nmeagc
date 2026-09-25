use crate::cli::{
    CliDataBits, CliFlowControl, CliParity, CliStopBits, DataSource, FileConfig, SerialConfig,
    TcpConfig,
};
use crate::event::{Event, EventHandler};
use crate::theme::THEME;

use color_eyre::Result;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug)]
enum SourceKind {
    Tcp,
    Serial,
    File,
}

impl SourceKind {
    const ALL: [Self; 3] = [Self::Tcp, Self::Serial, Self::File];

    fn label(self) -> &'static str {
        match self {
            Self::Tcp => "TCP",
            Self::Serial => "Serial",
            Self::File => "File",
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Step {
    Source,
    Text(&'static str),
    Choice(&'static str),
    Confirm,
}

#[derive(Debug)]
pub struct SourceSetup {
    source_kind: SourceKind,
    step: Step,
    selection: usize,
    input: String,
    error: Option<&'static str>,
    tcp_host: String,
    tcp_port: u16,
    serial_path: String,
    baudrate: u32,
    data_bits: CliDataBits,
    parity: CliParity,
    stop_bits: CliStopBits,
    flow_control: CliFlowControl,
    timeout: Option<u64>,
    file_path: PathBuf,
}

impl Default for SourceSetup {
    fn default() -> Self {
        Self {
            source_kind: SourceKind::Tcp,
            step: Step::Source,
            selection: 0,
            input: String::new(),
            error: None,
            tcp_host: "127.0.0.1".to_string(),
            tcp_port: 23000,
            serial_path: String::new(),
            baudrate: 9600,
            data_bits: CliDataBits::Eight,
            parity: CliParity::None,
            stop_bits: CliStopBits::One,
            flow_control: CliFlowControl::None,
            timeout: None,
            file_path: PathBuf::new(),
        }
    }
}

impl SourceSetup {
    pub fn run(
        terminal: &mut DefaultTerminal,
        events: &EventHandler,
    ) -> Result<Option<DataSource>> {
        let mut setup = Self::default();

        loop {
            terminal.draw(|frame| setup.render(frame))?;
            if let Event::Crossterm(CrosstermEvent::Key(key)) = events.next()? {
                match setup.handle_key(key) {
                    SetupAction::Continue => {}
                    SetupAction::Cancel => return Ok(None),
                    SetupAction::Complete(source) => return Ok(Some(source)),
                }
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> SetupAction {
        if key.kind != KeyEventKind::Press {
            return SetupAction::Continue;
        }

        match key.code {
            KeyCode::Esc => SetupAction::Cancel,
            KeyCode::Up => {
                self.selection = self.selection.saturating_sub(1);
                SetupAction::Continue
            }
            KeyCode::Down => {
                self.selection = self.selection.saturating_add(1);
                SetupAction::Continue
            }
            KeyCode::Backspace if !matches!(self.step, Step::Source | Step::Choice(_)) => {
                self.input.pop();
                SetupAction::Continue
            }
            KeyCode::Char(character) if !matches!(self.step, Step::Source | Step::Choice(_)) => {
                self.input.push(character);
                SetupAction::Continue
            }
            KeyCode::Enter => self.advance(),
            _ => SetupAction::Continue,
        }
    }

    fn advance(&mut self) -> SetupAction {
        match self.step {
            Step::Source => {
                self.source_kind = SourceKind::ALL[self.selection.min(SourceKind::ALL.len() - 1)];
                self.selection = 0;
                self.input = match self.source_kind {
                    SourceKind::Tcp => self.tcp_host.clone(),
                    SourceKind::Serial => self.serial_path.clone(),
                    SourceKind::File => self.file_path.display().to_string(),
                };
                self.step = Step::Text(match self.source_kind {
                    SourceKind::Tcp => "TCP host",
                    SourceKind::Serial => "Serial device path",
                    SourceKind::File => "File path",
                });
                SetupAction::Continue
            }
            Step::Text(label) => match self.accept_text(label) {
                Ok(()) => {
                    self.error = None;
                    self.set_next_step();
                    SetupAction::Continue
                }
                Err(message) => {
                    self.error = Some(message);
                    SetupAction::Continue
                }
            },
            Step::Choice(_) => {
                self.accept_choice();
                self.set_next_step();
                SetupAction::Continue
            }
            Step::Confirm => {
                if self.selection == 0 {
                    SetupAction::Complete(self.data_source())
                } else {
                    self.reset_for_source();
                    SetupAction::Continue
                }
            }
        }
    }

    fn accept_text(&mut self, label: &str) -> std::result::Result<(), &'static str> {
        if label == "Timeout (ms, optional)" && self.input.trim().is_empty() {
            self.timeout = None;
            return Ok(());
        }
        if self.input.trim().is_empty() {
            return Err(match label {
                "TCP host" => "TCP host cannot be empty",
                "Serial device path" => "Serial path cannot be empty",
                "File path" => "File path cannot be empty",
                _ => "Value cannot be empty",
            });
        }

        match (self.source_kind, label) {
            (SourceKind::Tcp, "TCP host") => self.tcp_host = self.input.trim().to_string(),
            (SourceKind::Tcp, "TCP port") => {
                self.tcp_port = self.input.parse().map_err(|_| "Enter a valid TCP port")?;
            }
            (SourceKind::Serial, "Serial device path") => {
                self.serial_path = self.input.trim().to_string()
            }
            (SourceKind::Serial, "Baudrate") => {
                self.baudrate = self.input.parse().map_err(|_| "Enter a valid baudrate")?;
            }
            (SourceKind::Serial, "Timeout (ms, optional)") => {
                self.timeout = Some(self.input.parse().map_err(|_| "Enter a valid timeout")?);
            }
            (SourceKind::File, "File path") => self.file_path = PathBuf::from(self.input.trim()),
            _ => return Err("Enter a valid value"),
        }
        Ok(())
    }

    fn accept_choice(&mut self) {
        match self.step {
            Step::Choice("Serial data bits") => {
                self.data_bits = [
                    CliDataBits::Five,
                    CliDataBits::Six,
                    CliDataBits::Seven,
                    CliDataBits::Eight,
                ][self.selection.min(3)]
            }
            Step::Choice("Serial parity") => {
                self.parity =
                    [CliParity::None, CliParity::Odd, CliParity::Even][self.selection.min(2)]
            }
            Step::Choice("Serial stop bits") => {
                self.stop_bits = [CliStopBits::One, CliStopBits::Two][self.selection.min(1)]
            }
            Step::Choice("Serial flow control") => {
                self.flow_control = [
                    CliFlowControl::None,
                    CliFlowControl::Software,
                    CliFlowControl::Hardware,
                ][self.selection.min(2)]
            }
            _ => {}
        }
    }

    fn set_next_step(&mut self) {
        self.selection = 0;
        self.input.clear();
        self.step = match (self.source_kind, self.step) {
            (SourceKind::Tcp, Step::Text("TCP host")) => Step::Text("TCP port"),
            (SourceKind::Tcp, Step::Text("TCP port")) => Step::Confirm,
            (SourceKind::Serial, Step::Text("Serial device path")) => Step::Text("Baudrate"),
            (SourceKind::Serial, Step::Text("Baudrate")) => Step::Choice("Serial data bits"),
            (SourceKind::Serial, Step::Choice("Serial data bits")) => Step::Choice("Serial parity"),
            (SourceKind::Serial, Step::Choice("Serial parity")) => Step::Choice("Serial stop bits"),
            (SourceKind::Serial, Step::Choice("Serial stop bits")) => {
                Step::Choice("Serial flow control")
            }
            (SourceKind::Serial, Step::Choice("Serial flow control")) => {
                Step::Text("Timeout (ms, optional)")
            }
            (SourceKind::Serial, Step::Text("Timeout (ms, optional)")) => Step::Confirm,
            (SourceKind::File, Step::Text("File path")) => Step::Confirm,
            (_, _) => Step::Source,
        };
        if let Step::Text(label) = self.step {
            self.input = match label {
                "TCP port" => self.tcp_port.to_string(),
                "Baudrate" => self.baudrate.to_string(),
                "Timeout (ms, optional)" => self
                    .timeout
                    .map_or(String::new(), |value| value.to_string()),
                _ => self.input.clone(),
            };
        }
    }

    fn reset_for_source(&mut self) {
        self.step = Step::Source;
        self.selection = 0;
        self.input.clear();
    }

    fn data_source(&self) -> DataSource {
        match self.source_kind {
            SourceKind::Tcp => DataSource::Tcp(TcpConfig {
                host: self.tcp_host.clone(),
                port: self.tcp_port,
            }),
            SourceKind::Serial => DataSource::Serial(SerialConfig {
                serial_path: self.serial_path.clone(),
                baudrate: self.baudrate,
                data_bits: self.data_bits,
                parity: self.parity,
                stop_bits: self.stop_bits,
                flow_control: self.flow_control,
                timeout: self.timeout,
            }),
            SourceKind::File => DataSource::File(FileConfig {
                path: self.file_path.clone(),
            }),
        }
    }

    fn render(&self, frame: &mut Frame) {
        let area = centered_rect(70, 70, frame.area());
        frame.render_widget(Clear, area);
        let block = Block::default()
            .title(" Configure NMEA source ")
            .borders(Borders::ALL)
            .border_style(THEME.borders)
            .style(THEME.content);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let [content, footer] = inner.layout(&Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(2),
        ]));
        match self.step {
            Step::Source => self.render_list(
                frame,
                content,
                "Source type",
                &SourceKind::ALL
                    .iter()
                    .map(|kind| kind.label())
                    .collect::<Vec<_>>(),
            ),
            Step::Choice(label) => self.render_choice(frame, content, label),
            Step::Text(label) => {
                let mut lines = vec![
                    Line::from(label),
                    Line::from(Span::styled(
                        format!("{}|", self.input),
                        Style::default().add_modifier(Modifier::UNDERLINED),
                    )),
                ];
                if let Some(error) = self.error {
                    lines.push(Line::from(error));
                }
                let prompt = Paragraph::new(lines);
                frame.render_widget(prompt, content);
            }
            Step::Confirm => {
                let summary = format!(
                    "{}\n\nPress Enter to connect or Up/Down to choose again",
                    self.summary()
                );
                frame.render_widget(Paragraph::new(summary), content);
            }
        }
        frame.render_widget(
            Paragraph::new("Up/Down select   Enter continue   Esc cancel"),
            footer,
        );
    }

    fn render_choice(&self, frame: &mut Frame, area: Rect, label: &str) {
        let values: Vec<String> = match label {
            "Serial data bits" => ["5", "6", "7", "8"]
                .iter()
                .map(|value| value.to_string())
                .collect(),
            "Serial parity" => ["None", "Odd", "Even"]
                .iter()
                .map(|value| value.to_string())
                .collect(),
            "Serial stop bits" => ["1", "2"].iter().map(|value| value.to_string()).collect(),
            _ => ["None", "Software", "Hardware"]
                .iter()
                .map(|value| value.to_string())
                .collect(),
        };
        let items: Vec<ListItem> = values
            .iter()
            .map(|value| ListItem::new(value.as_str()))
            .collect();
        let [title_area, list_area] = area.layout(&Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
        ]));
        let list = List::new(items).highlight_style(THEME.tabs_selected);
        let mut state = ListState::default();
        state.select(Some(self.selection.min(values.len() - 1)));
        frame.render_widget(Paragraph::new(label), title_area);
        frame.render_stateful_widget(list, list_area, &mut state);
    }

    fn render_list(&self, frame: &mut Frame, area: Rect, title: &str, values: &[&str]) {
        let items: Vec<ListItem> = values.iter().map(|value| ListItem::new(*value)).collect();
        let list = List::new(items).block(Block::default().title(title));
        let mut state = ListState::default();
        state.select(Some(self.selection.min(values.len() - 1)));
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn summary(&self) -> String {
        match self.source_kind {
            SourceKind::Tcp => format!("TCP {}:{}", self.tcp_host, self.tcp_port),
            SourceKind::Serial => format!("Serial {} at {} baud", self.serial_path, self.baudrate),
            SourceKind::File => format!("File {}", self.file_path.display()),
        }
    }
}

enum SetupAction {
    Continue,
    Cancel,
    Complete(DataSource),
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};

    fn enter() -> KeyEvent {
        KeyEvent::new_with_kind(KeyCode::Enter, KeyModifiers::NONE, KeyEventKind::Press)
    }

    #[test]
    fn default_tcp_source_is_constructed() {
        let mut setup = SourceSetup::default();
        setup.advance();
        setup.advance();
        setup.advance();

        assert!(
            matches!(setup.advance(), SetupAction::Complete(DataSource::Tcp(config))
            if config.host == "127.0.0.1" && config.port == 23000)
        );
    }

    #[test]
    fn invalid_tcp_port_keeps_the_same_step() {
        let mut setup = SourceSetup::default();
        setup.advance();
        setup.advance();
        setup.input = "not-a-port".to_string();

        setup.handle_key(enter());

        assert!(matches!(setup.step, Step::Text("TCP port")));
        assert_eq!(setup.error, Some("Enter a valid TCP port"));
    }
}
