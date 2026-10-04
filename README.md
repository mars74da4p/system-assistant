# System Assistant v1.1.0

A Windows-like Linux task manager styled as a terminal system dashboard with an AI assistant helper.

## Features

- Windows-like TUI layout with tabbed navigation
- Real CPU, Memory, Disk, and uptime monitoring
- Process list with sorting and management actions
- Service control via `systemctl`
- Assistant panel with built-in system guidance

## Controls

- Tab / Shift-Tab: switch tabs
- Up / Down: move through process or service list
- k: kill selected process
- n: lower niceness of selected process
- s: start selected service
- x: stop selected service
- R: restart selected service
- a: open assistant
- q: quit

## Run

```bash
cargo run
```
