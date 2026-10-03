# System Assistant v1.0

A beautiful Linux system monitor and task manager with TUI (Terminal User Interface) similar to Windows Task Manager, featuring an intelligent AI assistant helper.

## Features

- **Overview Tab**: Real-time CPU, Memory, Disk, and System statistics with gauges
- **Processes Tab**: List all running processes with CPU%, Memory usage, and status
- **Services Tab**: Monitor and manage system services
- **Assistant Tab**: Interactive AI helper for system diagnostics and commands

## Installation

```bash
git clone https://github.com/mars74da4p/system-assistant.git
cd system-assistant
cargo build --release
```

## Usage

```bash
cargo run
# or
./target/release/system-assistant
```

### Keyboard Shortcuts

- `Tab` / `Shift+Tab` - Switch between tabs
- `a` - Jump to Assistant tab
- `q` - Quit application
- `Esc` - Exit input mode

### Assistant Commands

- `cpu` - Show current CPU load
- `memory` / `ram` - Display memory usage
- `disk` / `storage` - Show disk usage
- `top` / `process` - List top memory-consuming processes
- `log` / `error` - Get help with system logs
- `status` - Overall system status
- `help` - List available commands

## Requirements

- Linux/Unix system
- Rust 1.70+
- Ollama (optional, for AI features): `https://ollama.ai`

## Optional: AI Integration

Install Ollama for local AI assistant capabilities:

```bash
curl https://ollama.ai/install.sh | sh
ollama run llama3.2
```

Then start System Assistant, and the AI will provide intelligent suggestions.

## Architecture

- `src/main.rs` - Entry point
- `src/app.rs` - TUI application logic and rendering
- `src/system.rs` - System information gathering
- `src/assistant.rs` - AI assistant with local knowledge and Ollama integration

## Roadmap (v1.1+)

- [ ] Process killing and priority management
- [ ] Service start/stop/restart functionality
- [ ] Real-time CPU and memory charts
- [ ] System log viewer
- [ ] Network monitoring
- [ ] Settings and theme customization
- [ ] OpenAI/Claude API integration
- [ ] Configuration file support

## License

MIT
