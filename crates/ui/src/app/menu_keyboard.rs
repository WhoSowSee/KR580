use iced::Task;
use iced::keyboard;

use super::messages::{MenuId, Message, TopMenuFocus, TopMenuIndicator};
use super::state::DesktopApp;

const TOP_MENUS: [MenuId; 5] = [
    MenuId::File,
    MenuId::Mp,
    MenuId::View,
    MenuId::Settings,
    MenuId::Help,
];
const FILE_MENU_ACTIONS: &[Message] = &[
    Message::NewFile,
    Message::OpenSnapshot,
    Message::SaveSnapshot,
    Message::SaveSnapshotAs,
    Message::Import,
    Message::Export,
];
const MP_MENU_ACTIONS: &[Message] = &[
    Message::ToggleRun,
    Message::StepInstruction,
    Message::StepTact,
    Message::ResetRam,
    Message::ResetCpu,
    Message::ClearHalt,
];
const VIEW_MENU_ACTIONS: &[Message] = &[
    Message::OpenMonitor,
    Message::OpenFloppy,
    Message::OpenHdd,
    Message::OpenNetwork,
    Message::OpenPrinter,
    Message::ToggleStackView,
];
const HELP_MENU_ACTIONS: &[Message] = &[Message::OpenHelp, Message::OpenAbout];

pub(super) fn navigation_key_message(
    key: &keyboard::Key,
    modifiers: keyboard::Modifiers,
) -> Option<Message> {
    match key {
        keyboard::Key::Named(keyboard::key::Named::Tab) => Some(Message::FocusCycle {
            backward: modifiers.shift(),
        }),
        keyboard::Key::Named(keyboard::key::Named::ArrowUp) => Some(Message::ArrowKey(1)),
        keyboard::Key::Named(keyboard::key::Named::ArrowDown) => Some(Message::ArrowKey(-1)),
        keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
            Some(Message::HorizontalArrowKey(-1))
        }
        keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
            Some(Message::HorizontalArrowKey(1))
        }
        keyboard::Key::Named(keyboard::key::Named::Enter) => Some(Message::EnterPressed),
        _ => None,
    }
}

impl DesktopApp {
    pub(crate) fn route_open_menu_message(&mut self, message: &Message) -> Option<Task<Message>> {
        if self.open_menu.is_none() && self.top_menu_focus.is_none() {
            return None;
        }
        match message {
            Message::ArrowKey(direction) => {
                self.move_top_menu_item(*direction);
                Some(Task::none())
            }
            Message::HorizontalArrowKey(direction) => {
                self.move_top_menu_category(*direction);
                Some(Task::none())
            }
            Message::FocusCycle { backward } => {
                self.cycle_top_menu_focus(*backward);
                Some(Task::none())
            }
            Message::EnterPressed => Some(self.activate_top_menu_focus()),
            Message::MousePressed | Message::MousePressedIgnored => {
                if self.open_menu.is_some() {
                    self.top_menu_indicator = TopMenuIndicator::Hidden;
                } else {
                    self.close_top_menu();
                }
                None
            }
            _ => None,
        }
    }

    pub(crate) fn toggle_top_menu(&mut self, menu: MenuId) {
        if self.open_menu == Some(menu) {
            self.close_top_menu();
        } else {
            self.open_menu = Some(menu);
            self.top_menu_focus = Some(TopMenuFocus::Category(menu));
            self.top_menu_indicator = TopMenuIndicator::Hidden;
        }
    }

    pub(crate) fn close_top_menu(&mut self) {
        self.open_menu = None;
        self.top_menu_focus = None;
        self.top_menu_indicator = TopMenuIndicator::Hidden;
    }

    fn move_top_menu_item(&mut self, direction: i32) {
        let Some(menu) = self.open_menu else {
            return;
        };
        let count = top_menu_item_count(menu, self.snapshot.cpu.halted);
        let current = match self.top_menu_focus {
            Some(TopMenuFocus::Item {
                menu: focused_menu,
                index,
            }) if focused_menu == menu && index < count => Some(index),
            _ => None,
        };
        let index = if direction > 0 {
            current.map_or(count - 1, |index| (index + count - 1) % count)
        } else {
            current.map_or(0, |index| (index + 1) % count)
        };
        self.set_top_menu_focus(
            TopMenuFocus::Item { menu, index },
            TopMenuIndicator::ArrowFill,
        );
    }

    fn move_top_menu_category(&mut self, direction: i32) {
        let Some(menu) = self.top_menu_focus.map(|focus| match focus {
            TopMenuFocus::Category(menu) | TopMenuFocus::Item { menu, .. } => menu,
        }) else {
            return;
        };
        let menu = adjacent_dropdown_menu(menu, direction < 0);
        self.set_top_menu_focus(TopMenuFocus::Category(menu), TopMenuIndicator::Hidden);
    }

    fn cycle_top_menu_focus(&mut self, backward: bool) {
        let current = match self.top_menu_focus {
            Some(TopMenuFocus::Category(menu)) => TopMenuFocus::Category(menu),
            Some(TopMenuFocus::Item { menu, index })
                if index < top_menu_item_count(menu, self.snapshot.cpu.halted) =>
            {
                TopMenuFocus::Item { menu, index }
            }
            _ => return,
        };
        let next = match current {
            TopMenuFocus::Category(menu) if backward => {
                let menu = adjacent_top_menu(menu, true);
                match top_menu_item_count(menu, self.snapshot.cpu.halted) {
                    0 => TopMenuFocus::Category(menu),
                    count => TopMenuFocus::Item {
                        menu,
                        index: count - 1,
                    },
                }
            }
            TopMenuFocus::Category(menu) => {
                if top_menu_item_count(menu, self.snapshot.cpu.halted) == 0 {
                    TopMenuFocus::Category(adjacent_top_menu(menu, false))
                } else {
                    TopMenuFocus::Item { menu, index: 0 }
                }
            }
            TopMenuFocus::Item { menu, index } if backward && index == 0 => {
                TopMenuFocus::Category(menu)
            }
            TopMenuFocus::Item { menu, index } if backward => TopMenuFocus::Item {
                menu,
                index: index - 1,
            },
            TopMenuFocus::Item { menu, index }
                if index + 1 < top_menu_item_count(menu, self.snapshot.cpu.halted) =>
            {
                TopMenuFocus::Item {
                    menu,
                    index: index + 1,
                }
            }
            TopMenuFocus::Item { menu, .. } => {
                TopMenuFocus::Category(adjacent_top_menu(menu, false))
            }
        };
        self.set_top_menu_focus(next, TopMenuIndicator::TabRing);
    }

    fn set_top_menu_focus(&mut self, focus: TopMenuFocus, indicator: TopMenuIndicator) {
        let menu = match focus {
            TopMenuFocus::Category(menu) | TopMenuFocus::Item { menu, .. } => menu,
        };
        self.open_menu = (menu != MenuId::Settings).then_some(menu);
        self.top_menu_focus = Some(focus);
        self.top_menu_indicator = indicator;
    }

    fn activate_top_menu_focus(&mut self) -> Task<Message> {
        let (menu, index) = match self.top_menu_focus {
            Some(TopMenuFocus::Category(MenuId::Settings)) => {
                self.close_top_menu();
                return Task::done(Message::OpenSettings);
            }
            Some(TopMenuFocus::Item { menu, index }) => (menu, index),
            _ => return Task::none(),
        };
        if index >= top_menu_item_count(menu, self.snapshot.cpu.halted) {
            return Task::none();
        }
        let action = top_menu_action(menu, index).expect("focused menu item must have an action");
        self.close_top_menu();
        Task::done(action)
    }
}

fn adjacent_top_menu(menu: MenuId, backward: bool) -> MenuId {
    let index = TOP_MENUS
        .iter()
        .position(|candidate| *candidate == menu)
        .unwrap_or(0);
    let next = if backward {
        (index + TOP_MENUS.len() - 1) % TOP_MENUS.len()
    } else {
        (index + 1) % TOP_MENUS.len()
    };
    TOP_MENUS[next]
}

fn adjacent_dropdown_menu(menu: MenuId, backward: bool) -> MenuId {
    let menu = adjacent_top_menu(menu, backward);
    if menu == MenuId::Settings {
        adjacent_top_menu(menu, backward)
    } else {
        menu
    }
}

fn top_menu_item_count(menu: MenuId, halted: bool) -> usize {
    let count = top_menu_actions(menu).len();
    if menu == MenuId::Mp && !halted {
        count - 1
    } else {
        count
    }
}

fn top_menu_actions(menu: MenuId) -> &'static [Message] {
    match menu {
        MenuId::File => FILE_MENU_ACTIONS,
        MenuId::Mp => MP_MENU_ACTIONS,
        MenuId::View => VIEW_MENU_ACTIONS,
        MenuId::Settings => &[],
        MenuId::Help => HELP_MENU_ACTIONS,
    }
}

pub(crate) fn top_menu_action(menu: MenuId, index: usize) -> Option<Message> {
    top_menu_actions(menu).get(index).cloned()
}

#[cfg(test)]
#[path = "menu_keyboard/tests.rs"]
mod tests;
