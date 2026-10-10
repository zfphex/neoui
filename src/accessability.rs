use minwin::{Key, Mouse, PlatformWindow, Rect, Window};
use rustc_hash::FxHasher;
use std::hash::Hasher;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Range};

/// Role flags for accessibility and semantic classification.
/// Implemented using standard-library bitfield operations without third-party crates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Role(pub u16);

impl Role {
    pub const NONE: Self = Self(0);
    pub const BUTTON: Self = Self(1 << 0);
    pub const TEXT_INPUT: Self = Self(1 << 1);
    pub const SLIDER: Self = Self(1 << 2);
    pub const CHECKBOX: Self = Self(1 << 3);
    pub const HEADER: Self = Self(1 << 4);
    pub const LINK: Self = Self(1 << 5);
    pub const SCROLL_AREA: Self = Self(1 << 6);
    pub const CONTAINER: Self = Self(1 << 7);
    pub const IMAGE: Self = Self(1 << 8);
    pub const LABEL: Self = Self(1 << 9);

    /// Combined flag identifying all naturally focusable interactive roles.
    pub const FOCUSABLE: Self =
        Self(Self::BUTTON.0 | Self::TEXT_INPUT.0 | Self::SLIDER.0 | Self::CHECKBOX.0 | Self::LINK.0);

    #[inline(always)]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub const fn all() -> Self {
        Self(0xFFFF)
    }

    #[inline(always)]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline(always)]
    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    #[inline(always)]
    pub const fn is_focusable(self) -> bool {
        (self.0 & Self::FOCUSABLE.0) != 0
    }

    #[inline(always)]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline(always)]
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    #[inline(always)]
    pub const fn bits(self) -> u16 {
        self.0
    }

    #[inline(always)]
    pub const fn from_bits_truncate(bits: u16) -> Self {
        Self(bits)
    }
}

impl BitOr for Role {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Role {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for Role {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Role {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Not for Role {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> Self {
        Self(!self.0)
    }
}

/// State flags representing UI element interactivity and accessibility state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct StateFlags(pub u8);

impl StateFlags {
    pub const NONE: Self = Self(0);
    pub const DISABLED: Self = Self(1 << 0);
    pub const CHECKED: Self = Self(1 << 1);
    pub const EXPANDED: Self = Self(1 << 2);
    pub const FOCUSED: Self = Self(1 << 3);
    pub const HOVERED: Self = Self(1 << 4);
    pub const SELECTED: Self = Self(1 << 5);

    #[inline(always)]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline(always)]
    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    #[inline(always)]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline(always)]
    pub const fn bits(self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub const fn from_bits_truncate(bits: u8) -> Self {
        Self(bits)
    }
}

impl BitOr for StateFlags {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for StateFlags {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for StateFlags {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for StateFlags {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Not for StateFlags {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> Self {
        Self(!self.0)
    }
}

/// A compact, flat semantic node emitted sequentially by widgets each frame.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SemanticNode {
    /// Screen-space bounding box.
    pub bounds: Rect,
    /// Byte range index into the linear string arena.
    pub text_range: (u32, u32),
    /// Byte range index for accessibility hint into the linear string arena.
    pub hint_range: (u32, u32),
    /// Bitfield specifying semantic roles (BUTTON, HEADER, LINK, etc.).
    pub role: Role,
    /// Bitfield specifying accessibility state (DISABLED, CHECKED, etc.).
    pub state: StateFlags,
    /// Depth layer corresponding to neoui's 0..15 depth stack.
    pub depth: u8,
    /// Fast 32-bit hash of the node's text content.
    pub text_signature: u32,
}

impl SemanticNode {
    #[inline(always)]
    pub fn new(
        bounds: Rect,
        text_range: Range<u32>,
        role: Role,
        state: StateFlags,
        depth: usize,
        text_signature: u32,
    ) -> Self {
        Self {
            bounds,
            text_range: (text_range.start, text_range.end),
            hint_range: (0, 0),
            role,
            state,
            depth: depth as u8,
            text_signature,
        }
    }

    #[inline(always)]
    pub fn with_hint(mut self, hint_range: Range<u32>) -> Self {
        self.hint_range = (hint_range.start, hint_range.end);
        self
    }

    #[inline(always)]
    pub fn text<'a>(&self, arena: &'a str) -> &'a str {
        &arena[self.text_range.0 as usize..self.text_range.1 as usize]
    }

    #[inline(always)]
    pub fn hint<'a>(&self, arena: &'a str) -> &'a str {
        &arena[self.hint_range.0 as usize..self.hint_range.1 as usize]
    }

    /// Screen-space centroid of this node.
    #[inline(always)]
    pub fn centroid(&self) -> (f32, f32) {
        (
            self.bounds.x as f32 + (self.bounds.width as f32) * 0.5,
            self.bounds.y as f32 + (self.bounds.height as f32) * 0.5,
        )
    }
}

/// The spatial focus anchor representing the active focus target across frames.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialCursor {
    /// Continuous 2D sub-pixel coordinate (centroid of active element).
    pub point: (f32, f32),
    /// Preferred column/row for a run of arrow navigation, separate from focus identity.
    pub navigation_anchor: (f32, f32),
    pub navigation_direction: Option<Direction>,
    /// Role flags of the active element.
    pub role: Role,
    /// 32-bit content signature of the active element's text.
    pub text_signature: u32,
    /// Index resolved in the current/previous frame semantic array.
    pub stream_index: usize,
    /// Depth layer of the focused element.
    pub depth: usize,
}

impl SpatialCursor {
    pub fn new(point: (f32, f32), role: Role, text_signature: u32, stream_index: usize, depth: usize) -> Self {
        Self {
            point,
            navigation_anchor: point,
            navigation_direction: None,
            role,
            text_signature,
            stream_index,
            depth,
        }
    }
}

/// 2D Cardinal Direction for spatial navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    #[inline(always)]
    pub fn vector(self) -> (f32, f32) {
        match self {
            Direction::Right => (1.0, 0.0),
            Direction::Left => (-1.0, 0.0),
            Direction::Down => (0.0, 1.0),
            Direction::Up => (0.0, -1.0),
        }
    }
}

#[inline]
pub fn hash32(text: &str) -> u32 {
    let mut hasher = FxHasher::default();
    hasher.write(text.as_bytes());
    hasher.finish() as u32
}

#[inline(always)]
pub fn dist_sq((x1, y1): (f32, f32), (x2, y2): (f32, f32)) -> f32 {
    let dx = x1 - x2;
    let dy = y1 - y2;
    dx * dx + dy * dy
}

#[inline(always)]
pub fn rect_contains_point(r: Rect, (px, py): (f32, f32)) -> bool {
    let x0 = r.x as f32;
    let y0 = r.y as f32;
    let x1 = (r.x + r.width) as f32;
    let y1 = (r.y + r.height) as f32;
    px >= x0 && px < x1 && py >= y0 && py < y1
}

/// The three-tier geometric snap algorithm.
/// Reconciles focus from Frame N-1 to Frame N.
///
/// Tier 1 (Containment): Exact Point-in-Rect Hit Test.
/// Tier 2 (Localized Shift): Search radius R for matching role and text signature.
/// Tier 3 (Deletion Fallback): Directional / nearest neighbor focusable element.
pub fn snap_focus(
    nodes: &[SemanticNode],
    cursor: &mut SpatialCursor,
    search_radius: f32,
    active_depth: Option<usize>,
) -> bool {
    let search_radius_sq = search_radius * search_radius;
    let mut exact = None;
    let mut shifted = None;
    let mut containing = None;
    let mut nearest = None;
    let mut shifted_dist_sq = f32::MAX;
    let mut nearest_dist_sq = f32::MAX;

    for (i, node) in nodes.iter().enumerate() {
        if node.state.contains(StateFlags::DISABLED) {
            continue;
        }
        if let Some(depth) = active_depth {
            if (node.depth as usize) < depth {
                continue;
            }
        }

        let matching_role = node.role.intersects(cursor.role);
        if matching_role && rect_contains_point(node.bounds, cursor.point) {
            if cursor.text_signature == 0 || node.text_signature == cursor.text_signature {
                exact = Some(i);
                break;
            }
            if containing.is_none() {
                containing = Some(i);
            }
        }

        let d_sq = dist_sq(cursor.point, node.centroid());
        if matching_role
            && node.text_signature == cursor.text_signature
            && d_sq <= search_radius_sq
            && d_sq < shifted_dist_sq
        {
            shifted_dist_sq = d_sq;
            shifted = Some(i);
        }
        if node.role.is_focusable() && d_sq < nearest_dist_sq {
            nearest_dist_sq = d_sq;
            nearest = Some(i);
        }
    }

    if let Some(idx) = exact.or(shifted).or(containing).or(nearest) {
        let node = &nodes[idx];
        if exact.is_none() && shifted.is_none() {
            cursor.navigation_anchor = node.centroid();
            cursor.navigation_direction = None;
        }
        cursor.stream_index = idx;
        cursor.point = node.centroid();
        cursor.role = node.role;
        cursor.text_signature = node.text_signature;
        cursor.depth = node.depth as usize;
        return true;
    }

    false
}

/// Sequential navigation (Tab / Shift-Tab) across the flat semantic stream.
pub fn navigate_sequential(
    nodes: &[SemanticNode],
    cursor: &mut SpatialCursor,
    forward: bool,
    active_depth: Option<usize>,
) -> bool {
    navigate_semantic(nodes, cursor, Role::FOCUSABLE, forward, active_depth)
}

/// Arrow navigation prefers candidates beyond the source rectangle's facing edge.
/// Overlapping candidates are a fallback only if they extend beyond that edge.
/// The orthogonal anchor survives wide/narrow items and frame reconciliation.
pub fn navigate_directional(
    nodes: &[SemanticNode],
    cursor: &mut SpatialCursor,
    direction: Direction,
    alpha: f32,
    active_depth: Option<usize>,
) -> bool {
    let Some(source) = nodes.get(cursor.stream_index) else {
        return false;
    };
    let horizontal = matches!(direction, Direction::Left | Direction::Right);
    let same_axis = cursor
        .navigation_direction
        .is_some_and(|previous| matches!(previous, Direction::Left | Direction::Right) == horizontal);
    let anchor = if same_axis { cursor.navigation_anchor } else { cursor.point };
    let s = source.bounds;
    let edge = match direction {
        Direction::Right => (s.x + s.width) as f32,
        Direction::Left => s.x as f32,
        Direction::Down => (s.y + s.height) as f32,
        Direction::Up => s.y as f32,
    };
    let mut best: Option<(bool, f32, usize)> = None;
    for (i, node) in nodes.iter().enumerate() {
        if i == cursor.stream_index
            || node.state.contains(StateFlags::DISABLED)
            || !node.role.is_focusable()
            || node.bounds.is_empty()
            || active_depth.is_some_and(|depth| (node.depth as usize) < depth)
        {
            continue;
        }
        let r = node.bounds;
        let (near, far, orthogonal_start, orthogonal_end, preferred) = match direction {
            Direction::Right => (
                r.x as f32,
                (r.x + r.width) as f32,
                r.y as f32,
                (r.y + r.height) as f32,
                anchor.1,
            ),
            Direction::Left => (
                -(r.x + r.width) as f32,
                -r.x as f32,
                r.y as f32,
                (r.y + r.height) as f32,
                anchor.1,
            ),
            Direction::Down => (
                r.y as f32,
                (r.y + r.height) as f32,
                r.x as f32,
                (r.x + r.width) as f32,
                anchor.0,
            ),
            Direction::Up => (
                -(r.y + r.height) as f32,
                -r.y as f32,
                r.x as f32,
                (r.x + r.width) as f32,
                anchor.0,
            ),
        };
        let origin = if matches!(direction, Direction::Left | Direction::Up) { -edge } else { edge };
        if far <= origin {
            continue;
        }
        let overlapping = near < origin - 1.0;
        let primary = (near - origin).max(0.0);
        let secondary = (orthogonal_start - preferred).max(preferred - orthogonal_end).max(0.0);
        // Distance to the nearest point on the candidate, not its centre.
        let cost = primary + secondary * (1.0 + alpha.max(0.0) * 2.0);
        if best.is_none_or(|(was_overlapping, best_cost, _)| (overlapping, cost) < (was_overlapping, best_cost)) {
            best = Some((overlapping, cost, i));
        }
    }
    let Some((_, _, idx)) = best else {
        return false;
    };
    let node = &nodes[idx];
    cursor.stream_index = idx;
    cursor.point = node.centroid();
    cursor.role = node.role;
    cursor.text_signature = node.text_signature;
    cursor.depth = node.depth as usize;
    cursor.navigation_anchor = anchor;
    cursor.navigation_direction = Some(direction);
    true
}

/// Semantic jumping (e.g. H for Header, L for Link, B for Button).
pub fn navigate_semantic(
    nodes: &[SemanticNode],
    cursor: &mut SpatialCursor,
    target_role: Role,
    forward: bool,
    active_depth: Option<usize>,
) -> bool {
    if nodes.is_empty() {
        return false;
    }

    let count = nodes.len();
    let start_idx = cursor.stream_index;

    for step in 1..=count {
        let idx = if forward {
            (start_idx + step) % count
        } else {
            (start_idx + count - (step % count)) % count
        };

        let node = &nodes[idx];
        if let Some(depth) = active_depth {
            if (node.depth as usize) < depth {
                continue;
            }
        }
        if !node.state.contains(StateFlags::DISABLED) && node.role.intersects(target_role) {
            cursor.stream_index = idx;
            cursor.point = node.centroid();
            cursor.navigation_anchor = cursor.point;
            cursor.navigation_direction = None;
            cursor.role = node.role;
            cursor.text_signature = node.text_signature;
            cursor.depth = node.depth as usize;
            return true;
        }
    }

    false
}

/// Retained accessibility state managed in `UiState`.
#[derive(Debug, Clone)]
pub struct AccessabilityState {
    /// Semantic nodes emitted during the current frame.
    pub current_nodes: Vec<SemanticNode>,
    /// Semantic nodes retained from the previous frame.
    pub prev_nodes: Vec<SemanticNode>,
    /// Linear string arena holding text slices for the current frame.
    pub text_arena: String,
    /// The global spatial focus cursor.
    pub cursor: Option<SpatialCursor>,
    /// Search radius for Tier 2 shift resolution.
    pub search_radius: f32,
    /// Additional penalty for distance from the preferred navigation row/column.
    pub directional_alpha: f32,
    /// Flag indicating whether accessibility keyboard focus visual indicators are active.
    pub keyboard_nav_active: bool,
}

impl AccessabilityState {
    pub fn new() -> Self {
        Self {
            current_nodes: Vec::with_capacity(128),
            prev_nodes: Vec::with_capacity(128),
            text_arena: String::with_capacity(2048),
            cursor: None,
            search_radius: 200.0,
            directional_alpha: 2.0,
            keyboard_nav_active: false,
        }
    }

    /// Process arrow navigation and optional Tab/Shift-Tab traversal against `prev_nodes`,
    /// then clear the frame buffers.
    pub fn begin_frame(&mut self, window: Option<&Window>, active_depth: Option<usize>, tab_navigation: bool) {
        // A mouse press starts a new focus interaction. The clicked widget may
        // claim focus while drawing; a click on empty space leaves it cleared.
        if window.is_some_and(|win| !win.focused() || win.mouse_pressed(Mouse::Left)) {
            self.cursor = None;
            self.keyboard_nav_active = false;
        }
        if let Some(win) = window.filter(|win| win.focused()) {
            let modifiers = win.modifiers();
            let shift = modifiers.shift;
            let tab = tab_navigation && win.pressed(Key::Tab);
            let arrow_up = win.pressed(Key::ArrowUp) || win.pressed(Key::Up);
            let arrow_down = win.pressed(Key::ArrowDown) || win.pressed(Key::Down);
            let arrow_left = win.pressed(Key::ArrowLeft) || win.pressed(Key::Left);
            let arrow_right = win.pressed(Key::ArrowRight) || win.pressed(Key::Right);

            if tab || arrow_up || arrow_down || arrow_left || arrow_right {
                self.keyboard_nav_active = true;

                if let Some(cursor) = &mut self.cursor {
                    if tab {
                        navigate_sequential(&self.prev_nodes, cursor, !shift, active_depth);
                    } else {
                        let direction = if arrow_right {
                            Direction::Right
                        } else if arrow_left {
                            Direction::Left
                        } else if arrow_down {
                            Direction::Down
                        } else {
                            Direction::Up
                        };
                        navigate_directional(
                            &self.prev_nodes,
                            cursor,
                            direction,
                            self.directional_alpha,
                            active_depth,
                        );
                    }
                } else if !self.prev_nodes.is_empty() {
                    // Initialize cursor on first tab/arrow press to first focusable element
                    for (i, node) in self.prev_nodes.iter().enumerate() {
                        if !node.state.contains(StateFlags::DISABLED) && node.role.is_focusable() {
                            self.cursor = Some(SpatialCursor::new(
                                node.centroid(),
                                node.role,
                                node.text_signature,
                                i,
                                node.depth as usize,
                            ));
                            break;
                        }
                    }
                }
            }
        }

        self.current_nodes.clear();
        self.text_arena.clear();
    }

    /// Check if a node with given bounds and depth is currently focused.
    #[inline]
    pub fn is_focused(&self, bounds: Rect, role: Role) -> bool {
        let Some(cursor) = self.cursor else {
            return false;
        };
        // Check if cursor point is within bounds and role matches
        rect_contains_point(bounds, cursor.point) && (cursor.role.is_empty() || cursor.role.intersects(role))
    }

    /// End of frame focus resolution: snaps cursor to Frame N nodes and swaps buffers.
    pub fn end_frame(&mut self, active_depth: Option<usize>) {
        if let Some(cursor) = &mut self.cursor
            && !snap_focus(&self.current_nodes, cursor, self.search_radius, active_depth)
        {
            self.cursor = None;
            self.keyboard_nav_active = false;
        }

        std::mem::swap(&mut self.prev_nodes, &mut self.current_nodes);
    }

    #[inline(always)]
    pub fn node_text<'a>(&'a self, node: &SemanticNode) -> &'a str {
        node.text(&self.text_arena)
    }

    #[inline(always)]
    pub fn node_hint<'a>(&'a self, node: &SemanticNode) -> &'a str {
        node.hint(&self.text_arena)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buttons(bounds: &[Rect]) -> Vec<SemanticNode> {
        bounds
            .iter()
            .enumerate()
            .map(|(i, &bounds)| SemanticNode::new(bounds, 0..0, Role::BUTTON, StateFlags::NONE, 0, i as u32 + 1))
            .collect()
    }

    fn cursor_at(nodes: &[SemanticNode], index: usize) -> SpatialCursor {
        let n = &nodes[index];
        SpatialCursor::new(n.centroid(), n.role, n.text_signature, index, 0)
    }

    #[test]
    fn focus_clears_when_no_eligible_controls_remain() {
        let nodes = buttons(&[Rect::new(0, 0, 100, 30)]);
        let mut state = AccessabilityState::new();
        state.cursor = Some(cursor_at(&nodes, 0));
        state.keyboard_nav_active = true;
        state.end_frame(None);
        assert!(state.cursor.is_none());
        assert!(!state.keyboard_nav_active);
    }

    #[test]
    fn cleared_focus_is_not_restored_by_frame_reconciliation() {
        let mut state = AccessabilityState::new();
        state.current_nodes = buttons(&[Rect::new(0, 0, 100, 30)]);
        state.end_frame(None);
        assert!(state.cursor.is_none());
    }

    #[test]
    fn wide_task_row_preserves_toolbar_column_across_frames() {
        let nodes = buttons(&[
            Rect::new(0, 0, 100, 30),
            Rect::new(110, 0, 100, 30),
            Rect::new(220, 0, 100, 30),
            Rect::new(0, 50, 320, 30),
        ]);
        for start in 0..3 {
            let mut cursor = cursor_at(&nodes, start);
            assert!(navigate_directional(&nodes, &mut cursor, Direction::Down, 2.0, None));
            assert_eq!(cursor.stream_index, 3);
            assert!(snap_focus(&nodes, &mut cursor, 200.0, None));
            assert!(navigate_directional(&nodes, &mut cursor, Direction::Up, 2.0, None));
            assert_eq!(cursor.stream_index, start);
        }
    }

    #[test]
    fn right_from_library_excludes_toggle_inside_sidebar_column() {
        let nodes = buttons(&[
            Rect::new(8, 60, 264, 36),
            Rect::new(240, 10, 30, 30),
            Rect::new(488, 180, 600, 36),
            Rect::new(488, 218, 600, 36),
        ]);
        let mut cursor = cursor_at(&nodes, 0);
        assert!(navigate_directional(&nodes, &mut cursor, Direction::Right, 2.0, None));
        assert_eq!(cursor.stream_index, 2);
    }

    #[test]
    fn horizontal_anchor_survives_tall_destination() {
        let nodes = buttons(&[
            Rect::new(0, 0, 30, 40),
            Rect::new(0, 50, 30, 40),
            Rect::new(50, 0, 30, 90),
        ]);
        let mut cursor = cursor_at(&nodes, 1);
        assert!(navigate_directional(&nodes, &mut cursor, Direction::Right, 2.0, None));
        snap_focus(&nodes, &mut cursor, 200.0, None);
        assert!(navigate_directional(&nodes, &mut cursor, Direction::Left, 2.0, None));
        assert_eq!(cursor.stream_index, 1);
        navigate_sequential(&nodes, &mut cursor, true, None);
        assert_eq!(cursor.navigation_direction, None);
        assert_eq!(cursor.navigation_anchor, cursor.point);
    }

    #[test]
    fn axis_change_resets_anchor_and_failed_move_preserves_it() {
        let nodes = buttons(&[
            Rect::new(0, 0, 100, 30),
            Rect::new(0, 50, 320, 30),
            Rect::new(340, 50, 30, 30),
        ]);
        let mut cursor = cursor_at(&nodes, 0);
        navigate_directional(&nodes, &mut cursor, Direction::Down, 2.0, None);
        let point = cursor.point;
        navigate_directional(&nodes, &mut cursor, Direction::Right, 2.0, None);
        assert_eq!(cursor.navigation_anchor, point);
        let before = cursor;
        assert!(!navigate_directional(&nodes, &mut cursor, Direction::Right, 2.0, None));
        assert_eq!(cursor, before);
    }

    #[test]
    fn overlapping_fallback_respects_eligibility_and_prefers_separated_nodes() {
        let mut nodes = buttons(&[
            Rect::new(0, 0, 100, 30),
            Rect::new(80, 0, 40, 30),
            Rect::new(200, 0, 30, 30),
        ]);
        let mut cursor = cursor_at(&nodes, 0);
        navigate_directional(&nodes, &mut cursor, Direction::Right, 2.0, None);
        assert_eq!(cursor.stream_index, 2);
        nodes[2].state = StateFlags::DISABLED;
        cursor = cursor_at(&nodes, 0);
        navigate_directional(&nodes, &mut cursor, Direction::Right, 2.0, None);
        assert_eq!(cursor.stream_index, 1);
        cursor = cursor_at(&nodes, 0);
        assert!(!navigate_directional(
            &nodes,
            &mut cursor,
            Direction::Right,
            2.0,
            Some(1)
        ));
    }

    #[test]
    fn test_tier_1_containment() {
        let nodes = vec![
            SemanticNode::new(
                Rect::new(0, 0, 100, 30),
                0..4,
                Role::BUTTON,
                StateFlags::NONE,
                0,
                hash32("Save"),
            ),
            SemanticNode::new(
                Rect::new(0, 40, 100, 30),
                4..10,
                Role::BUTTON,
                StateFlags::NONE,
                0,
                hash32("Cancel"),
            ),
        ];

        let mut cursor = SpatialCursor::new((50.0, 55.0), Role::BUTTON, hash32("Cancel"), 1, 0);
        let snapped = snap_focus(&nodes, &mut cursor, 200.0, None);

        assert!(snapped);
        assert_eq!(cursor.stream_index, 1);
        assert_eq!(cursor.point, (50.0, 55.0));
    }

    #[test]
    fn test_tier_2_shift() {
        // Element "Cancel" shifts down by 60px because a new item was prepended
        let nodes = vec![
            SemanticNode::new(
                Rect::new(0, 0, 100, 30),
                0..3,
                Role::BUTTON,
                StateFlags::NONE,
                0,
                hash32("New"),
            ),
            SemanticNode::new(
                Rect::new(0, 40, 100, 30),
                3..7,
                Role::BUTTON,
                StateFlags::NONE,
                0,
                hash32("Save"),
            ),
            SemanticNode::new(
                Rect::new(0, 100, 100, 30),
                7..13,
                Role::BUTTON,
                StateFlags::NONE,
                0,
                hash32("Cancel"),
            ),
        ];

        // Cursor was at (50.0, 55.0) where "Cancel" was on previous frame
        let mut cursor = SpatialCursor::new((50.0, 55.0), Role::BUTTON, hash32("Cancel"), 1, 0);
        let snapped = snap_focus(&nodes, &mut cursor, 200.0, None);

        assert!(snapped);
        assert_eq!(cursor.stream_index, 2); // Snapped to shifted "Cancel"
        assert_eq!(cursor.point, (50.0, 115.0));
    }

    #[test]
    fn test_tier_3_deletion() {
        // "Cancel" was deleted; cursor was at (50.0, 100.0)
        let nodes = vec![
            SemanticNode::new(
                Rect::new(0, 0, 100, 30),
                0..4,
                Role::BUTTON,
                StateFlags::NONE,
                0,
                hash32("Save"),
            ),
            SemanticNode::new(
                Rect::new(0, 40, 100, 30),
                4..9,
                Role::BUTTON,
                StateFlags::NONE,
                0,
                hash32("Apply"),
            ),
        ];

        let mut cursor = SpatialCursor::new((50.0, 100.0), Role::BUTTON, hash32("Deleted"), 2, 0);
        let snapped = snap_focus(&nodes, &mut cursor, 200.0, None);

        assert!(snapped);
        assert_eq!(cursor.stream_index, 1); // Snapped to nearest "Apply"
        assert_eq!(cursor.point, (50.0, 55.0));
    }

    #[test]
    fn test_sequential_tab_navigation() {
        let nodes = vec![
            SemanticNode::new(
                Rect::new(0, 0, 100, 30),
                0..4,
                Role::BUTTON,
                StateFlags::NONE,
                0,
                hash32("Btn1"),
            ),
            SemanticNode::new(
                Rect::new(0, 40, 100, 30),
                4..10,
                Role::HEADER,
                StateFlags::NONE,
                0,
                hash32("Header"),
            ),
            SemanticNode::new(
                Rect::new(0, 80, 100, 30),
                10..14,
                Role::BUTTON,
                StateFlags::NONE,
                0,
                hash32("Btn2"),
            ),
        ];

        let mut cursor = SpatialCursor::new((50.0, 15.0), Role::BUTTON, hash32("Btn1"), 0, 0);

        // Tab forward -> skips Header (not focusable) -> reaches Btn2 (index 2)
        assert!(navigate_sequential(&nodes, &mut cursor, true, None));
        assert_eq!(cursor.stream_index, 2);

        // Tab forward again -> wraps to Btn1 (index 0)
        assert!(navigate_sequential(&nodes, &mut cursor, true, None));
        assert_eq!(cursor.stream_index, 0);

        // Shift-Tab backward -> wraps to Btn2 (index 2)
        assert!(navigate_sequential(&nodes, &mut cursor, false, None));
        assert_eq!(cursor.stream_index, 2);
    }

    #[test]
    fn test_directional_navigation() {
        // Grid layout:
        // [Btn (0,0)]    [Btn (120, 0)]
        // [Btn (0,50)]   [Btn (120, 50)]
        let nodes = vec![
            SemanticNode::new(
                Rect::new(0, 0, 100, 30),
                0..2,
                Role::BUTTON,
                StateFlags::NONE,
                0,
                hash32("TL"),
            ),
            SemanticNode::new(
                Rect::new(120, 0, 100, 30),
                2..4,
                Role::BUTTON,
                StateFlags::NONE,
                0,
                hash32("TR"),
            ),
            SemanticNode::new(
                Rect::new(0, 50, 100, 30),
                4..6,
                Role::BUTTON,
                StateFlags::NONE,
                0,
                hash32("BL"),
            ),
            SemanticNode::new(
                Rect::new(120, 50, 100, 30),
                6..8,
                Role::BUTTON,
                StateFlags::NONE,
                0,
                hash32("BR"),
            ),
        ];

        let mut cursor = SpatialCursor::new((50.0, 15.0), Role::BUTTON, hash32("TL"), 0, 0);

        // Navigate Right -> snaps to TR (index 1)
        assert!(navigate_directional(&nodes, &mut cursor, Direction::Right, 2.0, None));
        assert_eq!(cursor.stream_index, 1);

        // Navigate Down -> snaps to BR (index 3)
        assert!(navigate_directional(&nodes, &mut cursor, Direction::Down, 2.0, None));
        assert_eq!(cursor.stream_index, 3);

        // Navigate Left -> snaps to BL (index 2)
        assert!(navigate_directional(&nodes, &mut cursor, Direction::Left, 2.0, None));
        assert_eq!(cursor.stream_index, 2);

        // Navigate Up -> snaps to TL (index 0)
        assert!(navigate_directional(&nodes, &mut cursor, Direction::Up, 2.0, None));
        assert_eq!(cursor.stream_index, 0);
    }
}
