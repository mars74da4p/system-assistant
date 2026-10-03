use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Row, Table, Tabs},
    Frame, Terminal,
};

use crate::{
    assistant::Assistant,
    system::{snapshot, SystemSnapshot},
};

pub struct App {
    running: bool,
    selected_tab: usize,
    tabs: Vec<&'static str>,
    snapshot: SystemSnapshot,
    assistant: Assistant,
    assistant_input: String,
    assistant_log: Vec<String>,
    focus_input: bool,
}

impl App {
    pub fn run() -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        let mut app = App::new();

        while app.running {
            app.snapshot = snapshot();
            terminal.draw(|f| app.render(f))?;

            if event::poll(Duration::from_millis(200))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        app.handle_key(key.code);
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }

    fn new() -> Self {
        Self {
            running: true,
            selected_tab: 0,
            tabs: vec!["Overview", "Processes", "Services", "Assistant"],
            snapshot: snapshot(),
            assistant: Assistant::new(),
            assistant_input: String::new(),
            assistant_log: vec![
                "Добро пожаловать в System Assistant.".to_string(),
                "Команды: cpu, memory, disk, top, process, status, help".to_string(),
            ],
            focus_input: false,
        }
    }

    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.running = false,
            KeyCode::Tab => {
                self.selected_tab = (self.selected_tab + 1) % self.tabs.len();
                self.focus_input = false;
            }
            KeyCode::BackTab => {
                self.selected_tab = (self.selected_tab + self.tabs.len() - 1) % self.tabs.len();
                self.focus_input = false;
            }
            KeyCode::Char('a') => {
                self.selected_tab = 3;
                self.focus_input = true;
            }
            KeyCode::Char(c) => {
                if self.focus_input || self.selected_tab == 3 {
                    self.assistant_input.push(c);
                    self.focus_input = true;
                }
            }
            KeyCode::Backspace => {
                if self.focus_input || self.selected_tab == 3 {
                    self.assistant_input.pop();
                }
            }
            KeyCode::Enter => {
                if self.selected_tab == 3 {
                    let prompt = self.assistant_input.trim().to_string();
                    if !prompt.is_empty() {
                        let answer = self.assistant.answer_local(&prompt, &self.snapshot);
                        self.assistant_log.push(format!("> {}", prompt));
                        self.assistant_log.push(answer.clone());
                        self.assistant_input.clear();
                        self.focus_input = true;
                    }
                }
            }
            KeyCode::Esc => {
                self.focus_input = false;
            }
            _ => {}
        }
    }

    fn render(&mut self, f: &mut Frame) {
        let area = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(area);

        let tabs = Tabs::new(self.tabs.iter().map(|t| t.to_string()).collect::<Vec<_>>())
            .select(self.selected_tab)
            .block(Block::default().title("System Assistant").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(tabs, chunks[0]);

        let status = format!(
            "Host: {} | CPU: {:.1}% | RAM: {}MB / {}MB | Uptime: {}",
            self.snapshot.hostname,
            self.snapshot.cpu_percent,
            self.snapshot.memory_used_mb,
            self.snapshot.memory_total_mb,
            self.snapshot.uptime_human
        );
        f.render_widget(
            Paragraph::new(status)
                .style(Style::default().fg(Color::Green))
                .block(Block::default().borders(Borders::BOTTOM)),
            chunks[1],
        );

        match self.selected_tab {
            0 => self.render_overview(f, chunks[2]),
            1 => self.render_processes(f, chunks[2]),
            2 => self.render_services(f, chunks[2]),
            3 => self.render_assistant(f, chunks[2]),
            _ => {}
        }
    }

    fn render_overview(&self, f: &mut Frame, area: Rect) {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(area);

        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),
                Constraint::Length(7),
                Constraint::Min(0),
            ])
            .split(columns[0]);

        let cpu = Gauge::default()
            .block(Block::default().title("CPU").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(self.snapshot.cpu_percent as u16);
        f.render_widget(cpu, left[0]);

        let mem_percent = ((self.snapshot.memory_used_mb as f64 / self.snapshot.memory_total_mb as f64) * 100.0) as u16;
        let mem = Gauge::default()
            .block(Block::default().title("Memory").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Green))
            .percent(mem_percent);
        f.render_widget(mem, left[1]);

        let disk_blocks: Vec<Line> = self
            .snapshot
            .disks
            .iter()
            .take(6)
            .map(|d| {
                let ratio = ((d.used_mb as f64 / d.total_mb as f64) * 100.0) as u16;
                Line::from(format!(
                    "{} {}: {}MB / {}MB [{}%]",
                    d.name,
                    d.mount,
                    d.used_mb,
                    d.total_mb,
                    ratio.min(100)
                ))
                )
            })
            .collect();
        f.render_widget(
            Paragraph::new(disk_blocks)
                .block(Block::default().title("Disks").borders(Borders::ALL)),
            left[2],
        );

        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),
                Constraint::Length(7),
                Constraint::Min(0),
            ])
            .split(columns[1]);

        let system_summary = vec![
            format!("Host: {}", self.snapshot.hostname),
            format!("Uptime: {}", self.snapshot.uptime_human),
            format!("RAM used: {}MB", self.snapshot.memory_used_mb),
            format!("RAM free: {}MB", self.snapshot.memory_free_mb),
            format!("Processes: {}", self.snapshot.processes.len()),
        ];
        f.render_widget(
            Paragraph::new(system_summary.join("\n"))
                .block(Block::default().title("System").borders(Borders::ALL)),
            right[0],
        );

        let process_text = self
            .snapshot
            .processes
            .iter()
            .take(8)
            .map(|p| format!("{} | PID {} | {}MB | {}%", p.name, p.pid, p.memory_mb, p.cpu as u16))
            .collect::<Vec<_>>()
            .join("\n");

        f.render_widget(
            Paragraph::new(process_text)
                .block(Block::default().title("Top Processes").borders(Borders::ALL)),
            right[1],
        );
    }

    fn render_processes(&self, f: &mut Frame, area: Rect) {
        let rows: Vec<Row> = self
            .snapshot
            .processes
            .iter()
            .map(|p| {
                Row::new(vec![
                    p.name.clone(),
                    p.pid.to_string(),
                    format!("{:.1}", p.cpu),
                    format!("{}MB", p.memory_mb),
                    p.status.clone(),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(22),
                Constraint::Length(8),
                Constraint::Length(8),
                Constraint::Length(10),
                Constraint::Length(10),
            ],
        )
        .header(
            Row::new(vec!["Name", "PID", "CPU%", "Memory", "State"])
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        )
        .block(Block::default().title("Processes").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::DarkGray));
        f.render_widget(table, area);
    }

    fn render_services(&self, f: &mut Frame, area: Rect) {
        let services = vec![
            "ssh   active",
            "network active",
            "cups  inactive",
            "nginx active",
            "docker active",
            "bluetooth inactive",
        ];

        let text = services.join("\n");
        f.render_widget(
            Paragraph::new(text).block(Block::default().title("Services").borders(Borders::ALL)),
            area,
        );
    }

    fn render_assistant(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        let chat_lines = self
            .assistant_log
            .iter()
            .map(|line| Line::from(line.clone()))
            .collect::<Vec<_>>();

        f.render_widget(
            Paragraph::new(chat_lines)
                .block(Block::default().title("Assistant").borders(Borders::ALL)),
            chunks[0],
        );

        let input = format!("> {}", self.assistant_input);
        f.render_widget(
            Paragraph::new(input)
                .block(Block::default().title("Input").borders(Borders::ALL))
                .style(Style::default().fg(Color::Cyan)),
            chunks[1],
        );
    }
}
