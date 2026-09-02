use minwin::{Key, PlatformWindow, Rect, Window};
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
    if nodes.is_empty() {
        return false;
    }

    let search_radius_sq = search_radius * search_radius;

    // Tier 1: Exact Point-in-Rect Containment with Matching Signature
    // If the active point is still inside a node of matching role and matching text signature (same element in place), snap immediately.
    for (i, node) in nodes.iter().enumerate() {
        if let Some(depth) = active_depth {
            if (node.depth as usize) < depth {
                continue;
            }
        }
        if !node.state.contains(StateFlags::DISABLED)
            && node.role.intersects(cursor.role)
            && rect_contains_point(node.bounds, cursor.point)
            && (cursor.text_signature == 0 || node.text_signature == cursor.text_signature)
        {
            cursor.stream_index = i;
            cursor.point = node.centroid();
            cursor.role = node.role;
            cursor.text_signature = node.text_signature;
            cursor.depth = node.depth as usize;
            return true;
        }
    }

    // Tier 2: Localized Shift (Neighborhood Signature Match)
    // If the element moved (e.g. layout reflow, scrolling, prepend), find matching node within radius R.
    let mut best_tier2_idx: Option<usize> = None;
    let mut best_tier2_dist_sq = f32::MAX;

    for (i, node) in nodes.iter().enumerate() {
        if let Some(depth) = active_depth {
            if (node.depth as usize) < depth {
                continue;
            }
        }
        if node.state.contains(StateFlags::DISABLED) {
            continue;
        }

        if node.role.intersects(cursor.role) && node.text_signature == cursor.text_signature {
            let d_sq = dist_sq(cursor.point, node.centroid());
            if d_sq <= search_radius_sq && d_sq < best_tier2_dist_sq {
                best_tier2_dist_sq = d_sq;
                best_tier2_idx = Some(i);
            }
        }
    }

    if let Some(idx) = best_tier2_idx {
        let node = &nodes[idx];
        cursor.stream_index = idx;
        cursor.point = node.centroid();
        cursor.role = node.role;
        cursor.text_signature = node.text_signature;
        cursor.depth = node.depth as usize;
        return true;
    }

    // Tier 1b: In-Place Element Mutation (Rename Label in place)
    // If the element at cursor.point has matching role (even if label changed), lock focus in place.
    for (i, node) in nodes.iter().enumerate() {
        if let Some(depth) = active_depth {
            if (node.depth as usize) < depth {
                continue;
            }
        }
        if !node.state.contains(StateFlags::DISABLED)
            && node.role.intersects(cursor.role)
            && rect_contains_point(node.bounds, cursor.point)
        {
            cursor.stream_index = i;
            cursor.point = node.centroid();
            cursor.role = node.role;
            cursor.text_signature = node.text_signature;
            cursor.depth = node.depth as usize;
            return true;
        }
    }

    //Tier 3: Deletion Fallback (Nearest Focusable Element)
    // If the focused element was deleted or moved beyond radius R, snap to the nearest focusable node.
    let mut best_tier3_idx: Option<usize> = None;
    let mut best_tier3_dist_sq = f32::MAX;

    for (i, node) in nodes.iter().enumerate() {
        if let Some(depth) = active_depth {
            if (node.depth as usize) < depth {
                continue;
            }
        }
        if node.state.contains(StateFlags::DISABLED) {
            continue;
        }

        if node.role.is_focusable() {
            let d_sq = dist_sq(cursor.point, node.centroid());
            if d_sq < best_tier3_dist_sq {
                best_tier3_dist_sq = d_sq;
                best_tier3_idx = Some(i);
            }
        }
    }

    if let Some(idx) = best_tier3_idx {
        let node = &nodes[idx];
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
        if !node.state.contains(StateFlags::DISABLED) && node.role.is_focusable() {
            cursor.stream_index = idx;
            cursor.point = node.centroid();
            cursor.role = node.role;
            cursor.text_signature = node.text_signature;
            cursor.depth = node.depth as usize;
            return true;
        }
    }

    false
}

/// Spatial 2D directional navigation (Arrow Keys).
/// Projects directional edge-to-edge distances and cone constraints from `cursor.point`
/// across focusable candidate nodes, minimizing orthogonal secondary-axis drift.
pub fn navigate_directional(
    nodes: &[SemanticNode],
    cursor: &mut SpatialCursor,
    direction: Direction,
    alpha: f32,
    active_depth: Option<usize>,
) -> bool {
    if nodes.is_empty() {
        return false;
    }

    let (dx, dy) = direction.vector();
    let (cx, cy) = cursor.point;

    let mut best_idx: Option<usize> = None;
    let mut best_cost = f32::MAX;

    // Structured for auto-vectorization over candidate node chunks
    for (i, node) in nodes.iter().enumerate() {
        if let Some(depth) = active_depth {
            if (node.depth as usize) < depth {
                continue;
            }
        }
        if node.state.contains(StateFlags::DISABLED) || !node.role.is_focusable() {
            continue;
        }

        let (tx, ty) = node.centroid();
        let vx = tx - cx;
        let vy = ty - cy;
        let d_sq = vx * vx + vy * vy;

        if d_sq < 0.25 {
            // Skip currently focused element
            continue;
        }

        let d = d_sq.sqrt();
        let dot = (vx * dx + vy * dy) / d;

        // Bounding box edge-to-edge projection
        let r = node.bounds;
        let x0 = r.x as f32;
        let y0 = r.y as f32;
        let x1 = (r.x + r.width) as f32;
        let y1 = (r.y + r.height) as f32;

        let (primary_dist, secondary_dist, is_forward) = match direction {
            Direction::Right => {
                let forward = x1 > cx + 1.0;
                let p = if x0 >= cx { x0 - cx } else { 0.0 };
                let s = if y0 <= cy && cy <= y1 {
                    0.0
                } else if y1 < cy {
                    cy - y1
                } else {
                    y0 - cy
                };
                (p, s, forward)
            }
            Direction::Left => {
                let forward = x0 < cx - 1.0;
                let p = if x1 <= cx { cx - x1 } else { 0.0 };
                let s = if y0 <= cy && cy <= y1 {
                    0.0
                } else if y1 < cy {
                    cy - y1
                } else {
                    y0 - cy
                };
                (p, s, forward)
            }
            Direction::Down => {
                let forward = y1 > cy + 1.0;
                let p = if y0 >= cy { y0 - cy } else { 0.0 };
                let s = if x0 <= cx && cx <= x1 {
                    0.0
                } else if x1 < cx {
                    cx - x1
                } else {
                    x0 - cx
                };
                (p, s, forward)
            }
            Direction::Up => {
                let forward = y0 < cy - 1.0;
                let p = if y1 <= cy { cy - y1 } else { 0.0 };
                let s = if x0 <= cx && cx <= x1 {
                    0.0
                } else if x1 < cx {
                    cx - x1
                } else {
                    x0 - cx
                };
                (p, s, forward)
            }
        };

        if !is_forward {
            continue;
        }

        // Candidate must be within directional cone (cos(theta) >= 0.5) or have direct primary-axis projection overlap
        if dot < 0.5 && secondary_dist > 0.0 {
            continue;
        }

        let cos_theta = dot.clamp(-1.0, 1.0);
        let cost = primary_dist + secondary_dist * (1.0 + alpha * 2.0) + (1.0 - cos_theta) * 30.0;

        if cost < best_cost {
            best_cost = cost;
            best_idx = Some(i);
        }
    }

    if let Some(idx) = best_idx {
        let node = &nodes[idx];
        cursor.stream_index = idx;
        cursor.point = node.centroid();
        cursor.role = node.role;
        cursor.text_signature = node.text_signature;
        cursor.depth = node.depth as usize;
        return true;
    }

    false
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
    /// Angle cost factor for 2D directional cone navigation.
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

    /// Pre-frame input dispatch: processes Tab/Arrow keyboard navigation against `prev_nodes` and clears frame buffers.
    pub fn begin_frame(&mut self, window: Option<&Window>, active_depth: Option<usize>) {
        if let Some(win) = window {
            let modifiers = win.modifiers();
            let shift = modifiers.shift;
            let tab = win.pressed(Key::Tab);
            let arrow_up = win.pressed(Key::ArrowUp) || win.pressed(Key::Up);
            let arrow_down = win.pressed(Key::ArrowDown) || win.pressed(Key::Down);
            let arrow_left = win.pressed(Key::ArrowLeft) || win.pressed(Key::Left);
            let arrow_right = win.pressed(Key::ArrowRight) || win.pressed(Key::Right);

            if tab || arrow_up || arrow_down || arrow_left || arrow_right {
                self.keyboard_nav_active = true;

                if let Some(cursor) = &mut self.cursor {
                    if tab {
                        navigate_sequential(&self.prev_nodes, cursor, !shift, active_depth);
                    } else if arrow_right {
                        navigate_directional(
                            &self.prev_nodes,
                            cursor,
                            Direction::Right,
                            self.directional_alpha,
                            active_depth,
                        );
                    } else if arrow_left {
                        navigate_directional(
                            &self.prev_nodes,
                            cursor,
                            Direction::Left,
                            self.directional_alpha,
                            active_depth,
                        );
                    } else if arrow_down {
                        navigate_directional(
                            &self.prev_nodes,
                            cursor,
                            Direction::Down,
                            self.directional_alpha,
                            active_depth,
                        );
                    } else if arrow_up {
                        navigate_directional(
                            &self.prev_nodes,
                            cursor,
                            Direction::Up,
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
        if let Some(cursor) = &mut self.cursor {
            snap_focus(&self.current_nodes, cursor, self.search_radius, active_depth);
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
