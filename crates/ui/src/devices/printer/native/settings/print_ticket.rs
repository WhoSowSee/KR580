mod delta;
mod parser;

use super::super::wide_null;
use super::{configuration_from_devmode, print_devmode};
use crate::devices::printer::{
    PrinterInfo, PrinterPropertyChange, PrinterPropertySheet, PrinterSettings,
};
use windows::Win32::Graphics::Gdi::DEVMODEA;
use windows::Win32::Graphics::Printing::PrintTicket::{
    HPTPROVIDER, PTCloseProvider, PTConvertDevModeToPrintTicket, PTConvertPrintTicketToDevMode,
    PTGetPrintCapabilities, PTMergeAndValidatePrintTicket, PTOpenProvider, PTReleaseMemory,
    kPTJobScope, kUserDefaultDevmode,
};
use windows::Win32::System::Com::StructuredStorage::CreateStreamOnHGlobal;
use windows::Win32::System::Com::{
    COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize, IStream, STREAM_SEEK_END, STREAM_SEEK_SET,
};
use windows::core::{BSTR, PCWSTR};

pub(super) fn load(
    printer: &PrinterInfo,
    settings: &PrinterSettings,
) -> Result<PrinterPropertySheet, String> {
    let mut settings = settings.clone();
    settings.printer_name = printer.name.clone();
    let devmode = print_devmode(&settings)?.into_bytes();
    let configuration = configuration_from_devmode(printer, devmode.clone())?;
    match inspect(&printer.name, &devmode) {
        Ok((features, parameters)) => Ok(PrinterPropertySheet {
            configuration,
            features,
            parameters,
            provider_error: None,
        }),
        Err(error) => Ok(PrinterPropertySheet {
            configuration,
            features: Vec::new(),
            parameters: Vec::new(),
            provider_error: Some(error),
        }),
    }
}

pub(super) fn apply(
    printer: &PrinterInfo,
    settings: &PrinterSettings,
    change: &PrinterPropertyChange,
) -> Result<PrinterPropertySheet, String> {
    let _com = ComApartment::init()?;
    let provider = Provider::open(&printer.name)?;
    let mut settings = settings.clone();
    settings.printer_name = printer.name.clone();
    let base_devmode = print_devmode(&settings)?.into_bytes();
    let base_ticket = ticket_from_devmode(&provider, &base_devmode)?;
    let base_xml = stream_to_bytes(&base_ticket)?;
    let delta_xml = delta::build(&base_xml, change)?;
    let delta_ticket = stream_from_bytes(delta_xml.as_bytes())?;
    let result_ticket = empty_stream()?;
    let mut driver_error = BSTR::new();
    unsafe {
        PTMergeAndValidatePrintTicket(
            provider.0,
            &base_ticket,
            &delta_ticket,
            kPTJobScope,
            &result_ticket,
            Some(&mut driver_error),
        )
    }
    .map_err(|error| api_error("PTMergeAndValidatePrintTicket", error, &driver_error))?;
    let devmode = devmode_from_ticket(&provider, &result_ticket)?;
    let configuration = configuration_from_devmode(printer, devmode)?;
    let (features, parameters) = capabilities_from_ticket(&provider, &result_ticket)?;
    Ok(PrinterPropertySheet {
        configuration,
        features,
        parameters,
        provider_error: None,
    })
}

fn inspect(
    printer_name: &str,
    devmode: &[u8],
) -> Result<
    (
        Vec<crate::devices::printer::PrinterFeature>,
        Vec<crate::devices::printer::PrinterParameter>,
    ),
    String,
> {
    let _com = ComApartment::init()?;
    let provider = Provider::open(printer_name)?;
    let ticket = ticket_from_devmode(&provider, devmode)?;
    capabilities_from_ticket(&provider, &ticket)
}

fn ticket_from_devmode(provider: &Provider, devmode: &[u8]) -> Result<IStream, String> {
    let ticket = empty_stream()?;
    unsafe {
        PTConvertDevModeToPrintTicket(
            provider.0,
            devmode.len() as u32,
            devmode.as_ptr().cast::<DEVMODEA>(),
            kPTJobScope,
            &ticket,
        )
    }
    .map_err(|error| format!("PTConvertDevModeToPrintTicket failed: {error}"))?;
    rewind(&ticket)?;
    Ok(ticket)
}

fn capabilities_from_ticket(
    provider: &Provider,
    ticket: &IStream,
) -> Result<
    (
        Vec<crate::devices::printer::PrinterFeature>,
        Vec<crate::devices::printer::PrinterParameter>,
    ),
    String,
> {
    rewind(ticket)?;
    let capabilities = empty_stream()?;
    let mut driver_error = BSTR::new();
    unsafe { PTGetPrintCapabilities(provider.0, ticket, &capabilities, Some(&mut driver_error)) }
        .map_err(|error| api_error("PTGetPrintCapabilities", error, &driver_error))?;
    let ticket_xml = stream_to_bytes(ticket)?;
    let capabilities_xml = stream_to_bytes(&capabilities)?;
    parser::parse(&capabilities_xml, &ticket_xml)
}

fn devmode_from_ticket(provider: &Provider, ticket: &IStream) -> Result<Vec<u8>, String> {
    rewind(ticket)?;
    let mut size = 0;
    let mut output = std::ptr::null_mut::<DEVMODEA>();
    let mut driver_error = BSTR::new();
    unsafe {
        PTConvertPrintTicketToDevMode(
            provider.0,
            ticket,
            kUserDefaultDevmode,
            kPTJobScope,
            &mut size,
            &mut output,
            Some(&mut driver_error),
        )
    }
    .map_err(|error| api_error("PTConvertPrintTicketToDevMode", error, &driver_error))?;
    if output.is_null() || size == 0 {
        return Err("PTConvertPrintTicketToDevMode returned an empty DEVMODE".to_owned());
    }
    let bytes = unsafe { std::slice::from_raw_parts(output.cast::<u8>(), size as usize).to_vec() };
    unsafe { PTReleaseMemory(output.cast()) }
        .map_err(|error| format!("PTReleaseMemory failed: {error}"))?;
    Ok(bytes)
}

fn empty_stream() -> Result<IStream, String> {
    unsafe { CreateStreamOnHGlobal(None, true) }
        .map_err(|error| format!("CreateStreamOnHGlobal failed: {error}"))
}

fn stream_from_bytes(bytes: &[u8]) -> Result<IStream, String> {
    let stream = empty_stream()?;
    let mut written = 0;
    unsafe {
        stream.Write(
            bytes.as_ptr().cast(),
            bytes.len() as u32,
            Some(&mut written),
        )
    }
    .ok()
    .map_err(|error| format!("IStream::Write failed: {error}"))?;
    if written as usize != bytes.len() {
        return Err(format!(
            "IStream::Write wrote {written} of {} bytes",
            bytes.len()
        ));
    }
    rewind(&stream)?;
    Ok(stream)
}

fn stream_to_bytes(stream: &IStream) -> Result<Vec<u8>, String> {
    let mut size = 0;
    unsafe { stream.Seek(0, STREAM_SEEK_END, Some(&mut size)) }
        .map_err(|error| format!("IStream::Seek failed: {error}"))?;
    rewind(stream)?;
    let mut bytes = vec![0; size as usize];
    let mut read = 0;
    unsafe {
        stream.Read(
            bytes.as_mut_ptr().cast(),
            bytes.len() as u32,
            Some(&mut read),
        )
    }
    .ok()
    .map_err(|error| format!("IStream::Read failed: {error}"))?;
    bytes.truncate(read as usize);
    rewind(stream)?;
    Ok(bytes)
}

fn rewind(stream: &IStream) -> Result<(), String> {
    unsafe { stream.Seek(0, STREAM_SEEK_SET, None) }
        .map_err(|error| format!("IStream::Seek failed: {error}"))
}

fn api_error(api: &str, error: windows::core::Error, driver_error: &BSTR) -> String {
    let detail = driver_error.to_string();
    if detail.trim().is_empty() {
        format!("{api} failed: {error}")
    } else {
        format!("{api} failed: {error}: {detail}")
    }
}

struct Provider(HPTPROVIDER);

impl Provider {
    fn open(printer_name: &str) -> Result<Self, String> {
        let name = wide_null(printer_name);
        unsafe { PTOpenProvider(PCWSTR(name.as_ptr()), 1) }
            .map(Self)
            .map_err(|error| format!("PTOpenProvider failed: {error}"))
    }
}

impl Drop for Provider {
    fn drop(&mut self) {
        unsafe {
            let _ = PTCloseProvider(self.0);
        }
    }
}

struct ComApartment;

impl ComApartment {
    fn init() -> Result<Self, String> {
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
            .ok()
            .map_err(|error| format!("CoInitializeEx failed: {error}"))?;
        Ok(Self)
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
