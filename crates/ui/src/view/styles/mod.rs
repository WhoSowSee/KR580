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
    capsule_button_style, cell_button_style, enter_button_style, menu_button_style,
    mux_button_style, opcode_option_style, step_button_style, value_button_style,
};
pub(super) use containers::{
    alu_style, app_style, board_style, input_shell_style, inset_style, legend_label_style,
    memory_row_container_style, menu_bar_style, mux_header_style, mux_panel_style,
    opcode_dropdown_style, panel_style, schematic_block_style, solid_style, transparent_style,
};
pub(super) use inputs::{inline_value_input_style, input_borderless_style, input_style};
pub(super) use scrollable::scrollable_style;
