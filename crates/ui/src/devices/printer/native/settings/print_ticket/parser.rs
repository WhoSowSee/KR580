use crate::devices::printer::{
    PrinterFeature, PrinterFeatureGroup, PrinterFeatureOption, PrinterParameter,
};
use roxmltree::{Document, Node};
use std::collections::{HashMap, HashSet};

pub(super) fn parse(
    capabilities: &[u8],
    ticket: &[u8],
) -> Result<(Vec<PrinterFeature>, Vec<PrinterParameter>), String> {
    let capabilities = decode_xml(capabilities)?;
    let ticket = decode_xml(ticket)?;
    let capabilities = Document::parse(&capabilities)
        .map_err(|error| format!("PrintCapabilities XML is invalid: {error}"))?;
    let ticket =
        Document::parse(&ticket).map_err(|error| format!("PrintTicket XML is invalid: {error}"))?;
    let selected_features = selected_features(&ticket);
    let selected_parameters = selected_parameters(&ticket);
    let mut seen_features = HashSet::new();
    let features = capabilities
        .descendants()
        .filter(|node| is_element(*node, "Feature"))
        .filter_map(|node| parse_feature(node, &selected_features))
        .filter(|feature| seen_features.insert(feature.name.clone()))
        .collect();
    let parameters = capabilities
        .descendants()
        .filter(|node| is_element(*node, "ParameterDef"))
        .filter_map(|node| parse_parameter(node, &selected_parameters))
        .collect();
    Ok((features, parameters))
}

fn parse_feature(node: Node<'_, '_>, selected: &HashMap<String, String>) -> Option<PrinterFeature> {
    let name = node.attribute("name")?.to_owned();
    let options = node
        .children()
        .filter(|child| is_element(*child, "Option"))
        .filter_map(parse_option)
        .collect::<Vec<_>>();
    if options.is_empty() {
        return None;
    }
    let display_name = display_name(node).unwrap_or_else(|| readable_qname(&name));
    Some(PrinterFeature {
        group: classify_feature(&name),
        selected_option: selected.get(&name).cloned(),
        name,
        display_name,
        options,
    })
}

fn parse_option(node: Node<'_, '_>) -> Option<PrinterFeatureOption> {
    let name = node.attribute("name")?.to_owned();
    let display_name = display_name(node).unwrap_or_else(|| readable_option(&name));
    let constrained = property_value(node, "Constrained")
        .is_some_and(|value| !local_name(&value).eq_ignore_ascii_case("None"));
    Some(PrinterFeatureOption {
        name,
        display_name,
        constrained,
    })
}

fn parse_parameter(
    node: Node<'_, '_>,
    selected: &HashMap<String, String>,
) -> Option<PrinterParameter> {
    let name = node.attribute("name")?.to_owned();
    let value = selected
        .get(&name)
        .cloned()
        .or_else(|| property_value(node, "DefaultValue"))
        .unwrap_or_default();
    Some(PrinterParameter {
        display_name: display_name(node).unwrap_or_else(|| readable_qname(&name)),
        value_type: property_value(node, "DataType").unwrap_or_else(|| "xsd:string".to_owned()),
        value,
        minimum: property_value(node, "MinValue").and_then(|value| value.parse().ok()),
        maximum: property_value(node, "MaxValue").and_then(|value| value.parse().ok()),
        name,
    })
}

fn selected_features(document: &Document<'_>) -> HashMap<String, String> {
    document
        .descendants()
        .filter(|node| is_element(*node, "Feature"))
        .filter_map(|feature| {
            let name = feature.attribute("name")?;
            let option = feature
                .children()
                .find(|child| is_element(*child, "Option"))?
                .attribute("name")?;
            Some((name.to_owned(), option.to_owned()))
        })
        .collect()
}

fn selected_parameters(document: &Document<'_>) -> HashMap<String, String> {
    document
        .descendants()
        .filter(|node| is_element(*node, "ParameterInit"))
        .filter_map(|parameter| {
            let name = parameter.attribute("name")?;
            let value = parameter
                .descendants()
                .find(|child| is_element(*child, "Value"))?
                .text()?;
            Some((name.to_owned(), value.trim().to_owned()))
        })
        .collect()
}

fn display_name(node: Node<'_, '_>) -> Option<String> {
    property_value(node, "DisplayName").filter(|value| !value.trim().is_empty())
}

fn property_value(node: Node<'_, '_>, property_name: &str) -> Option<String> {
    node.children()
        .find(|child| {
            is_element(*child, "Property")
                && child
                    .attribute("name")
                    .is_some_and(|name| local_name(name) == property_name)
        })?
        .descendants()
        .find(|child| is_element(*child, "Value"))?
        .text()
        .map(|value| value.trim().to_owned())
}

fn classify_feature(name: &str) -> PrinterFeatureGroup {
    let name = local_name(name).to_ascii_lowercase();
    if [
        "media",
        "paper",
        "inputbin",
        "outputbin",
        "orientation",
        "duplex",
        "binding",
    ]
    .iter()
    .any(|needle| name.contains(needle))
    {
        PrinterFeatureGroup::Paper
    } else if [
        "resolution",
        "color",
        "toner",
        "darken",
        "black",
        "fineedge",
        "halftone",
        "quality",
        "scaling",
        "watermark",
        "overlay",
        "image",
        "graphic",
        "font",
    ]
    .iter()
    .any(|needle| name.contains(needle))
    {
        PrinterFeatureGroup::Graphics
    } else {
        PrinterFeatureGroup::General
    }
}

fn readable_option(name: &str) -> String {
    let local = local_name(name);
    if let Some(value) = local.strip_prefix('k')
        && value.chars().all(|character| character.is_ascii_digit())
    {
        return value.to_owned();
    }
    if let Some((width, height)) = local.split_once('x')
        && width.chars().all(|character| character.is_ascii_digit())
        && height
            .split('_')
            .next()
            .is_some_and(|value| value.chars().all(|character| character.is_ascii_digit()))
    {
        return local.replace('_', " ");
    }
    readable(local)
}

fn readable_qname(name: &str) -> String {
    readable(local_name(name))
}

fn readable(value: &str) -> String {
    let mut output = String::new();
    let mut previous_lower = false;
    for character in value.replace(['_', '-'], " ").chars() {
        if character.is_uppercase() && previous_lower {
            output.push(' ');
        }
        output.push(character);
        previous_lower = character.is_lowercase() || character.is_ascii_digit();
    }
    output
}

fn local_name(name: &str) -> &str {
    name.rsplit_once(':').map_or(name, |(_, local)| local)
}

fn is_element(node: Node<'_, '_>, name: &str) -> bool {
    node.is_element() && node.tag_name().name() == name
}

fn decode_xml(bytes: &[u8]) -> Result<String, String> {
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let words = bytes[2..]
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect::<Vec<_>>();
        return String::from_utf16(&words)
            .map_err(|error| format!("Print Schema XML is not UTF-16LE: {error}"));
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let words = bytes[2..]
            .chunks_exact(2)
            .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
            .collect::<Vec<_>>();
        return String::from_utf16(&words)
            .map_err(|error| format!("Print Schema XML is not UTF-16BE: {error}"));
    }
    std::str::from_utf8(bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes))
        .map(str::to_owned)
        .map_err(|error| format!("Print Schema XML is not UTF-8: {error}"))
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn parses_features_parameters_and_current_values() {
        let capabilities = br#"<psf:PrintCapabilities xmlns:psf="p" xmlns:psk="k" xmlns:xsd="x">
            <psf:Feature name="psk:PageResolution">
                <psf:Property name="psf:DisplayName"><psf:Value>Quality</psf:Value></psf:Property>
                <psf:Option name="psk:Draft"><psf:Property name="psf:DisplayName"><psf:Value>Draft</psf:Value></psf:Property></psf:Option>
                <psf:Option name="psk:High"/>
            </psf:Feature>
            <psf:ParameterDef name="psk:Copies">
                <psf:Property name="psf:DataType"><psf:Value>xsd:integer</psf:Value></psf:Property>
                <psf:Property name="psf:DefaultValue"><psf:Value>1</psf:Value></psf:Property>
                <psf:Property name="psf:MinValue"><psf:Value>1</psf:Value></psf:Property>
                <psf:Property name="psf:MaxValue"><psf:Value>99</psf:Value></psf:Property>
            </psf:ParameterDef>
        </psf:PrintCapabilities>"#;
        let ticket = br#"<psf:PrintTicket xmlns:psf="p" xmlns:psk="k" xmlns:xsd="x">
            <psf:Feature name="psk:PageResolution"><psf:Option name="psk:High"/></psf:Feature>
            <psf:ParameterInit name="psk:Copies"><psf:Value>3</psf:Value></psf:ParameterInit>
        </psf:PrintTicket>"#;
        let (features, parameters) = parse(capabilities, ticket).unwrap();
        assert_eq!(features[0].selected_option.as_deref(), Some("psk:High"));
        assert_eq!(features[0].options[0].display_name, "Draft");
        assert_eq!(parameters[0].value, "3");
        assert_eq!(parameters[0].maximum, Some(99));
    }
}
