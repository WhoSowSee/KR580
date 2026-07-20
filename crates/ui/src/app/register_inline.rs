use super::messages::RegisterInlineTarget;
use iced::keyboard;
use k580_core::RegisterName;

const SCHEMATIC_REGISTERS: [RegisterName; 3] = [RegisterName::A, RegisterName::B, RegisterName::C];
const MUX_REGISTERS: [RegisterName; 6] = [
    RegisterName::B,
    RegisterName::C,
    RegisterName::D,
    RegisterName::E,
    RegisterName::H,
    RegisterName::L,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RegisterMove {
    Up,
    Down,
    Left,
    Right,
}

pub(crate) fn ctrl_arrow_move(
    key: &keyboard::Key,
    modifiers: keyboard::Modifiers,
) -> Option<RegisterMove> {
    if !modifiers.command() {
        return None;
    }

    match key {
        keyboard::Key::Named(keyboard::key::Named::ArrowUp) => Some(RegisterMove::Up),
        keyboard::Key::Named(keyboard::key::Named::ArrowDown) => Some(RegisterMove::Down),
        keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => Some(RegisterMove::Left),
        keyboard::Key::Named(keyboard::key::Named::ArrowRight) => Some(RegisterMove::Right),
        _ => None,
    }
}

impl RegisterInlineTarget {
    pub(crate) fn tab_adjacent(self, backward: bool) -> Self {
        self.adjacent(backward).unwrap_or(match (self, backward) {
            (Self::Schematic(_), true) => Self::Mux(RegisterName::L),
            (Self::Schematic(_), false) => Self::Mux(RegisterName::B),
            (Self::Mux(_), true) => Self::Schematic(RegisterName::C),
            (Self::Mux(_), false) => Self::Schematic(RegisterName::A),
        })
    }

    pub(crate) fn adjacent(self, backward: bool) -> Option<Self> {
        let order = match self {
            Self::Schematic(_) => &SCHEMATIC_REGISTERS[..],
            Self::Mux(_) => &MUX_REGISTERS[..],
        };

        let register = self.register();
        let index = order.iter().position(|candidate| *candidate == register)?;
        let next = if backward {
            index.checked_sub(1)?
        } else {
            index.checked_add(1).filter(|next| *next < order.len())?
        };

        Some(match self {
            Self::Schematic(_) => Self::Schematic(order[next]),
            Self::Mux(_) => Self::Mux(order[next]),
        })
    }

    pub(crate) fn navigate(self, direction: RegisterMove) -> Option<Self> {
        match self {
            Self::Schematic(register) => {
                navigate_schematic(register, direction).map(Self::Schematic)
            }
            Self::Mux(register) => navigate_mux(register, direction).map(Self::Mux),
        }
    }
}

fn navigate_schematic(register: RegisterName, direction: RegisterMove) -> Option<RegisterName> {
    let index = SCHEMATIC_REGISTERS
        .iter()
        .position(|candidate| *candidate == register)?;

    let next = match direction {
        RegisterMove::Left => index.checked_sub(1)?,
        RegisterMove::Right => index
            .checked_add(1)
            .filter(|next| *next < SCHEMATIC_REGISTERS.len())?,
        RegisterMove::Up | RegisterMove::Down => return None,
    };

    Some(SCHEMATIC_REGISTERS[next])
}

fn navigate_mux(register: RegisterName, direction: RegisterMove) -> Option<RegisterName> {
    let index = MUX_REGISTERS
        .iter()
        .position(|candidate| *candidate == register)?;
    let (row, column) = (index / 2, index % 2);

    let (next_row, next_column) = match direction {
        RegisterMove::Up => (row.checked_sub(1)?, column),
        RegisterMove::Down => (
            row.checked_add(1)
                .filter(|next| *next < MUX_REGISTERS.len() / 2)?,
            column,
        ),
        RegisterMove::Left => (row, column.checked_sub(1)?),
        RegisterMove::Right => (row, column.checked_add(1).filter(|next| *next < 2)?),
    };

    MUX_REGISTERS.get(next_row * 2 + next_column).copied()
}

#[cfg(test)]
mod tests {
    use super::RegisterInlineTarget;
    use crate::app::DesktopApp;
    use k580_core::RegisterName;

    #[test]
    fn schematic_inline_order_walks_buffers_only() {
        use RegisterInlineTarget::Schematic;

        assert_eq!(
            Schematic(RegisterName::A).adjacent(false),
            Some(Schematic(RegisterName::B))
        );
        assert_eq!(
            Schematic(RegisterName::B).adjacent(false),
            Some(Schematic(RegisterName::C))
        );
        assert_eq!(Schematic(RegisterName::C).adjacent(false), None);
        assert_eq!(Schematic(RegisterName::A).adjacent(true), None);
        assert_eq!(
            Schematic(RegisterName::C).adjacent(true),
            Some(Schematic(RegisterName::B))
        );
    }

    #[test]
    fn mux_inline_order_walks_visible_register_grid() {
        use RegisterInlineTarget::Mux;

        assert_eq!(
            Mux(RegisterName::B).adjacent(false),
            Some(Mux(RegisterName::C))
        );
        assert_eq!(
            Mux(RegisterName::C).adjacent(false),
            Some(Mux(RegisterName::D))
        );
        assert_eq!(
            Mux(RegisterName::D).adjacent(false),
            Some(Mux(RegisterName::E))
        );
        assert_eq!(
            Mux(RegisterName::E).adjacent(false),
            Some(Mux(RegisterName::H))
        );
        assert_eq!(
            Mux(RegisterName::H).adjacent(false),
            Some(Mux(RegisterName::L))
        );
        assert_eq!(Mux(RegisterName::L).adjacent(false), None);
        assert_eq!(Mux(RegisterName::B).adjacent(true), None);
        assert_eq!(
            Mux(RegisterName::L).adjacent(true),
            Some(Mux(RegisterName::H))
        );
    }

    #[test]
    fn schematic_arrow_navigation_is_horizontal_only() {
        use super::RegisterMove::{Down, Left, Right, Up};
        use RegisterInlineTarget::Schematic;

        assert_eq!(
            Schematic(RegisterName::A).navigate(Right),
            Some(Schematic(RegisterName::B))
        );
        assert_eq!(
            Schematic(RegisterName::B).navigate(Right),
            Some(Schematic(RegisterName::C))
        );
        assert_eq!(Schematic(RegisterName::C).navigate(Right), None);
        assert_eq!(Schematic(RegisterName::A).navigate(Left), None);
        assert_eq!(
            Schematic(RegisterName::C).navigate(Left),
            Some(Schematic(RegisterName::B))
        );
        assert_eq!(Schematic(RegisterName::B).navigate(Up), None);
        assert_eq!(Schematic(RegisterName::B).navigate(Down), None);
    }

    #[test]
    fn mux_arrow_navigation_respects_columns_and_rows() {
        use super::RegisterMove::{Down, Left, Right, Up};
        use RegisterInlineTarget::Mux;

        assert_eq!(
            Mux(RegisterName::B).navigate(Right),
            Some(Mux(RegisterName::C))
        );
        assert_eq!(Mux(RegisterName::C).navigate(Right), None);
        assert_eq!(
            Mux(RegisterName::C).navigate(Left),
            Some(Mux(RegisterName::B))
        );
        assert_eq!(Mux(RegisterName::B).navigate(Left), None);

        assert_eq!(
            Mux(RegisterName::B).navigate(Down),
            Some(Mux(RegisterName::D))
        );
        assert_eq!(
            Mux(RegisterName::D).navigate(Down),
            Some(Mux(RegisterName::H))
        );
        assert_eq!(Mux(RegisterName::H).navigate(Down), None);
        assert_eq!(
            Mux(RegisterName::L).navigate(Up),
            Some(Mux(RegisterName::E))
        );
        assert_eq!(
            Mux(RegisterName::E).navigate(Up),
            Some(Mux(RegisterName::C))
        );
        assert_eq!(Mux(RegisterName::C).navigate(Up), None);
    }

    #[test]
    fn selecting_memory_clears_register_keyboard_context() {
        let (mut app, _task) = DesktopApp::with_initial_path(None);
        app.select_register_target(RegisterInlineTarget::Mux(RegisterName::B));

        app.select_memory(0x0010);

        assert_eq!(app.active_register_target, None);
        assert_eq!(app.inline_register_target, None);
    }

    #[test]
    fn right_register_editor_value_is_shared_with_register_readouts() {
        let (mut app, _task) = DesktopApp::with_initial_path(None);
        app.select_register(RegisterName::B);
        app.change_register_value("7F".to_owned());

        assert_eq!(app.display_register_value(RegisterName::B), "7F");
        assert_eq!(app.display_register_value(RegisterName::C), "00");
    }

    #[test]
    fn inline_register_value_is_shared_with_matching_readouts() {
        use RegisterInlineTarget::Mux;

        let (mut app, _task) = DesktopApp::with_initial_path(None);
        app.enter_inline_register(Mux(RegisterName::C));
        app.change_inline_register_value(Mux(RegisterName::C), "12".to_owned());

        assert_eq!(app.display_register_value(RegisterName::C), "12");
        assert_eq!(app.display_register_value(RegisterName::B), "00");
    }

    #[test]
    fn ctrl_arrow_navigation_keeps_register_inline_editor_active() {
        use super::RegisterMove::Right;
        use RegisterInlineTarget::Schematic;

        let (mut app, _task) = DesktopApp::with_initial_path(None);
        app.enter_inline_register(Schematic(RegisterName::A));
        app.focused_input = Some(crate::app::REGISTER_INLINE_INPUT_ID);

        let _task = app.navigate_inline_register_target(Right);

        assert_eq!(app.active_register_target, Some(Schematic(RegisterName::B)));
        assert_eq!(app.inline_register_target, Some(Schematic(RegisterName::B)));
        assert_eq!(
            app.focused_input,
            Some(crate::app::REGISTER_INLINE_INPUT_ID)
        );
    }

    #[test]
    fn ctrl_arrow_keys_map_to_register_moves() {
        let modifiers = iced::keyboard::Modifiers::CTRL;

        assert_eq!(
            super::ctrl_arrow_move(
                &iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft),
                modifiers
            ),
            Some(super::RegisterMove::Left)
        );
        assert_eq!(
            super::ctrl_arrow_move(
                &iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight),
                modifiers
            ),
            Some(super::RegisterMove::Right)
        );
        assert_eq!(
            super::ctrl_arrow_move(&iced::keyboard::Key::Character("a".into()), modifiers),
            None
        );
        assert_eq!(
            super::ctrl_arrow_move(
                &iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft),
                iced::keyboard::Modifiers::default()
            ),
            None
        );
    }

    #[test]
    fn focus_reconcile_outside_inline_register_cancels_edit() {
        use RegisterInlineTarget::Schematic;

        let (mut app, _task) = DesktopApp::with_initial_path(None);
        app.enter_inline_register(Schematic(RegisterName::A));
        app.focused_input = Some(crate::app::REGISTER_INLINE_INPUT_ID);
        let _task = app.handle_focus_reconciled(0, None);
        assert_eq!(app.inline_register_target, Some(Schematic(RegisterName::A)));

        let _task = app.handle_focus_reconciled(0, None);

        assert_eq!(app.inline_register_target, None);
        assert_eq!(app.focused_input, None);
        assert_eq!(app.active_register_target, Some(Schematic(RegisterName::A)));
    }

    #[test]
    fn focus_reconcile_on_inline_register_input_keeps_edit() {
        use RegisterInlineTarget::Mux;

        let (mut app, _task) = DesktopApp::with_initial_path(None);
        app.enter_inline_register(Mux(RegisterName::B));
        app.focused_input = Some(crate::app::REGISTER_INLINE_INPUT_ID);
        let _task = app.handle_focus_reconciled(0, None);

        let hit = iced::widget::Id::new(crate::app::REGISTER_INLINE_INPUT_ID);
        let _task = app.handle_focus_reconciled(0, Some(hit));

        assert_eq!(app.inline_register_target, Some(Mux(RegisterName::B)));
        assert_eq!(
            app.focused_input,
            Some(crate::app::REGISTER_INLINE_INPUT_ID)
        );
    }

    #[test]
    fn focus_reconcile_keeps_inline_open_on_entry_frame() {
        use RegisterInlineTarget::Mux;

        let (mut app, _task) = DesktopApp::with_initial_path(None);
        app.enter_inline_register(Mux(RegisterName::C));
        assert!(app.inline_register_just_entered);

        let _task = app.handle_focus_reconciled(0, None);

        assert_eq!(app.inline_register_target, Some(Mux(RegisterName::C)));
        assert!(!app.inline_register_just_entered);
    }
}
