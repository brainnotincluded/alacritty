//! Pane management for cmux terminal multiplexer.
//!
//! Panes are subdivisions of a window that each contain a terminal.
//! They can be split horizontally or vertically, and navigated between.

use std::collections::HashMap;
use std::sync::Arc;

use cmux_terminal::sync::FairMutex;
use cmux_terminal::term::Term;
use cmux_terminal::event_loop::Notifier;
use cmux_terminal::grid::Dimensions;
use serde::{Deserialize, Serialize};
use winit::window::WindowId;

use crate::event::EventProxy;

/// Unique identifier for a pane within a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct PaneId(pub u64);

impl From<PaneId> for u64 {
    fn from(id: PaneId) -> u64 {
        id.0
    }
}

impl From<u64> for PaneId {
    fn from(id: u64) -> PaneId {
        PaneId(id)
    }
}

/// Direction for pane operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Split direction for creating new panes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    /// Split horizontally (creates top/bottom panes).
    Horizontal,
    /// Split vertically (creates left/right panes).
    Vertical,
}

/// Layout information for a pane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaneLayout {
    /// X position (0.0 - 1.0, percentage of parent).
    pub x: f32,
    /// Y position (0.0 - 1.0, percentage of parent).
    pub y: f32,
    /// Width (0.0 - 1.0, percentage of parent).
    pub width: f32,
    /// Height (0.0 - 1.0, percentage of parent).
    pub height: f32,
}

impl Default for PaneLayout {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }
    }
}

/// A pane containing a terminal.
pub struct Pane {
    /// Unique identifier.
    pub id: PaneId,
    /// Terminal state.
    pub terminal: Arc<FairMutex<Term<EventProxy>>>,
    /// PTY notifier for sending input.
    pub notifier: Notifier,
    /// Layout information.
    pub layout: PaneLayout,
    /// Whether this pane is active/focused.
    pub is_active: bool,
    /// Window ID this pane belongs to.
    pub window_id: WindowId,
    /// Pane title (usually the running process).
    pub title: String,
    /// Whether the pane is zoomed (fills the window).
    pub is_zoomed: bool,
    /// Parent pane ID (for tree structure).
    pub parent: Option<PaneId>,
    /// Left/Top child.
    pub first_child: Option<PaneId>,
    /// Right/Bottom child.
    pub second_child: Option<PaneId>,
    /// Split direction if this pane has been split.
    pub split_direction: Option<SplitDirection>,
}

impl Pane {
    /// Create a new pane.
    pub fn new(
        id: PaneId,
        terminal: Arc<FairMutex<Term<EventProxy>>>,
        notifier: Notifier,
        window_id: WindowId,
    ) -> Self {
        Self {
            id,
            terminal,
            notifier,
            layout: PaneLayout::default(),
            is_active: false,
            window_id,
            title: String::new(),
            is_zoomed: false,
            parent: None,
            first_child: None,
            second_child: None,
            split_direction: None,
        }
    }

    /// Get the terminal size in columns and rows.
    pub fn size(&self) -> (usize, usize) {
        let terminal = self.terminal.lock();
        (terminal.columns(), terminal.screen_lines())
    }

    /// Send text input to the pane's PTY.
    pub fn send_text(&self, text: &str) {
        let _ = self.notifier.0.send(cmux_terminal::event_loop::Msg::Input(text.as_bytes().to_vec().into()));
    }

    /// Check if this pane is a leaf (has no children).
    pub fn is_leaf(&self) -> bool {
        self.first_child.is_none() && self.second_child.is_none()
    }

    /// Check if this pane has been split.
    pub fn is_split(&self) -> bool {
        self.split_direction.is_some()
    }
}

/// Manages all panes in a window.
pub struct PaneManager {
    /// All panes by ID.
    panes: HashMap<PaneId, Pane>,
    /// The currently active pane ID.
    active_pane: Option<PaneId>,
    /// Next pane ID to assign.
    next_id: u64,
    /// Root pane ID.
    root_pane: Option<PaneId>,
    /// Border width in pixels.
    pub border_width: u32,
    /// Active border color.
    pub active_border_color: [f32; 4],
    /// Inactive border color.
    pub inactive_border_color: [f32; 4],
}

impl Default for PaneManager {
    fn default() -> Self {
        Self {
            panes: HashMap::new(),
            active_pane: None,
            next_id: 1,
            root_pane: None,
            border_width: 2,
            active_border_color: [1.0, 0.67, 0.0, 1.0], // Orange
            inactive_border_color: [0.27, 0.27, 0.27, 1.0], // Dark gray
        }
    }
}

impl PaneManager {
    /// Create a new pane manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create the root pane.
    pub fn create_root(
        &mut self,
        terminal: Arc<FairMutex<Term<EventProxy>>>,
        notifier: Notifier,
        window_id: WindowId,
    ) -> PaneId {
        let id = PaneId(self.next_id);
        self.next_id += 1;

        let mut pane = Pane::new(id, terminal, notifier, window_id);
        pane.is_active = true;
        pane.layout = PaneLayout::default();

        self.panes.insert(id, pane);
        self.active_pane = Some(id);
        self.root_pane = Some(id);

        id
    }

    /// Split a pane.
    pub fn split_pane(
        &mut self,
        pane_id: PaneId,
        direction: SplitDirection,
        new_terminal: Arc<FairMutex<Term<EventProxy>>>,
        new_notifier: Notifier,
    ) -> Option<PaneId> {
        let pane = self.panes.get(&pane_id)?;
        
        // Can't split if already has children.
        if !pane.is_leaf() {
            return None;
        }

        let window_id = pane.window_id;
        let layout = pane.layout;

        // Create new pane IDs.
        let first_id = PaneId(self.next_id);
        self.next_id += 1;
        let second_id = PaneId(self.next_id);
        self.next_id += 1;

        // Calculate layouts.
        let (first_layout, second_layout) = match direction {
            SplitDirection::Horizontal => {
                let half_height = layout.height / 2.0;
                (
                    PaneLayout {
                        x: layout.x,
                        y: layout.y,
                        width: layout.width,
                        height: half_height,
                    },
                    PaneLayout {
                        x: layout.x,
                        y: layout.y + half_height,
                        width: layout.width,
                        height: half_height,
                    },
                )
            }
            SplitDirection::Vertical => {
                let half_width = layout.width / 2.0;
                (
                    PaneLayout {
                        x: layout.x,
                        y: layout.y,
                        width: half_width,
                        height: layout.height,
                    },
                    PaneLayout {
                        x: layout.x + half_width,
                        y: layout.y,
                        width: half_width,
                        height: layout.height,
                    },
                )
            }
        };

        // Create the new panes.
        let first_terminal = Arc::clone(&self.panes[&pane_id].terminal);
        let first_notifier = Notifier(self.panes[&pane_id].notifier.0.clone());
        
        let mut first_pane = Pane::new(first_id, first_terminal, first_notifier, window_id);
        first_pane.layout = first_layout;
        first_pane.parent = Some(pane_id);
        first_pane.is_active = false;

        let mut second_pane = Pane::new(second_id, new_terminal, new_notifier, window_id);
        second_pane.layout = second_layout;
        second_pane.parent = Some(pane_id);
        second_pane.is_active = true;

        // Update the parent pane to be a container.
        let parent = self.panes.get_mut(&pane_id)?;
        parent.split_direction = Some(direction);
        parent.first_child = Some(first_id);
        parent.second_child = Some(second_id);
        parent.is_active = false;

        // Insert new panes.
        self.panes.insert(first_id, first_pane);
        self.panes.insert(second_id, second_pane);

        // Update active pane.
        self.active_pane = Some(second_id);

        Some(second_id)
    }

    /// Get a reference to a pane.
    pub fn get(&self, id: PaneId) -> Option<&Pane> {
        self.panes.get(&id)
    }

    /// Get a mutable reference to a pane.
    pub fn get_mut(&mut self, id: PaneId) -> Option<&mut Pane> {
        self.panes.get_mut(&id)
    }

    /// Get the active pane ID.
    pub fn active_pane(&self) -> Option<PaneId> {
        self.active_pane
    }

    /// Get a reference to the active pane.
    pub fn active(&self) -> Option<&Pane> {
        self.active_pane.and_then(|id| self.panes.get(&id))
    }

    /// Get a mutable reference to the active pane.
    pub fn active_mut(&mut self) -> Option<&mut Pane> {
        self.active_pane.and_then(|id| self.panes.get_mut(&id))
    }

    /// Set the active pane.
    pub fn set_active(&mut self, id: PaneId) -> bool {
        if !self.panes.contains_key(&id) {
            return false;
        }

        // Deactivate current active pane.
        if let Some(active) = self.active_pane {
            if let Some(pane) = self.panes.get_mut(&active) {
                pane.is_active = false;
            }
        }

        // Activate new pane.
        if let Some(pane) = self.panes.get_mut(&id) {
            pane.is_active = true;
        }

        self.active_pane = Some(id);
        true
    }

    /// Navigate to a pane in the specified direction.
    pub fn navigate(&mut self, direction: Direction) -> Option<PaneId> {
        let current_id = self.active_pane?;
        let current = self.panes.get(&current_id)?;

        // Get current layout center.
        let current_center_x = current.layout.x + current.layout.width / 2.0;
        let current_center_y = current.layout.y + current.layout.height / 2.0;

        // Find the nearest pane in the specified direction.
        let mut best_candidate: Option<(PaneId, f32)> = None;

        for (id, pane) in &self.panes {
            if *id == current_id || !pane.is_leaf() {
                continue;
            }

            let center_x = pane.layout.x + pane.layout.width / 2.0;
            let center_y = pane.layout.y + pane.layout.height / 2.0;

            let is_in_direction = match direction {
                Direction::Up => center_y < current_center_y,
                Direction::Down => center_y > current_center_y,
                Direction::Left => center_x < current_center_x,
                Direction::Right => center_x > current_center_x,
            };

            if is_in_direction {
                // Calculate Euclidean distance.
                let dx = center_x - current_center_x;
                let dy = center_y - current_center_y;
                let distance = (dx * dx + dy * dy).sqrt();

                if best_candidate.map_or(true, |(_, best_dist)| distance < best_dist) {
                    best_candidate = Some((*id, distance));
                }
            }
        }

        if let Some((id, _)) = best_candidate {
            self.set_active(id);
            Some(id)
        } else {
            None
        }
    }

    /// Close a pane.
    pub fn close_pane(&mut self, id: PaneId) -> bool {
        // First, collect all the info we need.
        let pane = match self.panes.get(&id) {
            Some(p) => p,
            None => return false,
        };

        // Can't close if it has children.
        if !pane.is_leaf() {
            return false;
        }

        let parent_id = pane.parent;
        
        // Collect sibling and grandparent info if there's a parent.
        let sibling_info = parent_id.and_then(|pid| {
            let parent = self.panes.get(&pid)?;
            let sibling = if parent.first_child == Some(id) {
                parent.second_child
            } else {
                parent.first_child
            };
            sibling.map(|sid| {
                let sibling_layout = self.panes[&sid].layout;
                let parent_layout = self.panes[&pid].layout;
                let grandparent_id = self.panes[&pid].parent;
                (sid, sibling_layout, parent_layout, grandparent_id, pid)
            })
        });

        // Remove the pane.
        self.panes.remove(&id);

        // If this was the active pane, try to find another one.
        if self.active_pane == Some(id) {
            self.active_pane = self.panes.keys().copied().find(|k| self.panes[k].is_leaf());
            if let Some(new_active) = self.active_pane {
                if let Some(pane) = self.panes.get_mut(&new_active) {
                    pane.is_active = true;
                }
            }
        }

        // Handle collapsing the split.
        if let Some((sibling_id, _sibling_layout, parent_layout, grandparent_id, parent_id)) = sibling_info {
            // Update sibling's parent and layout.
            if let Some(sibling) = self.panes.get_mut(&sibling_id) {
                sibling.parent = grandparent_id;
                sibling.layout = parent_layout;
            }

            // Update grandparent's child reference.
            if let Some(gp_id) = grandparent_id {
                if let Some(gp) = self.panes.get_mut(&gp_id) {
                    if gp.first_child == Some(parent_id) {
                        gp.first_child = Some(sibling_id);
                    } else {
                        gp.second_child = Some(sibling_id);
                    }
                }
            } else {
                // This was the root, update root.
                self.root_pane = Some(sibling_id);
            }

            // Remove the parent container.
            self.panes.remove(&parent_id);

            // Recalculate layouts for the promoted subtree.
            self.recalculate_layouts(sibling_id);
        } else if parent_id.is_none() {
            // This was the root pane, clear root.
            self.root_pane = None;
        }

        true
    }

    /// Recalculate layouts for a subtree.
    fn recalculate_layouts(&mut self, root_id: PaneId) {
        // Collect all layout updates we need to make first.
        let mut updates: Vec<(PaneId, PaneLayout)> = Vec::new();
        self.collect_layout_updates(root_id, &mut updates);
        
        // Apply all updates.
        for (id, layout) in updates {
            if let Some(pane) = self.panes.get_mut(&id) {
                pane.layout = layout;
            }
        }
    }
    
    /// Collect layout updates without borrowing.
    fn collect_layout_updates(&self, root_id: PaneId, updates: &mut Vec<(PaneId, PaneLayout)>) {
        let pane = match self.panes.get(&root_id) {
            Some(p) => p,
            None => return,
        };

        if pane.is_split() {
            let new_layout = pane.layout;
            let (first_layout, second_layout) = match pane.split_direction.unwrap() {
                SplitDirection::Horizontal => {
                    let half_height = new_layout.height / 2.0;
                    (
                        PaneLayout {
                            x: new_layout.x,
                            y: new_layout.y,
                            width: new_layout.width,
                            height: half_height,
                        },
                        PaneLayout {
                            x: new_layout.x,
                            y: new_layout.y + half_height,
                            width: new_layout.width,
                            height: half_height,
                        },
                    )
                }
                SplitDirection::Vertical => {
                    let half_width = new_layout.width / 2.0;
                    (
                        PaneLayout {
                            x: new_layout.x,
                            y: new_layout.y,
                            width: half_width,
                            height: new_layout.height,
                        },
                        PaneLayout {
                            x: new_layout.x + half_width,
                            y: new_layout.y,
                            width: half_width,
                            height: new_layout.height,
                        },
                    )
                }
            };

            if let Some(first_id) = pane.first_child {
                updates.push((first_id, first_layout));
                self.collect_layout_updates(first_id, updates);
            }
            if let Some(second_id) = pane.second_child {
                updates.push((second_id, second_layout));
                self.collect_layout_updates(second_id, updates);
            }
        }
    }

    /// Get all leaf panes (actual terminal panes).
    pub fn leaf_panes(&self) -> impl Iterator<Item = &Pane> {
        self.panes.values().filter(|p| p.is_leaf())
    }

    /// Get mutable references to all leaf panes.
    pub fn leaf_panes_mut(&mut self) -> impl Iterator<Item = &mut Pane> {
        self.panes.values_mut().filter(|p| p.is_leaf())
    }

    /// Get the number of panes.
    pub fn len(&self) -> usize {
        self.panes.values().filter(|p| p.is_leaf()).count()
    }

    /// Check if there are no panes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Resize a pane.
    pub fn resize_pane(&mut self, _id: PaneId, _direction: Direction, _amount: f32) {
        // TODO: Implement pane resizing.
    }

    /// Get all panes with their layouts in render order.
    pub fn visible_panes(&self) -> Vec<&Pane> {
        self.panes.values().filter(|p| p.is_leaf()).collect()
    }

    /// Get the root pane ID.
    pub fn root_pane(&self) -> Option<PaneId> {
        self.root_pane
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pane_id() {
        let id1 = PaneId(1);
        let id2 = PaneId(2);
        
        assert_ne!(id1, id2);
        assert_eq!(id1.0, 1);
    }

    #[test]
    fn test_layout_default() {
        let layout = PaneLayout::default();
        assert_eq!(layout.x, 0.0);
        assert_eq!(layout.y, 0.0);
        assert_eq!(layout.width, 1.0);
        assert_eq!(layout.height, 1.0);
    }
}
