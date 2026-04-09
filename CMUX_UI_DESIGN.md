# cmux Beautiful UI Design Document

## Research Summary

### Inspiration from Top Terminal Emulators

#### 1. **Zellij** (Modern Rust Multiplexer)
- **Discoverable UI**: Status bar showing available keybindings for current mode
- **Pane Borders**: Clean borders around each pane with visual indicators
- **Floating Panes**: Overlay panes that hover above tiled layout
- **Smart Layouts**: Auto-placement of new panes based on available space
- **WASM Plugins**: UI components as plugins (tab bar, status bar)

#### 2. **iTerm2** (macOS Gold Standard)
- **Pane Title Bars**: Show session info with drag handles and close buttons
- **Dim Inactive Panes**: Visual distinction for non-focused panes
- **Smooth Animations**: Pane dimming and transitions
- **Split Indicators**: Visual feedback during splits

#### 3. **Kitty** (GPU-Accelerated)
- **Borderless Design**: Minimal chrome, content-focused
- **Tab Bar**: Clean tabs at top with close buttons
- **OS Integration**: Native window tabs on macOS

#### 4. **Ghostty** (Newest GPU Terminal)
- **Native UI Components**: Uses native Cocoa/AppKit on macOS
- **Theme Support**: Hundreds of built-in themes
- **Minimal Chrome**: Focus on content with subtle UI

#### 5. **WezTerm** (Most Customizable)
- **Lua Configuration**: Deep UI customization
- **Multi-Backend Rendering**: OpenGL, Vulkan, Metal, DX12
- **Tab/Pane UI**: Built-in with extensive theming

---

## cmux UI Design Proposal

### Phase 1: Foundation (Immediate)

#### 1.1 Pane Borders System
```rust
// Render borders around each terminal pane
struct PaneBorder {
    // Visual styling
    active_color: Rgb,
    inactive_color: Rgb,
    width: f32,
    
    // Elements
    show_title: bool,
    show_close_button: bool,
    show_resize_handles: bool,
}
```

**Design Spec:**
- **Active Pane**: 2px border in accent color (blue/cyan)
- **Inactive Panes**: 1px border in muted gray
- **Title Bar**: Optional pane title at top
- **Rounded Corners**: 4px radius for modern look

#### 1.2 Status Bar (Bottom)
```rust
struct StatusBar {
    // Left side: Session/Window info
    session_name: String,
    window_index: usize,
    
    // Center: Current mode indicators
    prefix_mode_active: bool,
    
    // Right side: Helpful hints
    shortcuts: Vec<(String, String)>, // (key, action)
}
```

**Design Spec:**
- **Height**: 24px
- **Background**: Darker than terminal background
- **Left**: Session name, window number
- **Right**: Current keybinding hints (changes with mode)
- **Colors**: Muted for inactive, bright for active

#### 1.3 Tab Bar (Top)
```rust
struct TabBar {
    tabs: Vec<Tab>,
    active_index: usize,
}

struct Tab {
    title: String,
    is_active: bool,
    is_modified: bool, // Unsaved changes indicator
}
```

**Design Spec:**
- **Height**: 28px
- **Active Tab**: Full background, bold text
- **Inactive Tabs**: Transparent, normal text
- **Close Button**: Appears on hover
- **Add Button**: "+" at end of tab list

---

### Phase 2: Visual Polish

#### 2.1 Dim Inactive Panes
- Reduce opacity of non-focused panes to 85%
- Slightly desaturate colors
- Keep active pane at 100% brightness

#### 2.2 Smooth Transitions
- Pane focus changes: 150ms fade
- Window switching: 200ms slide
- Pane creation: 200ms scale-in

#### 2.3 Modern Typography
- Use system font stack
- Consistent font sizing
- Ligature support (if font supports)

#### 2.4 Theme System
```rust
struct Theme {
    // Core colors
    background: Rgb,
    foreground: Rgb,
    accent: Rgb,
    
    // UI elements
    pane_border_active: Rgb,
    pane_border_inactive: Rgb,
    status_bar_bg: Rgb,
    tab_active_bg: Rgb,
    tab_inactive_bg: Rgb,
    
    // Transparency
    inactive_pane_opacity: f32,
}
```

---

### Phase 3: Advanced Features

#### 3.1 Floating Panes (Zellij-style)
- Overlay panes that float above tiled layout
- Can be pinned to stay visible
- Drag to move, resize handles

#### 3.2 Smart Layout Algorithm
- Auto-placement of new panes
- Balance pane sizes intelligently
- Remember layouts per session

#### 3.3 Visual Mode Indicators
- Large overlay when entering prefix mode
- Mode-specific keybinding hints
- Command palette (CMD+Shift+P style)

#### 3.4 Mouse Support
- Drag pane borders to resize
- Click to focus panes
- Drag tabs to reorder
- Right-click context menu

---

## Implementation Approach

### Architecture

```
┌─────────────────────────────────────┐
│           Tab Bar                   │  <- 28px
├─────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐          │
│  │ Pane 1  │  │ Pane 2  │          │  <- Main content
│  │         │  ├─────────┤          │     with borders
│  │         │  │ Pane 3  │          │
│  └─────────┘  └─────────┘          │
├─────────────────────────────────────┤
│           Status Bar                │  <- 24px
└─────────────────────────────────────┘
```

### Rendering Pipeline

1. **Layout Engine**: Calculate pane positions/sizes
2. **Border Renderer**: Draw pane borders
3. **Terminal Renderer**: Draw terminal content (existing Alacritty code)
4. **UI Overlay**: Draw tab bar, status bar, indicators

### Technology Choices

- **GPU Rendering**: Continue using Alacritty's OpenGL renderer
- **Text Rendering**: Existing crossfont system
- **UI Framework**: Custom immediate-mode UI (similar to Dear ImGui concept)
- **Animation**: Easing functions for smooth transitions

---

## Implementation Plan

### Week 1: Foundation
- [ ] Implement pane border rendering
- [ ] Add active/inactive pane visual distinction
- [ ] Test with multiple panes

### Week 2: Status & Tab Bars
- [ ] Create status bar component
- [ ] Create tab bar component
- [ ] Integrate with multiplexer state

### Week 3: Polish
- [ ] Add smooth transitions
- [ ] Implement theme system
- [ ] Add configuration options

### Week 4: Advanced
- [ ] Mouse interaction for panes
- [ ] Floating pane support
- [ ] Command palette

---

## Design Mockups

### Dark Theme (Default)
```
Background: #1a1a1a
Foreground: #d4d4d4
Accent: #58a6ff

Pane Border Active: #58a6ff
Pane Border Inactive: #444444
Status Bar BG: #0d0d0d
Tab Active BG: #2d2d2d
Tab Inactive BG: transparent
```

### Light Theme
```
Background: #ffffff
Foreground: #1a1a1a
Accent: #0969da

Pane Border Active: #0969da
Pane Border Inactive: #d0d7de
Status Bar BG: #f6f8fa
Tab Active BG: #ffffff
Tab Inactive BG: #f6f8fa
```

---

## Configuration

```toml
[ui]
# Pane borders
pane_borders = true
pane_border_width = 2
pane_border_radius = 4

# Status bar
status_bar = true
status_bar_position = "bottom" # or "top"

# Tab bar
tab_bar = true
tab_bar_position = "top"

# Visual effects
dim_inactive_panes = true
inactive_pane_opacity = 0.85
smooth_transitions = true

# Theme
theme = "dark" # or "light", or custom
```

---

## Success Metrics

1. **Performance**: Maintain Alacritty's <7ms input latency
2. **Visual Polish**: Match or exceed Zellij's UI quality
3. **Usability**: New users can discover features without reading docs
4. **Customization**: Power users can theme everything
