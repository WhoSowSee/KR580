mod icon;
#[cfg(test)]
mod tests;

use iced::widget::{button, container, row, svg, tooltip};
use iced::{Element, Length, alignment};

use super::styles::{menu_button_style, status_tooltip_style};
use super::theme::{tokyo_text, ui_text};
use crate::app::updates::UpdatePhase;
use crate::app::{Message, TopMenuIndicator};
use crate::i18n::Lang;

pub(super) fn update_menu_item(
    phase: &UpdatePhase,
    lang: Lang,
    indicator: TopMenuIndicator,
) -> Element<'static, Message> {
    let color = tokyo_text();
    let glyph: Element<'static, Message> = match phase {
        UpdatePhase::Idle => svg(super::icons::refresh_ccw())
            .width(16)
            .height(16)
            .style(move |_, _| svg::Style { color: Some(color) })
            .into(),
        UpdatePhase::Available(_) => icon::symbol(icon::Kind::Download, color),
        UpdatePhase::Checking(started) => {
            icon::symbol(icon::Kind::Refresh(started.elapsed().as_secs_f32()), color)
        }
        UpdatePhase::Downloading {
            release,
            progress,
            started,
        } => {
            if progress.preparing() {
                icon::symbol(icon::Kind::Spinner(started.elapsed().as_secs_f32()), color)
            } else {
                icon::symbol(
                    icon::Kind::Progress(
                        progress.fraction(release.asset.as_ref().map_or(0, |asset| asset.size)),
                    ),
                    color,
                )
            }
        }
        UpdatePhase::Current(_) | UpdatePhase::Ready(_) => icon::symbol(icon::Kind::Check, color),
        UpdatePhase::Failed { .. } => icon::symbol(icon::Kind::Error, color),
    };
    let body = container(
        row![glyph, ui_text(lang.t(phase.label()), 13, color)]
            .spacing(10)
            .align_y(alignment::Vertical::Center),
    )
    .padding([6, 10])
    .width(Length::Fill)
    .height(Length::Fixed(30.0));
    let item = button(body)
        .padding(0)
        .width(Length::Fill)
        .on_press_maybe(
            (!phase.busy() && !matches!(phase, UpdatePhase::Current(_)))
                .then_some(Message::UpdateActivated),
        )
        .style(move |_, status| menu_button_style(status, indicator));
    let hint = match phase {
        UpdatePhase::Available(release) => Some(release.version.clone()),
        UpdatePhase::Ready(prepared) => Some(prepared.version.clone()),
        UpdatePhase::Downloading {
            release, progress, ..
        } => Some(format!(
            "{} · {:.0}%",
            release.version,
            progress.fraction(release.asset.as_ref().map_or(0, |asset| asset.size)) * 100.0
        )),
        UpdatePhase::Failed { error, .. } => Some(error.clone()),
        _ => None,
    };
    match hint {
        Some(hint) => tooltip(
            item,
            container(ui_text(hint, 12, color)).padding(8),
            tooltip::Position::Bottom,
        )
        .style(status_tooltip_style)
        .into(),
        None => item.into(),
    }
}
