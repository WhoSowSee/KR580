pub(super) const DIALOG_WIDTH: f32 = 760.0;
pub(super) const DIALOG_HEIGHT: f32 = 496.0;
pub(super) const SIDEBAR_WIDTH: f32 = 220.0;
pub(super) const HEADER_HEIGHT: f32 = 52.0;
pub(super) const FOOTER_HEIGHT: f32 = 56.0;
pub(super) const SEARCH_ICON_SIZE: f32 = 13.0;
pub(super) const DROPDOWN_CHEVRON_SIZE: f32 = 14.0;
/// Width of the language anchor + dropdown panel – tight fit around
/// the longest translated name ("Английский" / "English") plus the
/// chevron and a touch of breath.
pub(super) const LANGUAGE_PICKER_WIDTH: f32 = 104.0;
/// Width of the "label + hint" left column. 4 × `SPEED_SEGMENT_WIDTH`
/// + 3 × 6 px gaps = 290 px — fits inside the ~299 px right column.
pub(super) const LABEL_COLUMN_WIDTH: f32 = 180.0;
pub(super) const CONTENT_PADDING: f32 = 20.0;
/// Height of one setting row (label 14pt + 4-px gap + hint 11pt). The
/// language dropdown overlay re-uses this value to position itself
/// directly under the anchor without re-measuring the row.
pub(super) const SETTING_ROW_HEIGHT: f32 = 44.0;
pub(super) const SPEED_SEGMENT_WIDTH: f32 = 68.0;
pub(super) const TOGGLE_SEGMENT_WIDTH: f32 = 96.0;
