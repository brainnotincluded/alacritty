//! Session management for cmux terminal multiplexer.
//!
//! Sessions are collections of windows that can be attached/detached.
//! They enable persistent terminal environments across connections.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::pane::PaneId;

/// Unique identifier for a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub u64);

impl From<SessionId> for u64 {
    fn from(id: SessionId) -> u64 {
        id.0
    }
}

impl From<u64> for SessionId {
    fn from(id: u64) -> SessionId {
        SessionId(id)
    }
}

/// Session information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session ID.
    pub id: SessionId,
    /// Session name.
    pub name: String,
    /// When the session was created.
    pub created_at: SystemTime,
    /// When the session was last attached.
    pub last_attached: Option<SystemTime>,
    /// Whether the session is currently attached.
    pub is_attached: bool,
    /// IDs of windows in this session.
    pub windows: Vec<SessionWindowId>,
    /// Currently active window index.
    pub active_window_index: usize,
    /// Whether this session should be killed when detached.
    pub kill_on_detach: bool,
    /// Working directory for new windows.
    pub working_directory: Option<PathBuf>,
    /// Environment variables.
    pub environment: HashMap<String, String>,
}

impl Session {
    /// Create a new session.
    pub fn new(id: SessionId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            created_at: SystemTime::now(),
            last_attached: None,
            is_attached: false,
            windows: Vec::new(),
            active_window_index: 0,
            kill_on_detach: false,
            working_directory: None,
            environment: HashMap::new(),
        }
    }

    /// Get the active window ID.
    pub fn active_window(&self) -> Option<SessionWindowId> {
        self.windows.get(self.active_window_index).copied()
    }

    /// Set the active window.
    pub fn set_active_window(&mut self, window_id: SessionWindowId) -> bool {
        if let Some(index) = self.windows.iter().position(|&w| w == window_id) {
            self.active_window_index = index;
            true
        } else {
            false
        }
    }

    /// Add a window to the session.
    pub fn add_window(&mut self, window_id: SessionWindowId) -> usize {
        self.windows.push(window_id);
        self.windows.len() - 1
    }

    /// Remove a window from the session.
    pub fn remove_window(&mut self, window_id: SessionWindowId) -> bool {
        if let Some(index) = self.windows.iter().position(|&w| w == window_id) {
            self.windows.remove(index);
            
            // Adjust active window index.
            if !self.windows.is_empty() && self.active_window_index >= self.windows.len() {
                self.active_window_index = self.windows.len() - 1;
            }
            
            true
        } else {
            false
        }
    }

    /// Get the number of windows.
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    /// Mark the session as attached.
    pub fn attach(&mut self) {
        self.is_attached = true;
        self.last_attached = Some(SystemTime::now());
    }

    /// Mark the session as detached.
    pub fn detach(&mut self) {
        self.is_attached = false;
    }
}

/// Window identifier within a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionWindowId(pub u64);

impl From<SessionWindowId> for u64 {
    fn from(id: SessionWindowId) -> u64 {
        id.0
    }
}

impl From<u64> for SessionWindowId {
    fn from(id: u64) -> SessionWindowId {
        SessionWindowId(id)
    }
}

/// Window information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    /// Window ID.
    pub id: SessionWindowId,
    /// Window name/title.
    pub name: String,
    /// Session this window belongs to.
    pub session_id: SessionId,
    /// Pane IDs in this window.
    pub panes: Vec<PaneId>,
    /// Currently active pane.
    pub active_pane: Option<PaneId>,
    /// Window index within the session.
    pub index: usize,
    /// Whether the window is zoomed (one pane fills the window).
    pub is_zoomed: bool,
    /// When the window was created.
    pub created_at: SystemTime,
    /// Whether this window has activity since last viewed.
    pub has_activity: bool,
    /// Whether this window has a bell since last viewed.
    pub has_bell: bool,
    /// Whether this window is silent (no activity).
    pub is_silent: bool,
}

impl Window {
    /// Create a new window.
    pub fn new(id: SessionWindowId, name: impl Into<String>, session_id: SessionId) -> Self {
        Self {
            id,
            name: name.into(),
            session_id,
            panes: Vec::new(),
            active_pane: None,
            index: 0,
            is_zoomed: false,
            created_at: SystemTime::now(),
            has_activity: false,
            has_bell: false,
            is_silent: false,
        }
    }

    /// Add a pane to the window.
    pub fn add_pane(&mut self, pane_id: PaneId) {
        self.panes.push(pane_id);
        if self.active_pane.is_none() {
            self.active_pane = Some(pane_id);
        }
    }

    /// Remove a pane from the window.
    pub fn remove_pane(&mut self, pane_id: PaneId) -> bool {
        if let Some(index) = self.panes.iter().position(|&p| p == pane_id) {
            self.panes.remove(index);
            
            // Update active pane.
            if self.active_pane == Some(pane_id) {
                self.active_pane = self.panes.first().copied();
            }
            
            true
        } else {
            false
        }
    }

    /// Get the number of panes.
    pub fn pane_count(&self) -> usize {
        self.panes.len()
    }
}

/// Manages all sessions.
pub struct SessionManager {
    /// All sessions by ID.
    sessions: HashMap<SessionId, Session>,
    /// All windows by ID.
    windows: HashMap<SessionWindowId, Window>,
    /// Session name to ID mapping.
    session_names: HashMap<String, SessionId>,
    /// Next session ID.
    next_session_id: u64,
    /// Next window ID.
    next_window_id: u64,
    /// Currently attached session (if any).
    attached_session: Option<SessionId>,
    /// Session storage path for persistence.
    storage_path: Option<PathBuf>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            windows: HashMap::new(),
            session_names: HashMap::new(),
            next_session_id: 1,
            next_window_id: 1,
            attached_session: None,
            storage_path: None,
        }
    }
}

impl SessionManager {
    /// Create a new session manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the storage path for session persistence.
    pub fn set_storage_path(&mut self, path: PathBuf) {
        self.storage_path = Some(path);
    }

    /// Create a new session.
    pub fn create_session(&mut self, name: impl Into<String>) -> Result<SessionId, String> {
        let name = name.into();
        
        // Check for duplicate names.
        if self.session_names.contains_key(&name) {
            return Err(format!("Session '{}' already exists", name));
        }

        let id = SessionId(self.next_session_id);
        self.next_session_id += 1;

        let session = Session::new(id, &name);
        self.session_names.insert(name, id);
        self.sessions.insert(id, session);

        Ok(id)
    }

    /// Get a session by ID.
    pub fn get_session(&self, id: SessionId) -> Option<&Session> {
        self.sessions.get(&id)
    }

    /// Get a session by ID (mutable).
    pub fn get_session_mut(&mut self, id: SessionId) -> Option<&mut Session> {
        self.sessions.get_mut(&id)
    }

    /// Get a session by name.
    pub fn get_session_by_name(&self, name: &str) -> Option<&Session> {
        self.session_names.get(name).and_then(|&id| self.sessions.get(&id))
    }

    /// Get a session by name (mutable).
    pub fn get_session_by_name_mut(&mut self, name: &str) -> Option<&mut Session> {
        self.session_names.get(name).copied()
            .and_then(|id| self.sessions.get_mut(&id))
    }

    /// Kill a session.
    pub fn kill_session(&mut self, id: SessionId) -> bool {
        if let Some(session) = self.sessions.remove(&id) {
            self.session_names.remove(&session.name);
            
            // Clean up windows.
            for window_id in &session.windows {
                self.windows.remove(window_id);
            }
            
            // Detach if this was the attached session.
            if self.attached_session == Some(id) {
                self.attached_session = None;
            }
            
            true
        } else {
            false
        }
    }

    /// Rename a session.
    pub fn rename_session(
        &mut self,
        id: SessionId,
        new_name: impl Into<String>,
    ) -> Result<(), String> {
        let new_name = new_name.into();
        
        // Check for duplicates.
        if self.session_names.contains_key(&new_name) {
            return Err(format!("Session '{}' already exists", new_name));
        }

        let session = self.sessions.get_mut(&id)
            .ok_or_else(|| "Session not found".to_string())?;

        // Update name mapping.
        self.session_names.remove(&session.name);
        session.name = new_name.clone();
        self.session_names.insert(new_name, id);

        Ok(())
    }

    /// Create a new window in a session.
    pub fn create_window(
        &mut self,
        session_id: SessionId,
        name: impl Into<String>,
    ) -> Option<SessionWindowId> {
        let session = self.sessions.get_mut(&session_id)?;

        let id = SessionWindowId(self.next_window_id);
        self.next_window_id += 1;

        let index = session.windows.len();
        let window = Window::new(id, name, session_id);
        
        self.windows.insert(id, window);
        session.add_window(id);

        Some(id)
    }

    /// Get a window by ID.
    pub fn get_window(&self, id: SessionWindowId) -> Option<&Window> {
        self.windows.get(&id)
    }

    /// Get a window by ID (mutable).
    pub fn get_window_mut(&mut self, id: SessionWindowId) -> Option<&mut Window> {
        self.windows.get_mut(&id)
    }

    /// Kill a window.
    pub fn kill_window(&mut self, id: SessionWindowId) -> bool {
        let window = match self.windows.remove(&id) {
            Some(w) => w,
            None => return false,
        };

        let session_id = window.session_id;
        let should_kill_session = if let Some(session) = self.sessions.get_mut(&session_id) {
            session.remove_window(id);
            session.windows.is_empty() && session.kill_on_detach
        } else {
            false
        };

        // If session has no windows and should be killed, kill it.
        if should_kill_session {
            self.kill_session(session_id);
        }

        true
    }

    /// Rename a window.
    pub fn rename_window(&mut self, id: SessionWindowId, name: impl Into<String>) -> bool {
        if let Some(window) = self.windows.get_mut(&id) {
            window.name = name.into();
            true
        } else {
            false
        }
    }

    /// Attach to a session.
    pub fn attach_session(&mut self, id: SessionId) -> Result<(), String> {
        if !self.sessions.contains_key(&id) {
            return Err("Session not found".to_string());
        }

        // Detach from current session.
        if let Some(current) = self.attached_session {
            if let Some(session) = self.sessions.get_mut(&current) {
                session.detach();
            }
        }

        // Attach to new session.
        if let Some(session) = self.sessions.get_mut(&id) {
            session.attach();
        }
        self.attached_session = Some(id);

        Ok(())
    }

    /// Detach from the current session.
    pub fn detach_session(&mut self) -> Option<SessionId> {
        let session_id = self.attached_session?;

        if let Some(session) = self.sessions.get_mut(&session_id) {
            session.detach();
            
            // Check if session should be killed.
            if session.kill_on_detach {
                self.kill_session(session_id);
            }
        }

        self.attached_session = None;
        Some(session_id)
    }

    /// Get the attached session ID.
    pub fn attached_session(&self) -> Option<SessionId> {
        self.attached_session
    }

    /// Get the attached session.
    pub fn attached(&self) -> Option<&Session> {
        self.attached_session.and_then(|id| self.sessions.get(&id))
    }

    /// List all sessions.
    pub fn list_sessions(&self) -> Vec<&Session> {
        self.sessions.values().collect()
    }

    /// Check if a session exists.
    pub fn session_exists(&self, name: &str) -> bool {
        self.session_names.contains_key(name)
    }

    /// Get session count.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get window count across all sessions.
    pub fn total_window_count(&self) -> usize {
        self.windows.len()
    }

    /// Select the next window in the attached session.
    pub fn next_window(&mut self) -> Option<SessionWindowId> {
        let session = self.attached()?;
        let current_index = session.active_window_index;
        
        if session.windows.len() <= 1 {
            return session.active_window();
        }

        let new_index = (current_index + 1) % session.windows.len();
        let new_window = session.windows[new_index];

        if let Some(session) = self.attached_session
            .and_then(|id| self.sessions.get_mut(&id)) {
            session.active_window_index = new_index;
        }

        Some(new_window)
    }

    /// Select the previous window in the attached session.
    pub fn previous_window(&mut self) -> Option<SessionWindowId> {
        let session = self.attached()?;
        let current_index = session.active_window_index;
        
        if session.windows.len() <= 1 {
            return session.active_window();
        }

        let new_index = if current_index == 0 {
            session.windows.len() - 1
        } else {
            current_index - 1
        };
        let new_window = session.windows[new_index];

        if let Some(session) = self.attached_session
            .and_then(|id| self.sessions.get_mut(&id)) {
            session.active_window_index = new_index;
        }

        Some(new_window)
    }

    /// Select a window by index.
    pub fn select_window(&mut self, index: usize) -> Option<SessionWindowId> {
        let session = self.attached()?;
        
        if index >= session.windows.len() {
            return None;
        }

        let window_id = session.windows[index];

        if let Some(session) = self.attached_session
            .and_then(|id| self.sessions.get_mut(&id)) {
            session.active_window_index = index;
        }

        Some(window_id)
    }

    /// Save sessions to disk.
    pub fn save(&self) -> Result<(), String> {
        // TODO: Implement session persistence.
        Ok(())
    }

    /// Load sessions from disk.
    pub fn load(&mut self) -> Result<(), String> {
        // TODO: Implement session restoration.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let mut manager = SessionManager::new();
        let id = manager.create_session("test").unwrap();
        
        assert_eq!(manager.session_count(), 1);
        assert!(manager.session_exists("test"));
        
        let session = manager.get_session(id).unwrap();
        assert_eq!(session.name, "test");
    }

    #[test]
    fn test_duplicate_session_name() {
        let mut manager = SessionManager::new();
        manager.create_session("test").unwrap();
        
        let result = manager.create_session("test");
        assert!(result.is_err());
    }

    #[test]
    fn test_window_creation() {
        let mut manager = SessionManager::new();
        let session_id = manager.create_session("test").unwrap();
        
        let window_id = manager.create_window(session_id, "window1").unwrap();
        
        let session = manager.get_session(session_id).unwrap();
        assert_eq!(session.window_count(), 1);
        
        let window = manager.get_window(window_id).unwrap();
        assert_eq!(window.name, "window1");
    }

    #[test]
    fn test_session_attach() {
        let mut manager = SessionManager::new();
        let id = manager.create_session("test").unwrap();
        
        manager.attach_session(id).unwrap();
        assert_eq!(manager.attached_session(), Some(id));
        
        let session = manager.get_session(id).unwrap();
        assert!(session.is_attached);
    }
}
