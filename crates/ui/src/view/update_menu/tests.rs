use super::update_menu_item;
use crate::app::updates::UpdatePhase;
use crate::app::{Message, TopMenuIndicator};
use crate::i18n::Lang;
use crate::runtime::updates::{Asset, PreparedUpdate, Release};
use iced::advanced::renderer::{self, Headless, Renderer as _};
use iced::advanced::{Layout, Shell, clipboard, layout, widget};
use iced::{Color, Event, Point, Rectangle, Size, mouse};

#[test]
fn update_row_keeps_dimensions_visible_icons_and_expected_click_messages() {
    let mut renderer = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            13.0.into(),
            Some("tiny-skia"),
        ))
        .unwrap();
    let release = Release {
        version: "4.1.0".into(),
        page: "https://github.com/WhoSowSee/KR580/releases/tag/v4.1.0".into(),
        asset: Some(Asset {
            name: "fixture.zip".into(),
            url: String::new(),
            size: 100,
            digest: None,
        }),
    };
    let phases = [
        UpdatePhase::Idle,
        UpdatePhase::Checking(std::time::Instant::now()),
        UpdatePhase::Available(release.clone()),
        UpdatePhase::Downloading {
            release,
            progress: Default::default(),
            started: std::time::Instant::now(),
        },
        UpdatePhase::Ready(PreparedUpdate {
            version: "4.1.0".into(),
            restart_flag: "restart.json".into(),
        }),
        UpdatePhase::Current(std::time::Instant::now()),
        UpdatePhase::Failed {
            error: "fixture error".into(),
            release: None,
        },
    ];
    for lang in [Lang::Ru, Lang::En] {
        for phase in &phases {
            let mut item = update_menu_item(phase, lang, TopMenuIndicator::Hidden);
            let mut tree = widget::Tree::new(&item);
            let node = item.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(252.0, 100.0)),
            );
            assert_eq!(node.size(), Size::new(252.0, 30.0));
            let drawing = node.clone().move_to(Point::new(30.0, 40.0));
            let bounds = Rectangle::with_size(Size::new(284.0, 74.0));
            for scale in [1.0, 1.25, 2.0] {
                renderer.reset(bounds);
                item.as_widget().draw(
                    &tree,
                    &mut renderer,
                    &iced::Theme::Dark,
                    &renderer::Style::default(),
                    Layout::new(&drawing),
                    mouse::Cursor::Unavailable,
                    &bounds,
                );
                let size = Size::new(
                    (bounds.width * scale) as u32,
                    (bounds.height * scale) as u32,
                );
                let pixels = renderer.screenshot(size, scale, Color::BLACK);
                let visible = ((48.0 * scale) as u32..(61.0 * scale) as u32)
                    .flat_map(|y| {
                        ((41.0 * scale) as u32..(55.0 * scale) as u32).map(move |x| (x, y))
                    })
                    .filter(|&(x, y)| {
                        let offset = ((y * size.width + x) * 4) as usize;
                        pixels[offset..offset + 3].iter().any(|value| *value > 15)
                    })
                    .count();
                assert!(
                    visible > 8,
                    "missing update icon: {phase:?}, scale={scale}, pixels={visible}"
                );
            }
            let mut messages = Vec::new();
            for event in [
                mouse::Event::ButtonPressed(mouse::Button::Left),
                mouse::Event::ButtonReleased(mouse::Button::Left),
            ] {
                item.as_widget_mut().update(
                    &mut tree,
                    &Event::Mouse(event),
                    Layout::new(&node),
                    mouse::Cursor::Available(Point::new(100.0, 15.0)),
                    &renderer,
                    &mut clipboard::Null,
                    &mut Shell::new(&mut messages),
                    &Layout::new(&node).bounds(),
                );
            }
            if phase.busy() || matches!(phase, UpdatePhase::Current(_)) {
                assert!(messages.is_empty());
            } else {
                assert!(matches!(messages.as_slice(), [Message::UpdateActivated]));
            }
        }
    }
}
