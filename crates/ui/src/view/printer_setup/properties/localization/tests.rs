use super::{feature_label, has_cyrillic, localized_options, parameter_label, parameter_visible};
use crate::i18n::Lang;
use k580_ui::devices::printer::{
    PrinterFeature, PrinterFeatureGroup, PrinterFeatureOption, PrinterParameter,
};

#[test]
fn localizes_hp_feature_and_option_names() {
    let feature = PrinterFeature {
        name: "psk:DocumentBlackOptimization".to_owned(),
        display_name: "DocumentBlackOptimization".to_owned(),
        group: PrinterFeatureGroup::Graphics,
        options: vec![PrinterFeatureOption {
            name: "psk:FitToPage".to_owned(),
            display_name: "FitToPage".to_owned(),
            constrained: false,
        }],
        selected_option: Some("psk:FitToPage".to_owned()),
    };

    assert_eq!(feature_label(&feature, Lang::Ru), "Оптимизация чёрного");
    assert_eq!(
        localized_options(&feature, Lang::Ru)[0].display_name,
        "По размеру страницы"
    );
}

#[test]
fn localizes_parameters_and_hides_driver_snapshots() {
    let angle = parameter("psk:PageWatermarkTextAngle", "PageWatermarkTextAngle");
    let snapshot = parameter("psk:PageDevmodeSnapshot", "PageDevmodeSnapshot");

    assert_eq!(parameter_label(&angle, Lang::Ru), "Угол водяного знака");
    assert!(parameter_visible(&angle));
    assert!(!parameter_visible(&snapshot));
}

#[test]
fn replaces_russian_driver_labels_in_english_ui() {
    let altitude = feature(
        "ns:JobHighAltitude",
        "Поправка на высоту",
        "ns:k1",
        "Стандартный",
    );
    assert_eq!(feature_label(&altitude, Lang::En), "Altitude correction");
    assert_eq!(
        localized_options(&altitude, Lang::En)[0].display_name,
        "Standard"
    );

    let source = feature(
        "psk:PageDefaultSource",
        "Источник бумаги",
        "vendor:k2",
        "Автовыбор",
    );
    assert_eq!(feature_label(&source, Lang::En), "Paper source");
    assert_eq!(
        localized_options(&source, Lang::En)[0].display_name,
        "Auto select"
    );
}

#[test]
fn unknown_russian_driver_labels_fall_back_to_qnames() {
    let custom = feature(
        "vendor:CustomPrintMode",
        "Особый режим",
        "vendor:k42",
        "Неизвестный режим",
    );
    let parameter = parameter("vendor:CustomPrintLevel", "Особый уровень");
    let labels = [
        feature_label(&custom, Lang::En),
        localized_options(&custom, Lang::En)[0].display_name.clone(),
        parameter_label(&parameter, Lang::En),
    ];

    assert_eq!(
        labels,
        ["Custom Print Mode", "Option 42", "Custom Print Level"]
    );
    assert!(labels.iter().all(|label| !has_cyrillic(label)));
}

#[test]
fn localizes_installed_hp_media_and_resolution_options() {
    let cases = [
        (
            "PageMediaSize",
            "ISOC5Envelope",
            "Конверт С5",
            "C5 envelope",
        ),
        ("PageDefaultSource", "AUTO", "Автовыбор", "Auto select"),
        ("PageMediaType", "OFF", "Не указано", "Unspecified"),
        ("PageMediaType", "NORMAL", "обычная", "Plain paper"),
        (
            "PageMediaType",
            "CARD",
            "Сверхплотная 121-163 г",
            "Cardstock (121–163 g/m²)",
        ),
        (
            "PageResolution",
            "0_1200x1200_dpi",
            "Высокое разрешение",
            "High resolution",
        ),
    ];

    for (feature_name, option_name, driver_label, expected) in cases {
        let feature = feature(
            &format!("psk:{feature_name}"),
            feature_name,
            &format!("vendor:{option_name}"),
            driver_label,
        );
        assert_eq!(
            localized_options(&feature, Lang::En)[0].display_name,
            expected
        );
    }
}

fn feature(
    name: &str,
    display_name: &str,
    option_name: &str,
    option_label: &str,
) -> PrinterFeature {
    PrinterFeature {
        group: PrinterFeatureGroup::General,
        name: name.to_owned(),
        display_name: display_name.to_owned(),
        selected_option: Some(option_name.to_owned()),
        options: vec![PrinterFeatureOption {
            name: option_name.to_owned(),
            display_name: option_label.to_owned(),
            constrained: false,
        }],
    }
}

fn parameter(name: &str, display_name: &str) -> PrinterParameter {
    PrinterParameter {
        name: name.to_owned(),
        display_name: display_name.to_owned(),
        value_type: "xsd:string".to_owned(),
        value: String::new(),
        minimum: None,
        maximum: None,
    }
}
