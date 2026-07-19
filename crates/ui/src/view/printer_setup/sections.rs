use super::dropdown::{self, DropdownControl, DropdownItem};
use super::labels::{Label, label, localized_status};
use super::styles::{footer_button, group_style, radio_style};
use crate::app::{Message, PrinterSetupDialog, PrinterSetupDropdown, PrinterSetupFocus};
use crate::i18n::Lang;
use crate::view::styles::legend_label_style;
use crate::view::theme::{tokyo_border, tokyo_muted, tokyo_text, ui_text};
use iced::widget::{Space, column, container, radio, row, stack};
use iced::{Alignment, Border, Element, Length, Padding, alignment};
use k580_ui::devices::printer::{PrinterOrientation, PrinterPaper, PrinterSource};

pub(super) fn printer_section<'a>(
    dialog: &'a PrinterSetupDialog,
    lang: Lang,
) -> Element<'a, Message> {
    let selected_name = dialog.selected_name.as_deref();
    let options = dialog
        .printers
        .iter()
        .map(|printer| DropdownItem {
            selected: selected_name == Some(printer.name.as_str()),
            label: printer.name.clone(),
            message: Message::PrinterSetupSelected(printer.name.clone()),
        })
        .collect::<Vec<_>>();
    let selector = dropdown::control(
        selected_name.map(str::to_owned).unwrap_or_else(|| {
            label(
                lang,
                if dialog.loading {
                    Label::Loading
                } else {
                    Label::SelectPrinter
                },
            )
            .to_owned()
        }),
        options,
        DropdownControl {
            opened: dialog.open_dropdown == Some(PrinterSetupDropdown::Printer),
            enabled: !dialog.loading,
            focused: dialog.focus == PrinterSetupFocus::Printer,
            toggle: Message::PrinterSetupDropdownToggled(PrinterSetupDropdown::Printer),
            dismiss: Message::PrinterSetupDropdownDismissed(PrinterSetupDropdown::Printer),
            highlighted: dialog.dropdown_highlight,
        },
    );
    let properties_ready = dialog.configuration.is_some()
        && !dialog.configuration_loading
        && !dialog.properties_pending;
    let properties = footer_button(
        label(lang, Label::Properties),
        properties_ready,
        dialog.focus == PrinterSetupFocus::Properties,
    )
    .width(Length::Fixed(132.0))
    .on_press_maybe(properties_ready.then_some(Message::PrinterSetupProperties));
    let name_row = row![field_label(label(lang, Label::Name)), selector, properties]
        .spacing(10)
        .align_y(Alignment::Center);

    let details = match dialog.selected_printer() {
        Some(printer) => {
            let place = if printer.location.trim().is_empty() {
                &printer.port
            } else {
                &printer.location
            };
            column![
                detail_row(
                    label(lang, Label::Status),
                    localized_status(&printer.status, lang),
                ),
                detail_row(label(lang, Label::Type), &printer.driver),
                detail_row(label(lang, Label::Place), place),
                detail_row(label(lang, Label::Comment), &printer.comment),
            ]
            .spacing(7)
        }
        None => column![detail_row(
            label(lang, Label::Status),
            label(lang, Label::NoSelection),
        )],
    };
    column![name_row, details].spacing(12).into()
}

pub(super) fn paper_section<'a>(
    dialog: &'a PrinterSetupDialog,
    lang: Lang,
) -> Element<'a, Message> {
    let Some(configuration) = dialog.configuration.as_ref() else {
        return loading_content(dialog, lang);
    };
    let paper = configuration.selected_paper().cloned();
    let source = configuration.selected_source().cloned();
    column![
        row![
            field_label(label(lang, Label::Size)),
            paper_dropdown(dialog, configuration.papers.clone(), paper),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        row![
            field_label(label(lang, Label::Source)),
            source_dropdown(dialog, configuration.sources.clone(), source),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    ]
    .spacing(14)
    .into()
}

pub(super) fn orientation_section<'a>(
    dialog: &'a PrinterSetupDialog,
    lang: Lang,
) -> Element<'a, Message> {
    let Some(configuration) = dialog.configuration.as_ref() else {
        return loading_content(dialog, lang);
    };
    let selected = Some(configuration.settings.orientation);
    let controls = column![
        radio(
            label(lang, Label::Portrait),
            PrinterOrientation::Portrait,
            selected,
            Message::PrinterSetupOrientationSelected,
        )
        .size(18)
        .spacing(8)
        .text_size(13)
        .style(radio_style(dialog.focus == PrinterSetupFocus::Portrait,)),
        radio(
            label(lang, Label::Landscape),
            PrinterOrientation::Landscape,
            selected,
            Message::PrinterSetupOrientationSelected,
        )
        .size(18)
        .spacing(8)
        .text_size(13)
        .style(radio_style(dialog.focus == PrinterSetupFocus::Landscape,)),
    ]
    .spacing(12);
    container(
        row![paper_icon(configuration.settings.orientation), controls]
            .spacing(22)
            .align_y(Alignment::Center),
    )
    .padding(Padding {
        top: 0.0,
        right: 0.0,
        bottom: 8.0,
        left: 0.0,
    })
    .height(Length::Fill)
    .align_y(alignment::Vertical::Center)
    .into()
}

pub(super) fn group_box<'a>(
    title: &'static str,
    content: Element<'a, Message>,
    height: Length,
) -> container::Container<'a, Message> {
    let panel = container(content)
        .padding(Padding {
            top: 20.0,
            right: 14.0,
            bottom: 12.0,
            left: 14.0,
        })
        .width(Length::Fill)
        .height(height)
        .style(group_style);
    let legend = row![
        Space::new().width(Length::Fixed(12.0)),
        container(ui_text(title, 13, tokyo_text()))
            .padding([0, 6])
            .style(legend_label_style),
        Space::new().width(Length::Fill),
    ];
    container(
        stack![
            column![Space::new().height(Length::Fixed(9.0)), panel].height(height),
            legend,
        ]
        .width(Length::Fill)
        .height(height),
    )
    .height(height)
}

fn paper_dropdown(
    dialog: &PrinterSetupDialog,
    options: Vec<PrinterPaper>,
    selected: Option<PrinterPaper>,
) -> Element<'static, Message> {
    let selected_id = selected.as_ref().map(|paper| paper.id);
    let label = selected.map_or_else(|| "—".to_owned(), |paper| paper.to_string());
    let items = options
        .into_iter()
        .map(|paper| DropdownItem {
            selected: Some(paper.id) == selected_id,
            label: paper.to_string(),
            message: Message::PrinterSetupPaperSelected(paper.id),
        })
        .collect();
    dropdown::control(
        label,
        items,
        DropdownControl {
            opened: dialog.open_dropdown == Some(PrinterSetupDropdown::Paper),
            enabled: !dialog.configuration_loading,
            focused: dialog.focus == PrinterSetupFocus::Paper,
            toggle: Message::PrinterSetupDropdownToggled(PrinterSetupDropdown::Paper),
            dismiss: Message::PrinterSetupDropdownDismissed(PrinterSetupDropdown::Paper),
            highlighted: dialog.dropdown_highlight,
        },
    )
}

fn source_dropdown(
    dialog: &PrinterSetupDialog,
    options: Vec<PrinterSource>,
    selected: Option<PrinterSource>,
) -> Element<'static, Message> {
    let selected_id = selected.as_ref().map(|source| source.id);
    let label = selected.map_or_else(|| "—".to_owned(), |source| source.to_string());
    let items = options
        .into_iter()
        .map(|source| DropdownItem {
            selected: Some(source.id) == selected_id,
            label: source.to_string(),
            message: Message::PrinterSetupSourceSelected(source.id),
        })
        .collect();
    dropdown::control(
        label,
        items,
        DropdownControl {
            opened: dialog.open_dropdown == Some(PrinterSetupDropdown::Source),
            enabled: !dialog.configuration_loading,
            focused: dialog.focus == PrinterSetupFocus::Source,
            toggle: Message::PrinterSetupDropdownToggled(PrinterSetupDropdown::Source),
            dismiss: Message::PrinterSetupDropdownDismissed(PrinterSetupDropdown::Source),
            highlighted: dialog.dropdown_highlight,
        },
    )
}

fn loading_content<'a>(dialog: &PrinterSetupDialog, lang: Lang) -> Element<'a, Message> {
    container(ui_text(
        label(
            lang,
            if dialog.configuration_loading {
                Label::LoadingSettings
            } else {
                Label::NoSelection
            },
        ),
        12,
        tokyo_muted(),
    ))
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(alignment::Horizontal::Center)
    .align_y(alignment::Vertical::Center)
    .into()
}

fn detail_row<'a>(label: &'static str, value: &'a str) -> Element<'a, Message> {
    let value = if value.trim().is_empty() {
        "—"
    } else {
        value
    };
    row![
        ui_text(label, 12, tokyo_muted()).width(Length::Fixed(150.0)),
        ui_text(value.to_owned(), 13, tokyo_text()),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn field_label(label: &'static str) -> Element<'static, Message> {
    ui_text(label, 13, tokyo_text())
        .width(Length::Fixed(78.0))
        .into()
}

fn paper_icon(orientation: PrinterOrientation) -> Element<'static, Message> {
    let (width, height) = match orientation {
        PrinterOrientation::Portrait => (54.0, 68.0),
        PrinterOrientation::Landscape => (68.0, 54.0),
    };
    let paper = container(ui_text("A", 28, tokyo_text()))
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .style(|_theme| container::Style {
            border: Border {
                radius: 2.0.into(),
                width: 1.0,
                color: tokyo_border(),
            },
            ..container::Style::default()
        });
    container(paper)
        .width(Length::Fixed(72.0))
        .height(Length::Fixed(72.0))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .into()
}
