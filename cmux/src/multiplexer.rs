//! Multiplexer coordinator for cmux.
//!
//! This module brings together sessions, windows, and panes to provide
//! the complete terminal multiplexer functionality.

use std::collections::HashMap;
use std::sync::Arc;

use winit::window::WindowId;
use log::{info, debug};

use cmux_terminal::sync::FairMutex;
use cmux_terminal::term::Term;
use cmux_terminal::event_loop::Notifier;

use crate::session::{SessionManager, SessionId, SessionWindowId};
use crate::pane::{PaneManager, PaneId, SplitDirection, Direction as PaneDirection};
use crate::event::EventProxy;
use crate::config::UiConfig;

/// Direction for navigation.
pub use crate::pane::Direction;

/// Multiplexer events that need to be handled by the event loop.
#[derive(Debug, Clone)]
pub enum MultiplexerEvent {
    /// Create a new pane in the specified direction.
    SplitPane { window_id: SessionWindowId, direction: SplitDirection },
    /// Navigate to a pane.
    NavigatePane { window_id: SessionWindowId, direction: Direction },
    /// Close the active pane.
    ClosePane { window_id: SessionWindowId },
    /// Create a new window in the session.
    CreateWindow { session_id: SessionId },
    /// Close a window.
    CloseWindow { window_id: SessionWindowId },
    /// Select next window.
    NextWindow { session_id: SessionId },
    /// Select previous window.
    PreviousWindow { session_id: SessionId },
    /// Select window by index.
    SelectWindow { session_id: SessionId, index: usize },
    /// Rename the current window.
    RenameWindow { window_id: SessionWindowId, name: String },
    /// Create a new session.
    CreateSession { name: String },
    /// Attach to a session.
    AttachSession { name: String },
    /// Detach from the current session.
    DetachSession,
    /// Kill a session.
    KillSession { name: String },
    /// Rename the current session.
    RenameSession { name: String },
}

/// Manages the multiplexer state across all windows.
pub struct Multiplexer {
    /// Session manager.
    pub sessions: SessionManager,
    /// Pane managers for each window (SessionWindowId -> PaneManager).
    pub pane_managers: HashMap<SessionWindowId, PaneManager>,
    /// Mapping from our SessionWindowId to winit WindowId.
    pub window_id_map: HashMap<SessionWindowId, WindowId>,
    /// Reverse mapping from winit WindowId to our SessionWindowId.
    pub reverse_window_map: HashMap<WindowId, SessionWindowId>,
    /// Next internal window ID.
    next_window_id: u64,
}

impl Default for Multiplexer {
    fn default() -> Self {
        Self {
            sessions: SessionManager::new(),
            pane_managers: HashMap::new(),
            window_id_map: HashMap::new(),
            reverse_window_map: HashMap::new(),
            next_window_id: 1,
        }
    }
}

impl Multiplexer {
    /// Create a new multiplexer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize with a default session.
    pub fn initialize_default_session(&mut self) -> Result<SessionId, String> {
        let session_id = self.sessions.create_session("0")?;
        info!("Created default session with ID {:?}", session_id);
        
        self.sessions.attach_session(session_id)?;
        
        Ok(session_id)
    }

    /// Create a new window with pane management.
    pub fn create_window(
        &mut self,
        winit_id: WindowId,
        terminal: Arc<FairMutex<Term<EventProxy>>>,
        notifier: Notifier,
    ) -> Option<SessionWindowId> {
        // Get the attached session.
        let session_id = self.sessions.attached_session()?;
        
        // Create internal window ID.
        let internal_id = SessionWindowId(self.next_window_id);
        self.next_window_id += 1;

        // Create the window in the session.
        let window_id = self.sessions.create_window(session_id, &format!("{}", internal_id.0))?;
        
        // Create pane manager for this window.
        let mut pane_manager = PaneManager::new();
        pane_manager.create_root(terminal, notifier, winit_id);
        
        self.pane_managers.insert(window_id, pane_manager);
        self.window_id_map.insert(window_id, winit_id);
        self.reverse_window_map.insert(winit_id, window_id);

        debug!("Created window {:?} with winit ID {:?}", window_id, winit_id);
        
        Some(window_id)
    }

    /// Get the SessionWindowId from a winit WindowId.
    pub fn get_window_id(&self, winit_id: WindowId) -> Option<SessionWindowId> {
        self.reverse_window_map.get(&winit_id).copied()
    }

    /// Get the winit WindowId from our SessionWindowId.
    pub fn get_winit_id(&self, window_id: SessionWindowId) -> Option<WindowId> {
        self.window_id_map.get(&window_id).copied()
    }

    /// Split the active pane in a window.
    pub fn split_pane(
        &mut self,
        window_id: SessionWindowId,
        direction: SplitDirection,
        new_terminal: Arc<FairMutex<Term<EventProxy>>>,
        new_notifier: Notifier,
    ) -> Option<PaneId> {
        let pane_manager = self.pane_managers.get_mut(&window_id)?;
        let active_pane = pane_manager.active_pane()?;
        
        let new_pane_id = pane_manager.split_pane(
            active_pane,
            direction,
            new_terminal,
            new_notifier,
        )?;

        // Update window's pane list.
        if let Some(window) = self.sessions.get_window_mut(window_id) {
            window.add_pane(new_pane_id);
            window.active_pane = Some(new_pane_id);
        }

        info!("Split pane {:?} in window {:?}", new_pane_id, window_id);
        
        Some(new_pane_id)
    }

    /// Navigate to a pane in the specified direction.
    pub fn navigate_pane(&mut self, window_id: SessionWindowId, direction: Direction) -> Option<PaneId> {
        let pane_manager = self.pane_managers.get_mut(&window_id)?;
        let new_pane = pane_manager.navigate(direction)?;

        // Update window's active pane.
        if let Some(window) = self.sessions.get_window_mut(window_id) {
            window.active_pane = Some(new_pane);
        }

        Some(new_pane)
    }

    /// Close a pane.
    pub fn close_pane(&mut self, window_id: SessionWindowId) -> bool {
        let pane_manager = match self.pane_managers.get_mut(&window_id) {
            Some(pm) => pm,
            None => return false,
        };

        let active_pane = match pane_manager.active_pane() {
            Some(id) => id,
            None => return false,
        };

        // Get the pane count before closing.
        let pane_count = pane_manager.len();

        // Close the pane.
        if !pane_manager.close_pane(active_pane) {
            return false;
        }

        // Update window.
        if let Some(window) = self.sessions.get_window_mut(window_id) {
            window.remove_pane(active_pane);
            window.active_pane = pane_manager.active_pane();
        }

        // If no panes left, close the window.
        if pane_count <= 1 {
            self.close_window(window_id);
        }

        true
    }

    /// Close a window.
    pub fn close_window(&mut self, window_id: SessionWindowId) -> bool {
        // Remove pane manager.
        self.pane_managers.remove(&window_id);

        // Remove window mappings.
        if let Some(winit_id) = self.window_id_map.remove(&window_id) {
            self.reverse_window_map.remove(&winit_id);
        }

        // Remove from session.
        let was_killed = self.sessions.kill_window(window_id);

        info!("Closed window {:?}", window_id);

        was_killed
    }

    /// Remove a window by its winit WindowId.
    pub fn remove_window_by_winit(&mut self, winit_id: WindowId) -> Option<SessionWindowId> {
        let window_id = self.reverse_window_map.remove(&winit_id)?;
        self.window_id_map.remove(&window_id);
        self.pane_managers.remove(&window_id);
        
        self.sessions.kill_window(window_id);
        
        Some(window_id)
    }

    /// Get the active pane for a window.
    pub fn active_pane(&self, window_id: SessionWindowId) -> Option<PaneId> {
        let pane_manager = self.pane_managers.get(&window_id)?;
        pane_manager.active_pane()
    }

    /// Get the pane manager for a window.
    pub fn pane_manager(&self, window_id: SessionWindowId) -> Option<&PaneManager> {
        self.pane_managers.get(&window_id)
    }

    /// Get the mutable pane manager for a window.
    pub fn pane_manager_mut(&mut self, window_id: SessionWindowId) -> Option<&mut PaneManager> {
        self.pane_managers.get_mut(&window_id)
    }

    /// Get the active pane for a winit window.
    pub fn active_pane_for_winit(&self, winit_id: WindowId) -> Option<PaneId> {
        let window_id = self.get_window_id(winit_id)?;
        self.active_pane(window_id)
    }

    /// Get the active window for the attached session.
    pub fn active_window(&self) -> Option<SessionWindowId> {
        self.sessions.attached()?.active_window()
    }

    /// Select the next window.
    pub fn next_window(&mut self) -> Option<SessionWindowId> {
        self.sessions.next_window()
    }

    /// Select the previous window.
    pub fn previous_window(&mut self) -> Option<SessionWindowId> {
        self.sessions.previous_window()
    }

    /// Select a window by index.
    pub fn select_window(&mut self, index: usize) -> Option<SessionWindowId> {
        self.sessions.select_window(index)
    }

    /// Rename the current window.
    pub fn rename_window(&mut self, name: impl Into<String>) -> bool {
        let window_id = match self.active_window() {
            Some(id) => id,
            None => return false,
        };

        self.sessions.rename_window(window_id, name)
    }

    /// Create a new session.
    pub fn create_session(&mut self, name: impl Into<String>) -> Result<SessionId, String> {
        self.sessions.create_session(name)
    }

    /// Attach to a session.
    pub fn attach_session(&mut self, name: impl Into<String>) -> Result<(), String> {
        let name = name.into();
        
        let session_id = self.sessions
            .get_session_by_name(&name)
            .map(|s| s.id)
            .ok_or_else(|| format!("Session '{}' not found", name))?;

        self.sessions.attach_session(session_id)
    }

    /// Detach from the current session.
    pub fn detach_session(&mut self) -> Option<SessionId> {
        self.sessions.detach_session()
    }

    /// List all sessions.
    pub fn list_sessions(&self) -> String {
        let sessions = self.sessions.list_sessions();
        
        if sessions.is_empty() {
            return "no server running on /tmp/cmux".to_string();
        }

        let mut output = String::new();
        for session in sessions {
            let attached_indicator = if session.is_attached { "*" } else { " " };
            let window_count = session.window_count();
            let created = format!("{:?}", session.created_at.elapsed().unwrap_or_default());
            
            output.push_str(&format!(
                "{} {}: {} window{} (created {})\n",
                attached_indicator,
                session.name,
                window_count,
                if window_count == 1 { "" } else { "s" },
                created
            ));
        }
        
        output
    }

    /// Kill a session.
    pub fn kill_session(&mut self, name: impl Into<String>) -> Result<(), String> {
        let name = name.into();
        
        let session_id = self.sessions
            .get_session_by_name(&name)
            .map(|s| s.id)
            .ok_or_else(|| format!("Session '{}' not found", name))?;

        self.sessions.kill_session(session_id);
        Ok(())
    }

    /// Rename the current session.
    pub fn rename_session(&mut self, name: impl Into<String>) -> Result<(), String> {
        let session_id = self.sessions.attached_session()
            .ok_or_else(|| "No session attached".to_string())?;

        self.sessions.rename_session(session_id, name)
    }

    /// Send input to the active pane of a window.
    pub fn send_input(&self, window_id: SessionWindowId, input: &[u8]) -> bool {
        let pane_manager = match self.pane_managers.get(&window_id) {
            Some(pm) => pm,
            None => return false,
        };

        let active_pane = match pane_manager.active() {
            Some(pane) => pane,
            None => return false,
        };

        let _ = active_pane.notifier.0.send(
            cmux_terminal::event_loop::Msg::Input(input.to_vec().into())
        );

        true
    }

    /// Get the number of windows.
    pub fn window_count(&self) -> usize {
        self.pane_managers.len()
    }

    /// Check if there are any windows.
    pub fn has_windows(&self) -> bool {
        !self.pane_managers.is_empty()
    }

    /// Get all window IDs.
    pub fn window_ids(&self) -> impl Iterator<Item = &SessionWindowId> {
        self.pane_managers.keys()
    }

    /// Handle a multiplexer event.
    pub fn handle_event(&mut self, event: MultiplexerEvent) -> Option<MultiplexerResult> {
        match event {
            MultiplexerEvent::CreateSession { name } => {
                match self.create_session(&name) {
                    Ok(id) => Some(MultiplexerResult::SessionCreated(id)),
                    Err(e) => Some(MultiplexerResult::Error(e)),
                }
            }
            MultiplexerEvent::AttachSession { name } => {
                match self.attach_session(name) {
                    Ok(()) => Some(MultiplexerResult::Success),
                    Err(e) => Some(MultiplexerResult::Error(e)),
                }
            }
            MultiplexerEvent::DetachSession => {
                self.detach_session();
                Some(MultiplexerResult::ShouldExit)
            }
            MultiplexerEvent::KillSession { name } => {
                match self.kill_session(name) {
                    Ok(()) => Some(MultiplexerResult::Success),
                    Err(e) => Some(MultiplexerResult::Error(e)),
                }
            }
            MultiplexerEvent::RenameSession { name } => {
                match self.rename_session(name) {
                    Ok(()) => Some(MultiplexerResult::Success),
                    Err(e) => Some(MultiplexerResult::Error(e)),
                }
            }
            MultiplexerEvent::CreateWindow { session_id: _ } => {
                // Window creation is handled by the window creation flow.
                Some(MultiplexerResult::Success)
            }
            MultiplexerEvent::CloseWindow { window_id } => {
                self.close_window(window_id);
                Some(MultiplexerResult::Success)
            }
            MultiplexerEvent::NextWindow { session_id: _ } => {
                self.next_window();
                Some(MultiplexerResult::Success)
            }
            MultiplexerEvent::PreviousWindow { session_id: _ } => {
                self.previous_window();
                Some(MultiplexerResult::Success)
            }
            MultiplexerEvent::SelectWindow { session_id: _, index } => {
                self.select_window(index);
                Some(MultiplexerResult::Success)
            }
            MultiplexerEvent::RenameWindow { window_id, name } => {
                self.sessions.rename_window(window_id, name);
                Some(MultiplexerResult::Success)
            }
            MultiplexerEvent::SplitPane { window_id: _, direction: _ } => {
                // Pane splitting requires terminal/notifier, handled separately.
                None
            }
            MultiplexerEvent::NavigatePane { window_id, direction } => {
                self.navigate_pane(window_id, direction);
                Some(MultiplexerResult::Success)
            }
            MultiplexerEvent::ClosePane { window_id } => {
                self.close_pane(window_id);
                Some(MultiplexerResult::Success)
            }
        }
    }
}

/// Result of handling a multiplexer event.
#[derive(Debug, Clone)]
pub enum MultiplexerResult {
    /// Operation succeeded.
    Success,
    /// A new session was created.
    SessionCreated(SessionId),
    /// An error occurred.
    Error(String),
    /// Should exit/detach.
    ShouldExit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiplexer_creation() {
        let mux = Multiplexer::new();
        assert!(!mux.has_windows());
        assert_eq!(mux.window_count(), 0);
    }

    #[test]
    fn test_default_session() {
        let mut mux = Multiplexer::new();
        let id = mux.initialize_default_session().unwrap();
        
        assert_eq!(mux.sessions.attached_session(), Some(id));
    }
}
