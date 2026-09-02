use minwin::Rect;
use neoui::*;

#[test]
fn test_in_place_rename_mutation() {
    let nodes = vec![
        SemanticNode::new(
            Rect::new(0, 0, 100, 30),
            0..4,
            Role::BUTTON,
            StateFlags::NONE,
            0,
            hash32("Renamed"),
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

    // Cursor had focus on "Old" at (50.0, 15.0) which was renamed to "Renamed" in place
    let mut cursor = SpatialCursor::new((50.0, 15.0), Role::BUTTON, hash32("Old"), 0, 0);
    let snapped = snap_focus(&nodes, &mut cursor, 200.0, None);

    assert!(snapped);
    assert_eq!(cursor.stream_index, 0); // Kept focus on index 0
    assert_eq!(cursor.point, (50.0, 15.0));
    assert_eq!(cursor.text_signature, hash32("Renamed"));
}

#[test]
fn test_semantic_jumping() {
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
            hash32("Header 1"),
        ),
        SemanticNode::new(
            Rect::new(0, 80, 100, 30),
            10..14,
            Role::BUTTON,
            StateFlags::NONE,
            0,
            hash32("Btn2"),
        ),
        SemanticNode::new(
            Rect::new(0, 120, 100, 30),
            14..20,
            Role::HEADER,
            StateFlags::NONE,
            0,
            hash32("Header 2"),
        ),
        SemanticNode::new(
            Rect::new(0, 160, 100, 30),
            20..24,
            Role::LINK,
            StateFlags::NONE,
            0,
            hash32("Link 1"),
        ),
    ];

    let mut cursor = SpatialCursor::new((50.0, 15.0), Role::BUTTON, hash32("Btn1"), 0, 0);

    // Jump to next HEADER -> Header 1 (index 1)
    assert!(navigate_semantic(&nodes, &mut cursor, Role::HEADER, true, None));
    assert_eq!(cursor.stream_index, 1);

    // Jump to next HEADER -> Header 2 (index 3)
    assert!(navigate_semantic(&nodes, &mut cursor, Role::HEADER, true, None));
    assert_eq!(cursor.stream_index, 3);

    // Jump to next LINK -> Link 1 (index 4)
    assert!(navigate_semantic(&nodes, &mut cursor, Role::LINK, true, None));
    assert_eq!(cursor.stream_index, 4);

    // Jump backward to previous HEADER -> Header 2 (index 3)
    assert!(navigate_semantic(&nodes, &mut cursor, Role::HEADER, false, None));
    assert_eq!(cursor.stream_index, 3);
}

#[test]
fn test_disabled_element_skipped() {
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
            4..8,
            Role::BUTTON,
            StateFlags::DISABLED,
            0,
            hash32("Disabled"),
        ),
        SemanticNode::new(
            Rect::new(0, 80, 100, 30),
            8..12,
            Role::BUTTON,
            StateFlags::NONE,
            0,
            hash32("Btn2"),
        ),
    ];

    let mut cursor = SpatialCursor::new((50.0, 15.0), Role::BUTTON, hash32("Btn1"), 0, 0);

    // Tab forward -> skips Disabled (index 1) -> snaps to Btn2 (index 2)
    assert!(navigate_sequential(&nodes, &mut cursor, true, None));
    assert_eq!(cursor.stream_index, 2);

    // Directional Down -> skips Disabled -> snaps to Btn2
    cursor.stream_index = 0;
    cursor.point = (50.0, 15.0);
    assert!(navigate_directional(&nodes, &mut cursor, Direction::Down, 2.0, None));
    assert_eq!(cursor.stream_index, 2);
}

#[test]
fn test_depth_layer_isolation() {
    let nodes = vec![
        // Background layer 0
        SemanticNode::new(
            Rect::new(10, 10, 80, 30),
            0..4,
            Role::BUTTON,
            StateFlags::NONE,
            0,
            hash32("Bg1"),
        ),
        SemanticNode::new(
            Rect::new(10, 50, 80, 30),
            4..8,
            Role::BUTTON,
            StateFlags::NONE,
            0,
            hash32("Bg2"),
        ),
        // Modal popup layer 1
        SemanticNode::new(
            Rect::new(100, 100, 80, 30),
            8..12,
            Role::BUTTON,
            StateFlags::NONE,
            1,
            hash32("Modal1"),
        ),
        SemanticNode::new(
            Rect::new(100, 140, 80, 30),
            12..16,
            Role::BUTTON,
            StateFlags::NONE,
            1,
            hash32("Modal2"),
        ),
    ];

    let mut cursor = SpatialCursor::new((140.0, 115.0), Role::BUTTON, hash32("Modal1"), 2, 1);

    // When modal at depth 1 is active:
    // Tab forward stays in depth 1 (cycles Modal1 -> Modal2 -> Modal1)
    assert!(navigate_sequential(&nodes, &mut cursor, true, Some(1)));
    assert_eq!(cursor.stream_index, 3); // Modal 2

    assert!(navigate_sequential(&nodes, &mut cursor, true, Some(1)));
    assert_eq!(cursor.stream_index, 2); // Modal 1 (wrapped within depth 1)
}

#[test]
fn test_ui_state_accessability_lifecycle() {
    let mut accessability = AccessabilityState::new();
    accessability.begin_frame(None, None);

    let node1 = SemanticNode::new(
        Rect::new(0, 0, 100, 30),
        0..4,
        Role::BUTTON,
        StateFlags::NONE,
        0,
        hash32("First"),
    );
    let node2 = SemanticNode::new(
        Rect::new(0, 40, 100, 30),
        4..8,
        Role::BUTTON,
        StateFlags::NONE,
        0,
        hash32("Second"),
    );
    accessability.current_nodes.push(node1);
    accessability.current_nodes.push(node2);

    accessability.cursor = Some(SpatialCursor::new((50.0, 15.0), Role::BUTTON, hash32("First"), 0, 0));

    // End frame snaps and swaps current into prev
    accessability.end_frame(None);

    assert_eq!(accessability.prev_nodes.len(), 2);
    assert!(accessability.is_focused(Rect::new(0, 0, 100, 30), Role::BUTTON));
    assert!(!accessability.is_focused(Rect::new(0, 40, 100, 30), Role::BUTTON));
}

#[test]
fn test_static_label_inference_skipped_by_navigation() {
    let nodes = vec![
        SemanticNode::new(
            Rect::new(0, 0, 100, 30),
            0..5,
            Role::HEADER,
            StateFlags::NONE,
            0,
            hash32("Title"),
        ),
        SemanticNode::new(
            Rect::new(0, 40, 100, 30),
            5..9,
            Role::BUTTON,
            StateFlags::NONE,
            0,
            hash32("Btn1"),
        ),
        SemanticNode::new(
            Rect::new(0, 80, 100, 30),
            9..13,
            Role::LABEL,
            StateFlags::NONE,
            0,
            hash32("Desc"),
        ),
        SemanticNode::new(
            Rect::new(0, 120, 100, 30),
            13..17,
            Role::BUTTON,
            StateFlags::NONE,
            0,
            hash32("Btn2"),
        ),
    ];

    let mut cursor = SpatialCursor::new((50.0, 55.0), Role::BUTTON, hash32("Btn1"), 1, 0);

    // Tab forward from Btn1 (index 1) -> skips "Desc" (index 2, RoleFlags::LABEL) -> reaches Btn2 (index 3)
    assert!(navigate_sequential(&nodes, &mut cursor, true, None));
    assert_eq!(cursor.stream_index, 3);

    // Tab forward from Btn2 (index 3) -> skips "Title" (index 0, RoleFlags::HEADER) -> wraps to Btn1 (index 1)
    assert!(navigate_sequential(&nodes, &mut cursor, true, None));
    assert_eq!(cursor.stream_index, 1);
}

#[test]
fn test_directional_row_vs_column_edge_projection() {
    // Toolbar row: [Prepend (index 0)] [Rename (index 1)] [Delete (index 2)]
    // Full width list below: [Task 1 (index 3)]
    let nodes = vec![
        SemanticNode::new(
            Rect::new(16, 260, 184, 30),
            0..7,
            Role::BUTTON,
            StateFlags::NONE,
            0,
            hash32("Prepend"),
        ),
        SemanticNode::new(
            Rect::new(208, 260, 152, 30),
            7..13,
            Role::BUTTON,
            StateFlags::NONE,
            0,
            hash32("Rename"),
        ),
        SemanticNode::new(
            Rect::new(368, 260, 172, 30),
            13..19,
            Role::BUTTON,
            StateFlags::NONE,
            0,
            hash32("Delete"),
        ),
        SemanticNode::new(
            Rect::new(16, 300, 548, 30),
            19..25,
            Role::BUTTON,
            StateFlags::NONE,
            0,
            hash32("Task 1"),
        ),
    ];

    // Start at Rename (index 1)
    let mut cursor = SpatialCursor::new((284.0, 275.0), Role::BUTTON, hash32("Rename"), 1, 0);

    // Pressing Right from Rename must go directly to Delete (index 2), NOT Task 1 (index 3)
    assert!(navigate_directional(&nodes, &mut cursor, Direction::Right, 2.0, None));
    assert_eq!(cursor.stream_index, 2);

    // Pressing Left from Delete goes back to Rename (index 1)
    assert!(navigate_directional(&nodes, &mut cursor, Direction::Left, 2.0, None));
    assert_eq!(cursor.stream_index, 1);

    // Pressing Left from Rename goes to Prepend (index 0)
    assert!(navigate_directional(&nodes, &mut cursor, Direction::Left, 2.0, None));
    assert_eq!(cursor.stream_index, 0);

    // Pressing Down from Rename goes to Task 1 (index 3)
    cursor.stream_index = 1;
    cursor.point = (284.0, 275.0);
    assert!(navigate_directional(&nodes, &mut cursor, Direction::Down, 2.0, None));
    assert_eq!(cursor.stream_index, 3);
}

#[test]
fn test_accessibility_hints() {
    let mut ui = ui("test_hints", 400, 300);
    ui.state.accessability = true;

    ui.frame(|ui| {
        ui.text("Save", text().hint("Saves current file to disk"));
        ui.text("Cancel", text());
    });

    assert_eq!(ui.state.accessability_state.prev_nodes.len(), 2);
    let save_node = &ui.state.accessability_state.prev_nodes[0];
    let cancel_node = &ui.state.accessability_state.prev_nodes[1];

    let arena = &ui.state.accessability_state.text_arena;
    assert_eq!(save_node.text(arena), "Save");
    assert_eq!(save_node.hint(arena), "Saves current file to disk");

    assert_eq!(cancel_node.text(arena), "Cancel");
    assert_eq!(cancel_node.hint(arena), "");
}

