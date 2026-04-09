//! Alacritty CLI Control Module
//!
//! Provides comprehensive CLI control over running Alacritty instances via IPC.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Window control commands
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum WindowControl {
    /// Minimize the window
    Minimize,
    /// Maximize the window
    Maximize,
    /// Restore the window from minimized/maximized state
    Restore,
    /// Toggle fullscreen mode
    ToggleFullscreen,
    /// Set fullscreen state
    SetFullscreen { enabled: bool },
    /// Toggle maximized state
    ToggleMaximized,
    /// Focus the window
    Focus,
    /// Set window urgency/highlight
    SetUrgent { urgent: bool },
    /// Set window title
    SetTitle { title: String },
    /// Set window opacity (0-1)
    SetOpacity { opacity: f64 },
    /// Set window blur
    SetBlur { blur: bool },
    /// Set window visibility
    SetVisible { visible: bool },
    /// Move window to position
    SetPosition { x: i32, y: i32 },
    /// Resize window
    SetSize { width: u32, height: u32 },
    /// Get window information
    GetInfo,
    /// Close the window
    Close,
    /// Get all window IDs
    ListWindows,
}

/// Terminal control commands
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TerminalControl {
    /// Send text to the terminal
    SendText { text: String },
    /// Send key sequence to the terminal
    SendKey { key: String, mods: Vec<String> },
    /// Scroll up by lines
    ScrollUp { lines: usize },
    /// Scroll down by lines
    ScrollDown { lines: usize },
    /// Scroll to top
    ScrollToTop,
    /// Scroll to bottom
    ScrollToBottom,
    /// Clear screen
    Clear,
    /// Copy selection to clipboard
    CopySelection,
    /// Paste from clipboard
    Paste,
    /// Get terminal content/lines
    GetContent { start: Option<usize>, end: Option<usize> },
    /// Get terminal size (cols x rows)
    GetSize,
    /// Set terminal size (cols x rows)
    SetSize { cols: usize, rows: usize },
}

/// Configuration control commands
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ConfigControl {
    /// Reload configuration from file
    Reload,
    /// Get current configuration
    GetConfig,
    /// Set configuration option
    SetOption { option: String, value: String },
    /// Reset configuration option to default
    ResetOption { option: String },
}

/// Session control commands
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SessionControl {
    /// Create new window
    CreateWindow { options: crate::cli::WindowOptions },
    /// List all windows with their IDs
    ListWindows,
    /// Get active window ID
    GetActiveWindow,
    /// Shutdown Alacritty daemon
    Shutdown,
}

/// Cursor control commands
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum CursorControl {
    /// Get cursor position (col, row)
    GetPosition,
    /// Set cursor style
    SetStyle { style: String },
    /// Set cursor blinking
    SetBlinking { blinking: bool },
}

/// Selection control commands
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SelectionControl {
    /// Get current selection text
    GetText,
    /// Clear selection
    Clear,
    /// Select all
    SelectAll,
}

/// Complete control message enum
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ControlMessage {
    Window(WindowControl),
    Terminal(TerminalControl),
    Config(ConfigControl),
    Session(SessionControl),
    Cursor(CursorControl),
    Selection(SelectionControl),
}

/// Control message response
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ControlResponse {
    /// Success response
    Ok,
    /// Success with text data
    Text(String),
    /// Success with window info
    WindowInfo {
        id: u64,
        title: String,
        size: (u32, u32),
        position: (i32, i32),
        is_focused: bool,
        is_maximized: bool,
        is_fullscreen: bool,
        is_minimized: bool,
    },
    /// Success with terminal size
    TerminalSize { cols: usize, rows: usize },
    /// Success with cursor position
    CursorPosition { col: usize, row: usize },
    /// Success with window list
    WindowList(Vec<u64>),
    /// Error response
    Error(String),
}

/// Window information structure
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowInfo {
    pub id: u64,
    pub title: String,
    pub working_directory: Option<PathBuf>,
}
