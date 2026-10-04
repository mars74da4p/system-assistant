# System Assistant v1.1.0

A Windows-like Linux task manager styled as a terminal system dashboard with an AI assistant helper.

## Features

- ✅ Windows-like TUI layout with tabbed navigation
- ✅ Real-time CPU, Memory, Disk, and uptime monitoring
- ✅ Process list with kill and priority control
- ✅ Service management via `systemctl` (start/stop/restart)
- ✅ AI Assistant panel with system diagnostics
- ✅ Interactive terminal interface with color-coded panels

## Requirements

- Linux/Unix system (macOS supported)
- Rust 1.70 or higher
- `systemctl` available (for service management)
- Terminal with 24+ color support

## Installation

### 1. Clone the repository

```bash
git clone https://github.com/mars74da4p/system-assistant.git
cd system-assistant
```

### 2. Build from source

```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized, smaller binary)
cargo build --release
```

### 3. Run the application

```bash
# Development mode
cargo run

# Release mode (faster execution)
cargo run --release

# Direct execution (after release build)
./target/release/system-assistant
```

## Usage Guide

### Tabs

- **Overview**: System statistics (CPU, RAM, Disk, top processes)
- **Processes**: Complete process list with filtering and management
- **Services**: Systemctl services with start/stop/restart controls
- **Assistant**: AI-powered system diagnostics and help

### Keyboard Controls

#### Navigation
| Key | Action |
|-----|--------|
| `Tab` | Next tab |
| `Shift+Tab` | Previous tab |
| `↑` `↓` | Navigate up/down in lists |
| `a` | Jump to Assistant tab |
| `q` | Quit application |

#### Process Tab Controls
| Key | Action |
|-----|--------|
| `k` | Kill selected process (SIGTERM) |
| `n` | Lower priority (nice value +10) |
| `r` | Refresh system data |

#### Services Tab Controls
| Key | Action |
|-----|--------|
| `s` | Start selected service |
| `x` | Stop selected service |
| `R` | Restart selected service |
| `r` | Refresh services list |

#### Assistant Tab
| Key | Action |
|-----|--------|
| Type | Input commands/questions |
| `Enter` | Send to assistant |
| `Backspace` | Delete character |
| `Esc` | Clear input |

### Assistant Commands

Built-in diagnostic commands:

```
cpu       - Show current CPU load
memory    - Display memory usage
ram       - Same as memory
disk      - Show disk usage
storage   - Same as disk
top       - Top memory-consuming processes
process   - Same as top
log       - Help with system logs
error     - Error diagnostics
status    - Overall system status
help      - List available commands
```

## Examples

### Running in debug mode
```bash
cargo run
```

### Running in release mode (recommended)
```bash
cargo build --release
./target/release/system-assistant
```

### Installing system-wide

```bash
# Build release version
cargo build --release

# Copy to local bin
cp target/release/system-assistant ~/.local/bin/
# or
sudo cp target/release/system-assistant /usr/local/bin/

# Now run from anywhere
system-assistant
```

## Architecture

```
src/
├── main.rs         - Entry point
├── app.rs          - TUI application with 4 tabs
├── system.rs       - System info & process/service control
└── assistant.rs    - AI assistant with local knowledge
```

## Features Breakdown

### System Monitoring
- CPU usage gauge
- Memory usage gauge
- Disk usage per partition
- System uptime and hostname
- Process count and status

### Process Management
- View all running processes
- Sort by memory usage
- Kill processes (SIGTERM)
- Adjust process priority (nice value)
- Real-time process updates

### Service Control
- List systemctl services
- Start services
- Stop services
- Restart services
- Real-time service status

### Assistant
- Local system diagnostics
- Memory and CPU analysis
- Disk space checks
- Process ranking
- Log file guidance
- Ready for Ollama/OpenAI integration

## Advanced Options

### Build for specific target
```bash
# For musl (smaller, portable binary)
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
./target/x86_64-unknown-linux-musl/release/system-assistant
```

### Debug with verbose output
```bash
RUST_BACKTRACE=1 cargo run
```

## Troubleshooting

### "Permission denied" when managing services
- Services require elevated privileges
- Run with `sudo` if needed: `sudo system-assistant`
- Or configure sudoers for specific commands

### "systemctl not found"
- Ensure you're on a systemd-based Linux distribution
- The app will still work but service tab will show errors

### High CPU usage on startup
- Normal during first system scan
- Stabilizes after initial frame rendering

## Optional: AI Integration

For advanced AI capabilities, install Ollama:

```bash
# Install Ollama from https://ollama.ai
curl https://ollama.ai/install.sh | sh

# Download a model
ollama pull llama2

# Start Ollama service
ollama serve
```

The assistant will automatically use Ollama if available at `http://127.0.0.1:11434`.

## Roadmap (v1.2+)

- [ ] Real-time CPU/RAM charts
- [ ] Process search and filtering
- [ ] Network monitoring
- [ ] System logs viewer
- [ ] Theme customization
- [ ] Configuration file support
- [ ] OpenAI/Claude API integration
- [ ] Custom commands

## License

MIT

## Contributing

Contributions welcome! Feel free to open issues or PRs.

## Support

For issues, questions, or feature requests: https://github.com/mars74da4p/system-assistant/issues
