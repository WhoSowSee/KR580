//! Style functions for the UI, sliced by widget category.
//!
//! Submodules each own a single widget kind to keep navigation easy and
//! file sizes within the workspace's 400-line ceiling. Everything is
//! re-exported at the module root so callers just `use super::styles::*`
//! (or pick the helpers they need by name).

mod buttons;
mod containers;
mod inputs;
mod scrollable;

pub(super) use buttons::{
    action_button_style, caption_button_style, close_caption_button_style, enter_button_style,
    menu_button_disabled_style, menu_button_style, opcode_option_style,
    schematic_block_button_style, schematic_select_button_style, step_button_style,
};
pub(super) use containers::{
    app_style, error_inset_style, info_inset_style, input_shell_style, inset_style,
    legend_label_style, memory_row_container_style, menu_bar_divider_style, menu_bar_style,
    mux_chip_style, mux_header_style, mux_panel_style, opcode_dropdown_style, panel_style,
    schematic_block_style, schematic_board_style, solid_style, status_tooltip_style,
    transparent_style,
};
pub(super) use inputs::{inline_value_input_style, input_borderless_style};
pub(super) use scrollable::scrollable_style;
