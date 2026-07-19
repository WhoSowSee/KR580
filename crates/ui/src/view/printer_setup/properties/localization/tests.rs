use super::{feature_label, localized_options, parameter_label, parameter_visible};
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
    let angle = parameter("psk:PageWatermarkTextAngle");
    let snapshot = parameter("psk:PageDevmodeSnapshot");

    assert_eq!(parameter_label(&angle, Lang::Ru), "Угол водяного знака");
    assert!(parameter_visible(&angle));
    assert!(!parameter_visible(&snapshot));
}

fn parameter(name: &str) -> PrinterParameter {
    PrinterParameter {
        name: name.to_owned(),
        display_name: name.to_owned(),
        value_type: "xsd:string".to_owned(),
        value: String::new(),
        minimum: None,
        maximum: None,
    }
}
