use super::PrinterConfiguration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrinterFeatureGroup {
    General,
    Paper,
    Graphics,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrinterFeatureOption {
    pub name: String,
    pub display_name: String,
    pub constrained: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrinterFeature {
    pub name: String,
    pub display_name: String,
    pub group: PrinterFeatureGroup,
    pub options: Vec<PrinterFeatureOption>,
    pub selected_option: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrinterParameter {
    pub name: String,
    pub display_name: String,
    pub value_type: String,
    pub value: String,
    pub minimum: Option<i64>,
    pub maximum: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrinterPropertySheet {
    pub configuration: PrinterConfiguration,
    pub features: Vec<PrinterFeature>,
    pub parameters: Vec<PrinterParameter>,
    pub provider_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrinterPropertyChange {
    Feature {
        feature_name: String,
        option_name: String,
    },
    Parameter {
        parameter_name: String,
        value_type: String,
        value: String,
    },
}
