//! UI components for cmux multiplexer.
//!
//! This module provides visual elements for the terminal multiplexer:
//! - Pane borders with active/inactive states
//! - Status bar showing session info and keybindings
//! - Tab bar for window navigation
//! - Visual mode indicators

use crate::display::color::Rgb;
use crate::display::SizeInfo;
use crate::renderer::rects::RenderRect;

/// Colors for the UI theme.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    /// Active pane border color.
    pub pane_border_active: Rgb,
    /// Inactive pane border color.
    pub pane_border_inactive: Rgb,
    /// Status bar background.
    pub status_bar_bg: Rgb,
    /// Status bar foreground.
    pub status_bar_fg: Rgb,
    /// Tab bar background.
    pub tab_bar_bg: Rgb,
    /// Active tab background.
    pub tab_active_bg: Rgb,
    /// Active tab foreground.
    pub tab_active_fg: Rgb,
    /// Inactive tab foreground.
    pub tab_inactive_fg: Rgb,
    /// Accent color for highlights.
    pub accent: Rgb,
}

impl Default for Theme {
    fn default() -> Self {
        // Default to Warp's iconic dark theme
        Self::warp_dark()
    }
}

impl Theme {
    /// Light theme variant.
    pub fn light() -> Self {
        Self {
            pane_border_active: Rgb::new(9, 105, 218),     // GitHub blue
            pane_border_inactive: Rgb::new(208, 215, 222), // Light gray
            status_bar_bg: Rgb::new(246, 248, 250),        // Light background
            status_bar_fg: Rgb::new(36, 41, 47),           // Dark text
            tab_bar_bg: Rgb::new(255, 255, 255),           // White
            tab_active_bg: Rgb::new(255, 255, 255),        // White
            tab_active_fg: Rgb::new(36, 41, 47),           // Dark text
            tab_inactive_fg: Rgb::new(88, 96, 105),        // Gray text
            accent: Rgb::new(9, 105, 218),                 // Blue accent
        }
    }

    /// Warp terminal dark theme - the iconic purple-accented dark theme.
    pub fn warp_dark() -> Self {
        // Based on Warp terminal's default dark theme
        // Purple accent (#A172FF), dark blue-gray background (#1A1B26)
        Self {
            pane_border_active: Rgb::new(161, 114, 255),   // Warp purple
            pane_border_inactive: Rgb::new(59, 62, 75),    // Dark gray-blue
            status_bar_bg: Rgb::new(26, 27, 38),           // Warp background
            status_bar_fg: Rgb::new(192, 202, 245),        // Light lavender
            tab_bar_bg: Rgb::new(26, 27, 38),              // Warp background
            tab_active_bg: Rgb::new(41, 43, 56),           // Lighter blue-gray
            tab_active_fg: Rgb::new(255, 255, 255),        // White
            tab_inactive_fg: Rgb::new(130, 136, 165),      // Muted lavender-gray
            accent: Rgb::new(161, 114, 255),               // Warp purple
        }
    }

    /// Warp terminal light theme.
    pub fn warp_light() -> Self {
        Self {
            pane_border_active: Rgb::new(137, 87, 229),    // Warp purple (darker for light)
            pane_border_inactive: Rgb::new(200, 202, 210), // Light gray
            status_bar_bg: Rgb::new(250, 250, 252),        // Near white
            status_bar_fg: Rgb::new(50, 52, 64),           // Dark gray
            tab_bar_bg: Rgb::new(250, 250, 252),           // Near white
            tab_active_bg: Rgb::new(255, 255, 255),        // White
            tab_active_fg: Rgb::new(50, 52, 64),           // Dark gray
            tab_inactive_fg: Rgb::new(130, 136, 165),      // Muted gray
            accent: Rgb::new(137, 87, 229),                // Warp purple
        }
    }
}

/// Warp terminal ANSI color palette (16 colors).
/// These are the colors used by Warp's default theme.
#[derive(Debug, Clone)]
pub struct WarpPalette;

impl WarpPalette {
    /// Warp's dark theme terminal colors
    pub fn dark() -> [Rgb; 16] {
        [
            // Normal colors (0-7)
            Rgb::new(21, 22, 33),      // Black: #151621
            Rgb::new(247, 118, 142),   // Red: #F7768E
            Rgb::new(158, 206, 106),   // Green: #9ECE6A
            Rgb::new(224, 175, 104),   // Yellow: #E0AF68
            Rgb::new(122, 162, 247),   // Blue: #7AA2F7
            Rgb::new(187, 154, 247),   // Magenta: #BB9AF7
            Rgb::new(125, 207, 255),   // Cyan: #7DCFFF
            Rgb::new(169, 177, 214),   // White: #A9B1D6
            // Bright colors (8-15)
            Rgb::new(65, 72, 104),     // Bright Black: #414868
            Rgb::new(247, 118, 142),   // Bright Red: #F7768E
            Rgb::new(158, 206, 106),   // Bright Green: #9ECE6A
            Rgb::new(224, 175, 104),   // Bright Yellow: #E0AF68
            Rgb::new(122, 162, 247),   // Bright Blue: #7AA2F7
            Rgb::new(187, 154, 247),   // Bright Magenta: #BB9AF7
            Rgb::new(125, 207, 255),   // Bright Cyan: #7DCFFF
            Rgb::new(192, 202, 245),   // Bright White: #C0CAF5
        ]
    }

    /// Warp's purple accent color
    pub fn accent() -> Rgb {
        Rgb::new(161, 114, 255) // #A172FF
    }

    /// Warp's dark background color
    pub fn background() -> Rgb {
        Rgb::new(26, 27, 38) // #1A1B26
    }

    /// Warp's foreground/text color
    pub fn foreground() -> Rgb {
        Rgb::new(192, 202, 245) // #C0CAF5
    }
}

/// Layout information for a pane.
#[derive(Debug, Clone)]
pub struct PaneInfo {
    /// X position in pixels.
    pub x: f32,
    /// Y position in pixels.
    pub y: f32,
    /// Width in pixels.
    pub width: f32,
    /// Height in pixels.
    pub height: f32,
    /// Whether this pane is active.
    pub is_active: bool,
    /// Pane title.
    pub title: String,
}

/// Renders pane borders.
pub struct PaneBorderRenderer {
    /// Border width in pixels.
    pub border_width: f32,
    /// Whether to draw title bars.
    pub show_titles: bool,
    /// Title bar height.
    pub title_height: f32,
}

impl Default for PaneBorderRenderer {
    fn default() -> Self {
        Self {
            border_width: 2.0,
            show_titles: true,
            title_height: 20.0,
        }
    }
}

impl PaneBorderRenderer {
    /// Render borders for all panes.
    pub fn render(&self, panes: &[PaneInfo], theme: &Theme, _size_info: &SizeInfo) -> Vec<RenderRect> {
        let mut rects = Vec::new();

        for pane in panes {
            let color = if pane.is_active {
                theme.pane_border_active
            } else {
                theme.pane_border_inactive
            };

            let alpha = if pane.is_active { 1.0 } else { 0.6 };

            // Draw border rects (top, bottom, left, right)
            let bw = self.border_width;

            // Top border
            rects.push(RenderRect::new(
                pane.x,
                pane.y,
                pane.width,
                bw,
                color,
                alpha,
            ));

            // Bottom border
            rects.push(RenderRect::new(
                pane.x,
                pane.y + pane.height - bw,
                pane.width,
                bw,
                color,
                alpha,
            ));

            // Left border
            rects.push(RenderRect::new(
                pane.x,
                pane.y,
                bw,
                pane.height,
                color,
                alpha,
            ));

            // Right border
            rects.push(RenderRect::new(
                pane.x + pane.width - bw,
                pane.y,
                bw,
                pane.height,
                color,
                alpha,
            ));

            // Draw title bar if enabled
            if self.show_titles {
                let title_bg = if pane.is_active {
                    theme.pane_border_active
                } else {
                    theme.pane_border_inactive
                };

                // Title bar background
                rects.push(RenderRect::new(
                    pane.x + bw,
                    pane.y + bw,
                    pane.width - 2.0 * bw,
                    self.title_height,
                    title_bg,
                    alpha * 0.3, // More transparent
                ));
            }
        }

        rects
    }
}

/// Status bar component.
pub struct StatusBar {
    /// Height in pixels.
    pub height: f32,
    /// Current mode text.
    pub mode_text: String,
    /// Session name.
    pub session_name: String,
    /// Window index.
    pub window_index: usize,
    /// Keybinding hints.
    pub hints: Vec<(String, String)>, // (key, action)
}

impl Default for StatusBar {
    fn default() -> Self {
        Self {
            height: 24.0,
            mode_text: String::new(),
            session_name: "default".to_string(),
            window_index: 0,
            hints: vec![
                ("Cmd+D".to_string(), "Split".to_string()),
                ("Cmd+W".to_string(), "Close".to_string()),
                ("Cmd+[".to_string(), "Prev".to_string()),
                ("Cmd+]".to_string(), "Next".to_string()),
            ],
        }
    }
}

impl StatusBar {
    /// Render the status bar background.
    pub fn render_background(&self, theme: &Theme, size_info: &SizeInfo) -> RenderRect {
        let y = size_info.height() - self.height;
        RenderRect::new(
            0.0,
            y,
            size_info.width(),
            self.height,
            theme.status_bar_bg,
            1.0,
        )
    }

    /// Update hints based on current mode.
    pub fn update_hints(&mut self, prefix_mode: bool) {
        if prefix_mode {
            self.mode_text = "PREFIX".to_string();
            self.hints = vec![
                ("d".to_string(), "Split V".to_string()),
                ("D".to_string(), "Split H".to_string()),
                ("x".to_string(), "Close".to_string()),
                ("n".to_string(), "Next".to_string()),
                ("p".to_string(), "Prev".to_string()),
                ("Esc".to_string(), "Cancel".to_string()),
            ];
        } else {
            self.mode_text.clear();
            self.hints = vec![
                ("Cmd+D".to_string(), "Split".to_string()),
                ("Cmd+W".to_string(), "Close".to_string()),
                ("Cmd+[".to_string(), "Prev".to_string()),
                ("Cmd+]".to_string(), "Next".to_string()),
                ("Cmd+B".to_string(), "Prefix".to_string()),
            ];
        }
    }
}

/// Tab bar component.
pub struct TabBar {
    /// Height in pixels.
    pub height: f32,
    /// Tab titles.
    pub tabs: Vec<TabInfo>,
    /// Active tab index.
    pub active_index: usize,
}

/// Information about a tab.
#[derive(Debug, Clone)]
pub struct TabInfo {
    /// Tab title.
    pub title: String,
    /// Whether the tab has unsaved changes.
    pub is_modified: bool,
}

impl Default for TabBar {
    fn default() -> Self {
        Self {
            height: 28.0,
            tabs: vec![TabInfo {
                title: "cmux".to_string(),
                is_modified: false,
            }],
            active_index: 0,
        }
    }
}

impl TabBar {
    /// Render tab bar background.
    pub fn render_background(&self, theme: &Theme, size_info: &SizeInfo) -> RenderRect {
        RenderRect::new(
            0.0,
            0.0,
            size_info.width(),
            self.height,
            theme.tab_bar_bg,
            1.0,
        )
    }

    /// Render individual tab backgrounds.
    pub fn render_tabs(&self, theme: &Theme, _size_info: &SizeInfo) -> Vec<RenderRect> {
        let mut rects = Vec::new();
        let tab_width = 150.0; // Fixed width for now
        let padding = 4.0;

        for (i, _tab) in self.tabs.iter().enumerate() {
            let x = i as f32 * (tab_width + padding);
            let is_active = i == self.active_index;

            let bg = if is_active {
                theme.tab_active_bg
            } else {
                theme.tab_bar_bg
            };

            rects.push(RenderRect::new(
                x,
                padding,
                tab_width,
                self.height - 2.0 * padding,
                bg,
                1.0,
            ));

            // Active indicator line at bottom
            if is_active {
                rects.push(RenderRect::new(
                    x,
                    self.height - 2.0,
                    tab_width,
                    2.0,
                    theme.accent,
                    1.0,
                ));
            }
        }

        rects
    }
}

/// Complete UI renderer for cmux.
pub struct UiRenderer {
    /// Current theme.
    pub theme: Theme,
    /// Pane border renderer.
    pub pane_borders: PaneBorderRenderer,
    /// Status bar.
    pub status_bar: StatusBar,
    /// Tab bar.
    pub tab_bar: TabBar,
    /// Whether UI is enabled.
    pub enabled: bool,
}

impl Default for UiRenderer {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            pane_borders: PaneBorderRenderer::default(),
            status_bar: StatusBar::default(),
            tab_bar: TabBar::default(),
            enabled: true,
        }
    }
}

impl UiRenderer {
    /// Render all UI components.
    pub fn render(&self, panes: &[PaneInfo], size_info: &SizeInfo) -> Vec<RenderRect> {
        if !self.enabled {
            return Vec::new();
        }

        let mut rects = Vec::new();

        // Tab bar
        rects.push(self.tab_bar.render_background(&self.theme, size_info));
        rects.extend(self.tab_bar.render_tabs(&self.theme, size_info));

        // Pane borders
        rects.extend(self.pane_borders.render(panes, &self.theme, size_info));

        // Status bar
        rects.push(self.status_bar.render_background(&self.theme, size_info));

        rects
    }

    /// Update status bar mode.
    pub fn set_prefix_mode(&mut self, active: bool) {
        self.status_bar.update_hints(active);
    }

    /// Set active pane.
    pub fn set_active_pane(&mut self, index: usize, panes: &mut [PaneInfo]) {
        for (i, pane) in panes.iter_mut().enumerate() {
            pane.is_active = i == index;
        }
    }
}
