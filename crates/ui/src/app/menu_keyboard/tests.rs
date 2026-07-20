use super::super::messages::{MenuId, Message, TopMenuFocus, TopMenuIndicator};
use super::super::state::DesktopApp;

fn open_menu(app: &mut DesktopApp, menu: MenuId) {
    let _ = app.update(Message::MenuToggled(menu));
}

fn press(app: &mut DesktopApp, message: Message) {
    let _ = app.update(message);
}

#[test]
fn pointer_open_starts_on_category_without_keyboard_ring() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);

    open_menu(&mut app, MenuId::File);

    assert_eq!(app.open_menu, Some(MenuId::File));
    assert_eq!(
        app.top_menu_focus,
        Some(TopMenuFocus::Category(MenuId::File))
    );
    assert_eq!(app.top_menu_indicator, TopMenuIndicator::Hidden);
}

#[test]
fn vertical_arrows_stay_in_open_menu_and_horizontal_arrows_switch_categories() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    app.memory_address_input = "0010".to_owned();
    open_menu(&mut app, MenuId::File);

    press(&mut app, Message::ArrowKey(-1));
    assert_eq!(
        app.top_menu_focus,
        Some(TopMenuFocus::Item {
            menu: MenuId::File,
            index: 0,
        })
    );
    assert_eq!(app.top_menu_indicator, TopMenuIndicator::ArrowFill);
    assert_eq!(app.memory_address_input, "0010");

    press(&mut app, Message::ArrowKey(-1));
    press(&mut app, Message::HorizontalArrowKey(1));
    assert_eq!(app.open_menu, Some(MenuId::Mp));
    assert_eq!(app.top_menu_focus, Some(TopMenuFocus::Category(MenuId::Mp)));
    assert_eq!(app.top_menu_indicator, TopMenuIndicator::Hidden);
    assert_eq!(app.memory_address_input, "0010");

    press(&mut app, Message::HorizontalArrowKey(-1));
    assert_eq!(app.open_menu, Some(MenuId::File));
    assert_eq!(
        app.top_menu_focus,
        Some(TopMenuFocus::Category(MenuId::File))
    );
    assert_eq!(app.top_menu_indicator, TopMenuIndicator::Hidden);

    press(&mut app, Message::ArrowKey(-1));
    assert_eq!(
        app.top_menu_focus,
        Some(TopMenuFocus::Item {
            menu: MenuId::File,
            index: 0,
        })
    );
}

#[test]
fn tab_walks_items_then_opens_the_next_menu_on_its_category() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    open_menu(&mut app, MenuId::File);

    for _ in 0..6 {
        press(&mut app, Message::FocusCycle { backward: false });
    }
    assert_eq!(
        app.top_menu_focus,
        Some(TopMenuFocus::Item {
            menu: MenuId::File,
            index: 5,
        })
    );

    press(&mut app, Message::FocusCycle { backward: false });
    assert_eq!(app.open_menu, Some(MenuId::Mp));
    assert_eq!(app.top_menu_focus, Some(TopMenuFocus::Category(MenuId::Mp)));
    assert_eq!(app.top_menu_indicator, TopMenuIndicator::TabRing);

    press(&mut app, Message::FocusCycle { backward: false });
    assert_eq!(
        app.top_menu_focus,
        Some(TopMenuFocus::Item {
            menu: MenuId::Mp,
            index: 0,
        })
    );
}

#[test]
fn shift_tab_reaches_the_previous_menu_last_item() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    open_menu(&mut app, MenuId::Mp);

    press(&mut app, Message::FocusCycle { backward: true });

    assert_eq!(app.open_menu, Some(MenuId::File));
    assert_eq!(
        app.top_menu_focus,
        Some(TopMenuFocus::Item {
            menu: MenuId::File,
            index: 5,
        })
    );
    assert_eq!(app.top_menu_indicator, TopMenuIndicator::TabRing);
}

#[test]
fn tab_skips_disabled_clear_halt_item() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    assert!(!app.snapshot.cpu.halted);
    open_menu(&mut app, MenuId::Mp);

    for _ in 0..5 {
        press(&mut app, Message::FocusCycle { backward: false });
    }
    assert_eq!(
        app.top_menu_focus,
        Some(TopMenuFocus::Item {
            menu: MenuId::Mp,
            index: 4,
        })
    );

    press(&mut app, Message::FocusCycle { backward: false });
    assert_eq!(app.open_menu, Some(MenuId::View));
    assert_eq!(
        app.top_menu_focus,
        Some(TopMenuFocus::Category(MenuId::View))
    );
}

#[test]
fn settings_is_a_tab_stop_without_a_dropdown() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    open_menu(&mut app, MenuId::View);

    for _ in 0..7 {
        press(&mut app, Message::FocusCycle { backward: false });
    }
    assert_eq!(app.open_menu, None);
    assert_eq!(
        app.top_menu_focus,
        Some(TopMenuFocus::Category(MenuId::Settings))
    );
    assert_eq!(app.top_menu_indicator, TopMenuIndicator::TabRing);

    press(&mut app, Message::FocusCycle { backward: false });
    assert_eq!(app.open_menu, Some(MenuId::Help));
    assert_eq!(
        app.top_menu_focus,
        Some(TopMenuFocus::Category(MenuId::Help))
    );

    press(&mut app, Message::FocusCycle { backward: true });
    assert_eq!(app.open_menu, None);
    assert_eq!(
        app.top_menu_focus,
        Some(TopMenuFocus::Category(MenuId::Settings))
    );

    press(&mut app, Message::FocusCycle { backward: true });
    assert_eq!(app.open_menu, Some(MenuId::View));
    assert_eq!(
        app.top_menu_focus,
        Some(TopMenuFocus::Item {
            menu: MenuId::View,
            index: 5,
        })
    );
}

#[test]
fn horizontal_arrows_skip_settings() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    open_menu(&mut app, MenuId::View);

    press(&mut app, Message::HorizontalArrowKey(1));
    assert_eq!(app.open_menu, Some(MenuId::Help));
    assert_eq!(
        app.top_menu_focus,
        Some(TopMenuFocus::Category(MenuId::Help))
    );
    assert_eq!(app.top_menu_indicator, TopMenuIndicator::Hidden);

    press(&mut app, Message::HorizontalArrowKey(-1));
    assert_eq!(app.open_menu, Some(MenuId::View));
    assert_eq!(
        app.top_menu_focus,
        Some(TopMenuFocus::Category(MenuId::View))
    );
    assert_eq!(app.top_menu_indicator, TopMenuIndicator::Hidden);
}

#[test]
fn enter_closes_menu_after_activating_focused_item() {
    let (mut app, _task) = DesktopApp::with_initial_path(None);
    open_menu(&mut app, MenuId::File);
    press(&mut app, Message::ArrowKey(-1));

    press(&mut app, Message::EnterPressed);

    assert_eq!(app.open_menu, None);
    assert_eq!(app.top_menu_focus, None);
    assert_eq!(app.top_menu_indicator, TopMenuIndicator::Hidden);
}
