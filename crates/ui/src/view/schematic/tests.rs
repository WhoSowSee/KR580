#[test]
fn central_column_status_gap_and_height_are_fixed() {
    assert_eq!(
        super::CENTRAL_COLUMN_SECTION_SPACING,
        super::LEFT_BOARD_SECTION_SPACING + super::LEGEND_LINE_OFFSET
    );
    assert_eq!(super::CENTRAL_STATUS_REGISTER_SPACING_TRIM, 4.0);
    assert_eq!(super::super::chips::SCHEMATIC_WIDE_READOUT_HEIGHT, 60.0);
    assert_eq!(super::FULLSCREEN_SCHEMATIC_MIN_HEIGHT, 900.0);
    assert_eq!(super::FULLSCREEN_SCHEMATIC_COLUMN_GAP, 72.0);
}
