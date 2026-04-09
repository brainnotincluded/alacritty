//! cmux - A Tiling Terminal Manager
//!
//! A GPU-accelerated terminal multiplexer with automatic tmux-style session management.
//! Built on Alacritty's rendering engine.

#![warn(rust_2018_idioms, future_incompatible)]
#![deny(clippy::all, clippy::if_not_else, clippy::enum_glob_use)]
#![cfg_attr(clippy, deny(warnings))]
// With the default subsystem, 'console', windows creates an additional console
// window for the program.
// This is silently ignored on non-windows systems.
// See https://msdn.microsoft.com/en-us/library/4cc7ya5b.aspx for more details.
#![windows_subsystem = "windows"]

#[cfg(not(any(feature = "x11", feature = "wayland", target_os = "macos", windows)))]
compile_error!(r#"at least one of the "x11"/"wayland" features must be enabled"#);

use std::error::Error;
use std::fmt::Write as _;
use std::io::{self, Write};
use std::path::PathBuf;
use std::{env, fs};

use log::info;
#[cfg(windows)]
use windows_sys::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole, FreeConsole};
use winit::event_loop::EventLoop;
#[cfg(target_os = "macos")]
use winit::platform::macos::{EventLoopBuilderExtMacOS, WindowExtMacOS};
#[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))]
use winit::raw_window_handle::{HasDisplayHandle, RawDisplayHandle};

use cmux_terminal::tty;

mod cli;
mod clipboard;
mod config;
mod control;
mod daemon;
mod display;
mod event;
mod input;
mod logging;
#[cfg(target_os = "macos")]
mod macos;
mod message_bar;
mod migrate;
mod multiplexer;
mod pane;
#[cfg(windows)]
mod panic;
#[cfg(unix)]
mod polling;
mod renderer;
mod scheduler;
mod session;
mod string;
mod window_context;

mod gl {
    #![allow(clippy::all, unsafe_op_in_unsafe_fn)]
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

#[cfg(unix)]
use crate::cli::MessageOptions;
#[cfg(unix)]
use crate::cli::SocketMessage;
use crate::cli::{Options, Subcommands, ControlOptions, ControlCommand};
use crate::config::UiConfig;
use crate::control::{ControlMessage, WindowControl, TerminalControl, ConfigControl, SessionControl, CursorControl, SelectionControl};
use crate::config::monitor::ConfigMonitor;
use crate::event::{Event, Processor};
#[cfg(target_os = "macos")]
use crate::macos::locale;
#[cfg(unix)]
use crate::polling::{IoListener, ipc};

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(windows)]
    panic::attach_handler();

    // When linked with the windows subsystem windows won't automatically attach
    // to the console of the parent process, so we do it explicitly. This fails
    // silently if the parent has no console.
    #[cfg(windows)]
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }

    // Load command line options.
    let options = Options::new();

    match options.subcommands {
        #[cfg(unix)]
        Some(Subcommands::Msg(options)) => msg(options)?,
        #[cfg(unix)]
        Some(Subcommands::Control(options)) => control_cmd(options)?,
        Some(Subcommands::Migrate(options)) => migrate::migrate(options),
        None => alacritty(options)?,
    }

    Ok(())
}

/// `msg` subcommand entrypoint.
#[cfg(unix)]
#[allow(unused_mut)]
fn msg(mut options: MessageOptions) -> Result<(), Box<dyn Error>> {
    #[cfg(not(any(target_os = "macos", windows)))]
    if let SocketMessage::CreateWindow(window_options) = &mut options.message {
        window_options.activation_token =
            env::var("XDG_ACTIVATION_TOKEN").or_else(|_| env::var("DESKTOP_STARTUP_ID")).ok();
    }
    ipc::send_message(options.socket, options.message).map_err(|err| err.into())
}

/// `control` subcommand entrypoint.
#[cfg(unix)]
fn control_cmd(options: ControlOptions) -> Result<(), Box<dyn Error>> {
    use crate::cli::{WindowCommand, TerminalCommand, ConfigCommand, SessionCommand, CursorCommand, SelectionCommand};
    
    // Convert ControlCommand to ControlMessage
    let control_message = match options.command {
        ControlCommand::Window(cmd) => {
            let window_control = match cmd {
                WindowCommand::Minimize => WindowControl::Minimize,
                WindowCommand::Maximize => WindowControl::Maximize,
                WindowCommand::Restore => WindowControl::Restore,
                WindowCommand::ToggleFullscreen => WindowControl::ToggleFullscreen,
                WindowCommand::Fullscreen { enabled } => WindowControl::SetFullscreen { enabled },
                WindowCommand::ToggleMaximized => WindowControl::ToggleMaximized,
                WindowCommand::Focus => WindowControl::Focus,
                WindowCommand::Urgent { urgent } => WindowControl::SetUrgent { urgent },
                WindowCommand::Title { title } => WindowControl::SetTitle { title },
                WindowCommand::Opacity { opacity } => WindowControl::SetOpacity { opacity },
                WindowCommand::Blur { blur } => WindowControl::SetBlur { blur },
                WindowCommand::Visible { visible } => WindowControl::SetVisible { visible },
                WindowCommand::Move { x, y } => WindowControl::SetPosition { x, y },
                WindowCommand::Resize { width, height } => WindowControl::SetSize { width, height },
                WindowCommand::Info => WindowControl::GetInfo,
                WindowCommand::Close => WindowControl::Close,
                WindowCommand::List => WindowControl::ListWindows,
            };
            ControlMessage::Window(window_control)
        },
        ControlCommand::Terminal(cmd) => {
            let terminal_control = match cmd {
                TerminalCommand::Send { text } => TerminalControl::SendText { text },
                TerminalCommand::Key { key } => TerminalControl::SendKey { key, mods: vec![] },
                TerminalCommand::ScrollUp { lines } => TerminalControl::ScrollUp { lines },
                TerminalCommand::ScrollDown { lines } => TerminalControl::ScrollDown { lines },
                TerminalCommand::ScrollTop => TerminalControl::ScrollToTop,
                TerminalCommand::ScrollBottom => TerminalControl::ScrollToBottom,
                TerminalCommand::Clear => TerminalControl::Clear,
                TerminalCommand::Copy => TerminalControl::CopySelection,
                TerminalCommand::Paste => TerminalControl::Paste,
                TerminalCommand::Content { start, end } => TerminalControl::GetContent { start, end },
                TerminalCommand::Size => TerminalControl::GetSize,
                TerminalCommand::Resize { cols, rows } => TerminalControl::SetSize { cols, rows },
            };
            ControlMessage::Terminal(terminal_control)
        },
        ControlCommand::Config(cmd) => {
            let config_control = match cmd {
                ConfigCommand::Reload => ConfigControl::Reload,
                ConfigCommand::Get => ConfigControl::GetConfig,
                ConfigCommand::Set { option, value } => ConfigControl::SetOption { option, value },
                ConfigCommand::Reset { option } => ConfigControl::ResetOption { option },
            };
            ControlMessage::Config(config_control)
        },
        ControlCommand::Session(cmd) => {
            let session_control = match cmd {
                SessionCommand::NewWindow { working_directory, command, title } => {
                    let mut opts = crate::cli::WindowOptions::default();
                    opts.terminal_options.working_directory = working_directory;
                    if let Some(cmd) = command {
                        opts.terminal_options.set_command(cmd, vec![]);
                    }
                    if let Some(t) = title {
                        opts.window_identity.title = Some(t);
                    }
                    SessionControl::CreateWindow { options: opts }
                },
                SessionCommand::List => SessionControl::ListWindows,
                SessionCommand::Active => SessionControl::GetActiveWindow,
                SessionCommand::Shutdown => SessionControl::Shutdown,
            };
            ControlMessage::Session(session_control)
        },
        ControlCommand::Cursor(cmd) => {
            let cursor_control = match cmd {
                CursorCommand::Pos => CursorControl::GetPosition,
                CursorCommand::Style { style } => CursorControl::SetStyle { style },
                CursorCommand::Blink { blinking } => CursorControl::SetBlinking { blinking },
            };
            ControlMessage::Cursor(cursor_control)
        },
        ControlCommand::Selection(cmd) => {
            let selection_control = match cmd {
                SelectionCommand::Get => SelectionControl::GetText,
                SelectionCommand::Clear => SelectionControl::Clear,
                SelectionCommand::All => SelectionControl::SelectAll,
            };
            ControlMessage::Selection(selection_control)
        },
    };
    
    ipc::send_message(options.socket, control_message).map_err(|err| err.into())
}

/// Temporary files stored for Alacritty.
///
/// This stores temporary files to automate their destruction through its `Drop` implementation.
struct TemporaryFiles {
    #[cfg(unix)]
    socket_path: Option<PathBuf>,
    log_file: Option<PathBuf>,
}

impl Drop for TemporaryFiles {
    fn drop(&mut self) {
        // Clean up the IPC socket file.
        #[cfg(unix)]
        if let Some(socket_path) = self.socket_path.as_deref() {
            let _ = fs::remove_file(socket_path);
        }

        // Clean up logfile.
        if let Some(log_file) = &self.log_file {
            if fs::remove_file(log_file).is_ok() {
                let _ = writeln!(io::stdout(), "Deleted log file at \"{}\"", log_file.display());
            }
        }
    }
}

/// Run main Alacritty entrypoint.
///
/// Creates a window, the terminal state, PTY, I/O event loop, input processor,
/// config change monitor, and runs the main display loop.
fn alacritty(mut options: Options) -> Result<(), Box<dyn Error>> {
    // Setup winit event loop.
    // On macOS, disable the default menu bar to prevent it from intercepting keyboard shortcuts.
    #[cfg(target_os = "macos")]
    let window_event_loop = EventLoop::<Event>::with_user_event()
        .with_default_menu(false)
        .build()?;
    #[cfg(not(target_os = "macos"))]
    let window_event_loop = EventLoop::<Event>::with_user_event().build()?;

    // Initialize the logger as soon as possible as to capture output from other subsystems.
    let log_file = logging::initialize(&options, window_event_loop.create_proxy())
        .expect("Unable to initialize logger");

    info!("Welcome to Alacritty");
    info!("Version {}", env!("VERSION"));

    #[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))]
    info!(
        "Running on {}",
        if matches!(
            window_event_loop.display_handle().unwrap().as_raw(),
            RawDisplayHandle::Wayland(_)
        ) {
            "Wayland"
        } else {
            "X11"
        }
    );
    #[cfg(not(any(feature = "x11", target_os = "macos", windows)))]
    info!("Running on Wayland");

    // Load configuration file.
    let config = config::load(&mut options);
    log_config_path(&config);

    // Update the log level from config.
    log::set_max_level(config.debug.log_level);

    // Set tty environment variables.
    tty::setup_env();

    // Set env vars from config.
    for (key, value) in config.env.iter() {
        unsafe { env::set_var(key, value) };
    }

    // Switch to home directory.
    #[cfg(target_os = "macos")]
    env::set_current_dir(home::home_dir().unwrap()).unwrap();

    // Set macOS locale.
    #[cfg(target_os = "macos")]
    locale::set_locale_environment();

    #[cfg(target_os = "macos")]
    macos::disable_autofill();

    // Spawn the Unix I/O event polling thread.
    #[cfg(unix)]
    let socket_path = match IoListener::spawn(&config, &options, window_event_loop.create_proxy()) {
        Ok(handle) => handle.ipc_socket_path,
        Err(err) if options.daemon => return Err(err.into()),
        Err(err) => {
            log::warn!("Unable to create socket: {err:?}");
            None
        },
    };

    // Setup automatic RAII cleanup for our files.
    let log_cleanup = log_file.filter(|_| !config.debug.persistent_logging);
    let _files = TemporaryFiles {
        #[cfg(unix)]
        socket_path,
        log_file: log_cleanup,
    };

    // Event processor.
    let mut processor = Processor::new(config, options, &window_event_loop);

    // Start event loop and block until shutdown.
    let result = processor.run(window_event_loop);

    // `Processor` must be dropped before calling `FreeConsole`.
    //
    // This is needed for ConPTY backend. Otherwise a deadlock can occur.
    // The cause:
    //   - Drop for ConPTY will deadlock if the conout pipe has already been dropped
    //   - ConPTY is dropped when the last of processor and window context are dropped, because both
    //     of them own an Arc<ConPTY>
    //
    // The fix is to ensure that processor is dropped first. That way, when window context (i.e.
    // PTY) is dropped, it can ensure ConPTY is dropped before the conout pipe in the PTY drop
    // order.
    //
    // FIXME: Change PTY API to enforce the correct drop order with the typesystem.

    // Terminate the config monitor.
    if let Some(config_monitor) = processor.config_monitor.take() {
        config_monitor.shutdown();
    }

    // Without explicitly detaching the console cmd won't redraw it's prompt.
    #[cfg(windows)]
    unsafe {
        FreeConsole();
    }

    info!("Goodbye");

    result
}

fn log_config_path(config: &UiConfig) {
    if config.config_paths.is_empty() {
        return;
    }

    let mut msg = String::from("Configuration files loaded from:");
    for path in &config.config_paths {
        let _ = write!(msg, "\n  {:?}", path.display());
    }

    info!("{msg}");
}
