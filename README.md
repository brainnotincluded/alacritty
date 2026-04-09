<p align="center">
    <img width="200" alt="cmux Logo" src="https://raw.githubusercontent.com/brainnotincluded/cmux/main/extra/logo/cmux-term.png">
</p>

<h1 align="center">cmux - A GPU-Accelerated Terminal Multiplexer</h1>

<p align="center">
  <strong>Built on Alacritty's blazing-fast GPU rendering engine</strong>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#configuration">Configuration</a> •
  <a href="#commands">Commands</a>
</p>

## About

**cmux** is a modern terminal multiplexer that combines Alacritty's GPU-accelerated rendering with the session management capabilities of tmux. It provides:

- **Pane Management**: Split windows horizontally and vertically
- **Session Persistence**: Detach and re-attach to sessions
- **Window Management**: Multiple windows per session with easy navigation
- **GPU Acceleration**: Lightning-fast rendering using OpenGL
- **CLI Control**: Comprehensive IPC for scripting and automation
- **Cross-Platform**: Works on Linux, macOS, BSD, and Windows

## Features

### Terminal Multiplexing
- Horizontal and vertical splits (`Ctrl-b %`, `Ctrl-b "`)
- Pane navigation (`Ctrl-b` + arrow keys)
- Pane resizing (`Ctrl-b` + `Ctrl-`arrow)
- Active pane highlighting with customizable borders

### Session Management
- Named sessions (`cmux new -s mysession`)
- Session listing (`cmux list-sessions`)
- Attach/detach (`cmux attach -t mysession`)
- Session persistence across disconnections

### Window Management
- Multiple windows per session (`Ctrl-b c`)
- Window status bar with indicators
- Window renaming (`Ctrl-b ,`)
- Window navigation (`Ctrl-b n`/`p`/`0-9`)

### IPC Control
- Full CLI control over running instances
- Create windows, send input, control panes remotely
- Perfect for automation and scripting

## Installation

### Prerequisites

- Rust 1.85.0 or later
- OpenGL ES 2.0 or higher
- On Windows: ConPTY support (Windows 10 version 1809+)

### From Source

```bash
git clone https://github.com/brainnotincluded/cmux.git
cd cmux
cargo build --release
sudo cp target/release/cmux /usr/local/bin/
```

### Prebuilt Binaries

Check the [releases page](https://github.com/brainnotincluded/cmux/releases) for prebuilt binaries.

## Quick Start

```bash
# Start a new session
cmux

# Start a named session
cmux new -s myproject

# List running sessions
cmux list-sessions

# Attach to an existing session
cmux attach -t myproject

# Start in daemon mode (headless)
cmux --daemon
```

### Key Bindings (Default Prefix: `Ctrl-b`)

| Key | Action |
|-----|--------|
| `Ctrl-b %` | Split vertically |
| `Ctrl-b "` | Split horizontally |
| `Ctrl-b` + arrow | Navigate panes |
| `Ctrl-b c` | Create new window |
| `Ctrl-b n` | Next window |
| `Ctrl-b p` | Previous window |
| `Ctrl-b 0-9` | Go to window |
| `Ctrl-b ,` | Rename window |
| `Ctrl-b d` | Detach session |
| `Ctrl-b ?` | Show all key bindings |

## Configuration

Configuration file locations (in order of precedence):

1. `$XDG_CONFIG_HOME/cmux/cmux.toml`
2. `$XDG_CONFIG_HOME/cmux.toml`
3. `$HOME/.config/cmux/cmux.toml`
4. `$HOME/.cmux.toml`
5. `/etc/cmux/cmux.toml`

On Windows: `%APPDATA%\cmux\cmux.toml`

### Example Configuration

```toml
# cmux.toml

# Session settings
[session]
# Automatically save and restore sessions
auto_save = true
# Default session name
default_name = "main"

# Pane styling
[panes]
# Pane border style: "single", "double", "heavy", "simple", "none"
border_style = "single"
# Active pane border color
active_border_color = "#ffaa00"
# Inactive pane border color
inactive_border_color = "#444444"
# Pane indicator position: "corner", "center", "off"
indicator_position = "corner"

# Status bar
[status_bar]
# Position: "top", "bottom", "off"
position = "bottom"
# Update interval in seconds
interval = 1
# Left side format
format_left = " #S "  # Session name
# Right side format
format_right = " %H:%M %d-%b-%y "
# Center format (window list)
format_center = " #W "

# Key bindings (prefix is Ctrl-b by default)
[key_bindings]
# Change prefix key
# prefix = "C-a"

# Custom key bindings
[[key_bindings.bind]]
key = "C-t"
command = "new-window"

[[key_bindings.bind]]
key = "C-s"
command = "split-window -v"
```

## Commands

### Session Commands

```bash
# Create new session
cmux new-session -s name
cmux new -s name

# List sessions
cmux list-sessions
cmux ls

# Attach to session
cmux attach-session -t name
cmux attach -t name

# Kill session
cmux kill-session -t name

# Rename session
cmux rename-session -t old_name new_name
```

### Window Commands

```bash
# Create window
cmux control session new-window --title "Window Name"

# List windows
cmux control session list

# Select window
cmux control window select --window-id <id>

# Rename window
cmux control window title --title "New Name"
```

### Pane Commands

```bash
# Split pane horizontally
cmux control pane split --direction horizontal

# Split pane vertically  
cmux control pane split --direction vertical

# Navigate to pane
cmux control pane select --direction up|down|left|right

# Resize pane
cmux control pane resize --direction right --amount 5

# Close pane
cmux control pane close
```

### Terminal Control

```bash
# Send text to terminal
cmux control terminal send --text "echo hello"

# Send key sequence
cmux control terminal key --key "C-c"

# Scroll
cmux control terminal scroll-up --lines 10
```

## Architecture

cmux is built on Alacritty's proven terminal emulation core with added multiplexer capabilities:

```
┌─────────────────────────────────────┐
│            cmux (GUI)               │
│  ┌─────────────────────────────┐   │
│  │    Window Manager           │   │
│  │  ┌─────┐ ┌─────┐ ┌─────┐   │   │
│  │  │Pane │ │Pane │ │Pane │   │   │
│  │  │  1  │ │  2  │ │  3  │   │   │
│  │  └──┬──┘ └──┬──┘ └──┬──┘   │   │
│  └─────┼───────┼───────┼──────┘   │
│        │       │       │          │
│  ┌─────┴───────┴───────┴──────┐   │
│  │   Session Manager          │   │
│  └────────────────────────────┘   │
└─────────────────────────────────────┘
            │
┌───────────┴─────────────────────────┐
│      cmux_terminal (Library)        │
│   - Terminal emulation (VTE)        │
│   - PTY management                  │
│   - Scrollback buffer               │
└─────────────────────────────────────┘
```

## Differences from tmux

| Feature | cmux | tmux |
|---------|------|------|
| Rendering | GPU-accelerated | CPU-based |
| Configuration | TOML | Custom syntax |
| IPC | Native CLI + socket | tmux command |
| True Color | Native | Requires config |
| Unicode | Full support | Full support |
| Performance | ~60fps rendering | Limited by terminal |

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details.

## License

cmux is released under the [Apache License, Version 2.0](LICENSE-APACHE).

## Acknowledgments

- Built on [Alacritty](https://github.com/alacritty/alacritty)'s excellent terminal emulation core
- Inspired by [tmux](https://github.com/tmux/tmux) for session management concepts
