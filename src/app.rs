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
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Row, Table, Tabs},
    Frame, Terminal,
};

use crate::{
    assistant::Assistant,
    system::{
        execute_service_action, kill_process, set_process_priority, snapshot,
        ServiceInfo, SystemSnapshot,
    },
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
    process_index: usize,
    service_index: usize,
    status_message: String,
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
                "System Assistant v1.1.0 loaded.".to_string(),
                "Use: Tab, arrows, k=kill, n=nice, s=start, x=stop, r=restart".to_string(),
            ],
            focus_input: false,
            process_index: 0,
            service_index: 0,
            status_message: "Ready".to_string(),
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
            KeyCode::Up => {
                if self.selected_tab == 1 && self.process_index > 0 {
                    self.process_index -= 1;
                }
                if self.selected_tab == 2 && self.service_index > 0 {
                    self.service_index -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_tab == 1 {
                    self.process_index = (self.process_index + 1).min(self.snapshot.processes.len().saturating_sub(1));
                }
                if self.selected_tab == 2 {
                    self.service_index = (self.service_index + 1).min(self.snapshot.services.len().saturating_sub(1));
                }
            }
            KeyCode::Char('r') => {
                self.snapshot = snapshot();
                self.status_message = "System refreshed".to_string();
            }
            KeyCode::Char('k') if self.selected_tab == 1 => {
                if let Some(proc) = self.snapshot.processes.get(self.process_index) {
                    match kill_process(proc.pid) {
                        Ok(msg) => self.status_message = msg,
                        Err(err) => self.status_message = err.to_string(),
                    }
                }
            }
            KeyCode::Char('n') if self.selected_tab == 1 => {
                if let Some(proc) = self.snapshot.processes.get(self.process_index) {
                    match set_process_priority(proc.pid, 10) {
                        Ok(msg) => self.status_message = msg,
                        Err(err) => self.status_message = err.to_string(),
                    }
                }
            }
            KeyCode::Char('s') if self.selected_tab == 2 => {
                if let Some(service) = self.snapshot.services.get(self.service_index) {
                    match execute_service_action(&service.name, "start") {
                        Ok(msg) => self.status_message = msg,
                        Err(err) => self.status_message = err.to_string(),
                    }
                }
            }
            KeyCode::Char('x') if self.selected_tab == 2 => {
                if let Some(service) = self.snapshot.services.get(self.service_index) {
                    match execute_service_action(&service.name, "stop") {
                        Ok(msg) => self.status_message = msg,
                        Err(err) => self.status_message = err.to_string(),
                    }
                }
            }
            KeyCode::Char('R') if self.selected_tab == 2 => {
                if let Some(service) = self.snapshot.services.get(self.service_index) {
                    match execute_service_action(&service.name, "restart") {
                        Ok(msg) => self.status_message = msg,
                        Err(err) => self.status_message = err.to_string(),
                    }
                }
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
                Constraint::Length(4),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        let tabs = Tabs::new(self.tabs.iter().map(|t| t.to_string()).collect::<Vec<_>>())
            .select(self.selected_tab)
            .block(Block::default().title(" System Assistant ").borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)))
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(tabs, chunks[0]);

        let topbar = format!(
            "Host: {} | CPU: {:.1}% | RAM: {}MB / {}MB | Uptime: {} | {}",
            self.snapshot.hostname,
            self.snapshot.cpu_percent,
            self.snapshot.memory_used_mb,
            self.snapshot.memory_total_mb,
            self.snapshot.uptime_human,
            self.status_message
        );
        f.render_widget(
            Paragraph::new(topbar)
                .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray))),
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

        let cpu_percent = self.snapshot.cpu_percent.min(100.0) as u16;
        let cpu = Gauge::default()
            .block(Block::default().title(" CPU ").borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(cpu_percent);
        f.render_widget(cpu, left[0]);

        let mem_percent = ((self.snapshot.memory_used_mb as f64 / self.snapshot.memory_total_mb as f64) * 100.0).min(100.0) as u16;
        let mem = Gauge::default()
            .block(Block::default().title(" Memory ").borders(Borders::ALL).border_style(Style::default().fg(Color::Green)))
            .gauge_style(Style::default().fg(Color::Green))
            .percent(mem_percent);
        f.render_widget(mem, left[1]);

        let disk_lines: Vec<Line> = self
            .snapshot
            .disks
            .iter()
            .take(8)
            .map(|d| {
                let ratio = ((d.used_mb as f64 / d.total_mb as f64) * 100.0).min(100.0) as u16;
                Line::from(format!(
                    "{} {} | {}MB / {}MB | {}%",
                    d.name,
                    d.mount,
                    d.used_mb,
                    d.total_mb,
                    ratio
                ))
            })
            .collect();
        f.render_widget(
            Paragraph::new(disk_lines)
                .block(Block::default().title(" Disks ").borders(Borders::ALL).border_style(Style::default().fg(Color::Magenta))),
            left[2],
        );

        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),
                Constraint::Length(9),
                Constraint::Min(0),
            ])
            .split(columns[1]);

        let system_lines = vec![
            format!("Host: {}", self.snapshot.hostname),
            format!("Uptime: {}", self.snapshot.uptime_human),
            format!("RAM used: {} MB", self.snapshot.memory_used_mb),
            format!("RAM free: {} MB", self.snapshot.memory_free_mb),
            format!("Total processes: {}", self.snapshot.processes.len()),
            format!("Services: {}", self.snapshot.services.len()),
        ];
        f.render_widget(
            Paragraph::new(system_lines.join("\n"))
                .block(Block::default().title(" System ").borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow))),
            right[0],
        );

        let process_lines = self
            .snapshot
            .processes
            .iter()
            .take(8)
            .map(|p| format!("{} | PID {} | {} MB | {}%", p.name, p.pid, p.memory_mb, p.cpu as u16))
            .collect::<Vec<_>>()
            .join("\n");
        f.render_widget(
            Paragraph::new(process_lines)
                .block(Block::default().title(" Top Processes ").borders(Borders::ALL).border_style(Style::default().fg(Color::Blue))),
            right[1],
        );
    }

    fn render_processes(&self, f: &mut Frame, area: Rect) {
        let rows: Vec<Row> = self
            .snapshot
            .processes
            .iter()
            .enumerate()
            .map(|(idx, p)| {
                let style = if idx == self.process_index {
                    Style::default().fg(Color::Black).bg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };

                Row::new(vec![
                    p.name.clone(),
                    p.pid.to_string(),
                    format!("{:.1}", p.cpu),
                    format!("{}MB", p.memory_mb),
                    p.status.clone(),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(24),
                Constraint::Length(9),
                Constraint::Length(8),
                Constraint::Length(10),
                Constraint::Length(10),
            ],
        )
        .header(
            Row::new(vec!["Name", "PID", "CPU%", "Memory", "State"])
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        )
        .block(Block::default().title(" Processes ").borders(Borders::ALL).border_style(Style::default().fg(Color::Green)))
        .highlight_style(Style::default().bg(Color::Rgb(0, 128, 0)));
        f.render_widget(table, area);
    }

    fn render_services(&self, f: &mut Frame, area: Rect) {
        let rows: Vec<Row> = self
            .snapshot
            .services
            .iter()
            .enumerate()
            .map(|(idx, s)| {
                let style = if idx == self.service_index {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else {
                    Style::default().fg(Color::White)
                };
                Row::new(vec![s.name.clone(), s.status.clone()]).style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Length(32), Constraint::Length(18)],
        )
        .header(
            Row::new(vec!["Service", "State"])
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        )
        .block(Block::default().title(" Services ").borders(Borders::ALL).border_style(Style::default().fg(Color::Magenta)));

        f.render_widget(table, area);
    }

    fn render_assistant(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        let chat_lines: Vec<Line> = self
            .assistant_log
            .iter()
            .map(|line| Line::from(line.clone()))
            .collect();

        f.render_widget(
            Paragraph::new(chat_lines)
                .block(Block::default().title(" Assistant ").borders(Borders::ALL).border_style(Style::default().fg(Color::Blue))),
            chunks[0],
        );

        let input = format!("> {}", self.assistant_input);
        f.render_widget(
            Paragraph::new(input)
                .block(Block::default().title(" Input ").borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)))
                .style(Style::default().fg(Color::Cyan)),
            chunks[1],
        );
    }
}
