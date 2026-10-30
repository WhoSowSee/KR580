use crate::devices::printer::PrinterError;
mod delta;
mod parser;

use super::super::wide_null;
use super::{AlignedDevMode, configuration_from_devmode, print_devmode};
use crate::devices::printer::{
    PrinterInfo, PrinterPropertyChange, PrinterPropertySheet, PrinterSettings,
};
use windows::Win32::Foundation::HGLOBAL;
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

const MAX_STREAM_BYTES: usize = 16_777_216;

pub(super) fn load(
    printer: &PrinterInfo,
    settings: &PrinterSettings,
) -> Result<PrinterPropertySheet, PrinterError> {
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
            provider_error: Some(error.to_string()),
        }),
    }
}

pub(super) fn apply(
    printer: &PrinterInfo,
    settings: &PrinterSettings,
    change: &PrinterPropertyChange,
) -> Result<PrinterPropertySheet, PrinterError> {
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
    // SAFETY: The provider and live COM streams belong to this initialized thread; the output BSTR remains live.
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
    PrinterError,
> {
    let _com = ComApartment::init()?;
    let provider = Provider::open(printer_name)?;
    let ticket = ticket_from_devmode(&provider, devmode)?;
    capabilities_from_ticket(&provider, &ticket)
}

fn ticket_from_devmode(provider: &Provider, devmode: &[u8]) -> Result<IStream, PrinterError> {
    let ticket = empty_stream()?;
    let size = u32::try_from(devmode.len())
        .map_err(|error| PrinterError::invalid_source("printer DEVMODE is too large", error))?;
    let devmode = AlignedDevMode::from_bytes(devmode);
    // SAFETY: The same-thread provider receives a validated aligned DEVMODE blob of size bytes and a live stream.
    unsafe {
        PTConvertDevModeToPrintTicket(
            provider.0,
            size,
            devmode.as_ptr().cast::<DEVMODEA>(),
            kPTJobScope,
            &ticket,
        )
    }
    .map_err(|error| {
        PrinterError::native(
            format!("PTConvertDevModeToPrintTicket failed: {error}"),
            error,
        )
    })?;
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
    PrinterError,
> {
    rewind(ticket)?;
    let capabilities = empty_stream()?;
    let mut driver_error = BSTR::new();
    // SAFETY: The same-thread provider and rewound input/output streams remain live with the output BSTR.
    unsafe { PTGetPrintCapabilities(provider.0, ticket, &capabilities, Some(&mut driver_error)) }
        .map_err(|error| api_error("PTGetPrintCapabilities", error, &driver_error))?;
    let ticket_xml = stream_to_bytes(ticket)?;
    let capabilities_xml = stream_to_bytes(&capabilities)?;
    parser::parse(&capabilities_xml, &ticket_xml)
}

fn devmode_from_ticket(provider: &Provider, ticket: &IStream) -> Result<Vec<u8>, PrinterError> {
    rewind(ticket)?;
    let mut size = 0;
    let mut output = std::ptr::null_mut::<DEVMODEA>();
    let mut driver_error = BSTR::new();
    // SAFETY: The same-thread provider and rewound stream remain live; size/pointer/BSTR outputs are writable.
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
    let output = ProviderMemory(output.cast());
    if output.0.is_null() || size == 0 || size as usize > isize::MAX as usize {
        return Err(PrinterError::invalid(
            "PTConvertPrintTicketToDevMode returned an empty DEVMODE",
        ));
    }
    // SAFETY: Successful conversion owns a non-null allocation of size initialized bytes until release.
    let bytes =
        unsafe { std::slice::from_raw_parts(output.0.cast::<u8>(), size as usize).to_vec() };
    output.release()?;
    Ok(bytes)
}

fn empty_stream() -> Result<IStream, PrinterError> {
    // SAFETY: Null HGLOBAL requests new owned storage; true makes the COM stream release it on Drop.
    unsafe { CreateStreamOnHGlobal(HGLOBAL(std::ptr::null_mut()), true) }.map_err(|error| {
        PrinterError::native(format!("CreateStreamOnHGlobal failed: {error}"), error)
    })
}

fn stream_from_bytes(bytes: &[u8]) -> Result<IStream, PrinterError> {
    if bytes.len() > MAX_STREAM_BYTES {
        return Err(PrinterError::invalid(
            "printer ticket exceeds the stream size limit",
        ));
    }
    let stream = empty_stream()?;
    let mut written = 0;
    // SAFETY: The bounded initialized input covers the byte count and written points to a live output.
    unsafe {
        stream.Write(
            bytes.as_ptr().cast(),
            bytes.len() as u32,
            Some(&mut written),
        )
    }
    .ok()
    .map_err(|error| PrinterError::native(format!("IStream::Write failed: {error}"), error))?;
    if written as usize != bytes.len() {
        return Err(PrinterError::invalid(format!(
            "IStream::Write wrote {written} of {} bytes",
            bytes.len()
        )));
    }
    rewind(&stream)?;
    Ok(stream)
}

fn stream_to_bytes(stream: &IStream) -> Result<Vec<u8>, PrinterError> {
    let mut size = 0;
    // SAFETY: The live same-thread stream writes its end position to a live u64 output.
    unsafe { stream.Seek(0, STREAM_SEEK_END, Some(&mut size)) }
        .map_err(|error| PrinterError::native(format!("IStream::Seek failed: {error}"), error))?;
    rewind(stream)?;
    let size = usize::try_from(size)
        .ok()
        .filter(|size| *size <= MAX_STREAM_BYTES)
        .ok_or_else(|| PrinterError::invalid("printer ticket exceeds the stream size limit"))?;
    let mut bytes = vec![0; size];
    let mut read = 0;
    // SAFETY: The bounded initialized byte buffer covers the supplied capacity and read is a live output.
    unsafe {
        stream.Read(
            bytes.as_mut_ptr().cast(),
            bytes.len() as u32,
            Some(&mut read),
        )
    }
    .ok()
    .map_err(|error| PrinterError::native(format!("IStream::Read failed: {error}"), error))?;
    bytes.truncate(read as usize);
    rewind(stream)?;
    Ok(bytes)
}

fn rewind(stream: &IStream) -> Result<(), PrinterError> {
    // SAFETY: The live same-thread stream accepts a zero seek offset and no result pointer.
    unsafe { stream.Seek(0, STREAM_SEEK_SET, None) }
        .map_err(|error| PrinterError::native(format!("IStream::Seek failed: {error}"), error))
}

fn api_error(api: &str, error: windows::core::Error, driver_error: &BSTR) -> PrinterError {
    let detail = driver_error.to_string();
    let message = if detail.trim().is_empty() {
        format!("{api} failed: {error}")
    } else {
        format!("{api} failed: {error}: {detail}")
    };
    PrinterError::native(message, error)
}

struct Provider(HPTPROVIDER, std::marker::PhantomData<std::rc::Rc<()>>);

impl Provider {
    fn open(printer_name: &str) -> Result<Self, PrinterError> {
        let name = wide_null(printer_name);
        // SAFETY: The terminated printer name remains live and the provider is retained on this initialized COM thread.
        unsafe { PTOpenProvider(PCWSTR(name.as_ptr()), 1) }
            .map(|handle| Self(handle, std::marker::PhantomData))
            .map_err(|error| PrinterError::native(format!("PTOpenProvider failed: {error}"), error))
    }
}

impl Drop for Provider {
    fn drop(&mut self) {
        // SAFETY: This non-Send guard closes the provider before its owning COM apartment is dropped.
        unsafe {
            let _ = PTCloseProvider(self.0);
        }
    }
}

struct ComApartment(std::marker::PhantomData<std::rc::Rc<()>>);

impl ComApartment {
    fn init() -> Result<Self, PrinterError> {
        // SAFETY: No reserved pointer is supplied; success is paired with the non-Send guard on this thread.
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
            .ok()
            .map_err(|error| {
                PrinterError::native(format!("CoInitializeEx failed: {error}"), error)
            })?;
        Ok(Self(std::marker::PhantomData))
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        // SAFETY: This non-Send guard pairs one successful COM initialization on the same thread.
        unsafe {
            CoUninitialize();
        }
    }
}

struct ProviderMemory(*mut std::ffi::c_void);

impl ProviderMemory {
    fn release(mut self) -> Result<(), PrinterError> {
        let ptr = std::mem::replace(&mut self.0, std::ptr::null_mut());
        // SAFETY: Conversion returned this provider-owned allocation and ownership is consumed once.
        unsafe { PTReleaseMemory(ptr) }.map_err(|error| {
            PrinterError::native(format!("PTReleaseMemory failed: {error}"), error)
        })
    }
}

impl Drop for ProviderMemory {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: The owned conversion allocation has not been explicitly released.
            if let Err(error) = unsafe { PTReleaseMemory(self.0) } {
                tracing::warn!(%error, "printer conversion memory release failed");
            }
        }
    }
}
