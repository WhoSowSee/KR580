use crate::devices::printer::PrinterPropertyChange;
use roxmltree::Document;
use std::fmt::Write;

const PSF_NAMESPACE: &str =
    "http://schemas.microsoft.com/windows/2003/08/printing/printschemaframework";
const XSI_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema-instance";
const XSD_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema";

pub(super) fn build(base_ticket: &[u8], change: &PrinterPropertyChange) -> Result<String, String> {
    let base = std::str::from_utf8(base_ticket)
        .map_err(|error| format!("PrintTicket is not UTF-8: {error}"))?;
    let document =
        Document::parse(base).map_err(|error| format!("PrintTicket XML is invalid: {error}"))?;
    let root = document.root_element();
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    xml.push_str("<psf:PrintTicket");
    let mut has_psf = false;
    let mut has_xsi = false;
    let mut has_xsd = false;
    for namespace in root.namespaces() {
        let prefix = namespace.name().unwrap_or_default();
        has_psf |= prefix == "psf";
        has_xsi |= prefix == "xsi";
        has_xsd |= prefix == "xsd";
        if prefix.is_empty() {
            write!(xml, " xmlns=\"{}\"", escape(namespace.uri())).unwrap();
        } else {
            write!(
                xml,
                " xmlns:{}=\"{}\"",
                escape(prefix),
                escape(namespace.uri())
            )
            .unwrap();
        }
    }
    append_namespace(&mut xml, "psf", PSF_NAMESPACE, has_psf);
    append_namespace(&mut xml, "xsi", XSI_NAMESPACE, has_xsi);
    append_namespace(&mut xml, "xsd", XSD_NAMESPACE, has_xsd);
    xml.push_str(" version=\"1\">");
    match change {
        PrinterPropertyChange::Feature {
            feature_name,
            option_name,
        } => {
            write!(
                xml,
                "<psf:Feature name=\"{}\"><psf:Option name=\"{}\"/></psf:Feature>",
                escape(feature_name),
                escape(option_name)
            )
            .unwrap();
        }
        PrinterPropertyChange::Parameter {
            parameter_name,
            value_type,
            value,
        } => {
            write!(
                xml,
                "<psf:ParameterInit name=\"{}\"><psf:Value xsi:type=\"{}\">{}</psf:Value></psf:ParameterInit>",
                escape(parameter_name),
                escape(value_type),
                escape(value)
            )
            .unwrap();
        }
    }
    xml.push_str("</psf:PrintTicket>");
    Ok(xml)
}

fn append_namespace(xml: &mut String, prefix: &str, uri: &str, present: bool) {
    if !present {
        write!(xml, " xmlns:{prefix}=\"{uri}\"").unwrap();
    }
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
