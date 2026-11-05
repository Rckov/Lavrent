use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr::{self, slice_from_raw_parts_mut};
use std::slice::from_raw_parts;

use lavret_core::{
    Core, CoreError, CoreOptions, DocumentHandle, HandleError, NodeHandle, PowerPreference,
};

mod document;

pub const LAVRET_STATUS_OK: u32 = 0;
pub const LAVRET_STATUS_INVALID_ARGUMENT: u32 = 1;
pub const LAVRET_STATUS_WRONG_THREAD: u32 = 2;
pub const LAVRET_STATUS_INVALID_HANDLE: u32 = 3;
pub const LAVRET_STATUS_STALE_HANDLE: u32 = 4;
pub const LAVRET_STATUS_FOREIGN_HANDLE: u32 = 5;
pub const LAVRET_STATUS_CROSS_DOCUMENT_HANDLE: u32 = 6;
pub const LAVRET_STATUS_DESTROYED_HANDLE: u32 = 7;
pub const LAVRET_STATUS_INVALID_MUTATION: u32 = 8;
pub const LAVRET_STATUS_UNSUPPORTED_RESOURCE_SCHEME: u32 = 9;
pub const LAVRET_STATUS_RESOURCE_LIMIT: u32 = 10;
pub const LAVRET_STATUS_RESOURCE_DECODE: u32 = 11;
pub const LAVRET_STATUS_UNRESOLVED_DOCUMENT: u32 = 12;
pub const LAVRET_STATUS_DOCUMENT_FAILURE: u32 = 13;
pub const LAVRET_STATUS_ADAPTER_UNAVAILABLE: u32 = 14;
pub const LAVRET_STATUS_DEVICE_CREATION: u32 = 15;
pub const LAVRET_STATUS_GPU_EXECUTION: u32 = 16;
pub const LAVRET_STATUS_GPU_READBACK: u32 = 17;
pub const LAVRET_STATUS_DEVICE_LOST: u32 = 18;
pub const LAVRET_STATUS_UNSUPPORTED_OPERATION: u32 = 19;
pub const LAVRET_STATUS_PANIC: u32 = 20;

pub const LAVRET_POWER_LOW: u32 = 0;
pub const LAVRET_POWER_HIGH: u32 = 1;

pub const LAVRET_COLOR_LIGHT: u32 = 0;
pub const LAVRET_COLOR_DARK: u32 = 1;

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

#[repr(C)]
pub struct LavretBytes {
    pub data: *mut u8,
    pub length: usize,
}

#[repr(C)]
pub struct LavretCoreOptions {
    pub power_preference: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LavretStringView {
    pub data: *const u8,
    pub length: usize,
}

#[repr(C)]
pub struct LavretDocumentSource {
    pub html: LavretStringView,
    pub author_css: *const LavretStringView,
    pub author_css_length: usize,
    pub base_resource_uri: LavretStringView,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LavretViewport {
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
    pub color_scheme: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LavretPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LavretRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[repr(C)]
pub struct LavretHandleArray {
    pub data: *mut u64,
    pub length: usize,
}

pub struct LavretCore {
    inner: Core,
    documents: HandleRegistry<DocumentHandle>,
    nodes: HandleRegistry<NodeHandle>,
}

struct HandleRegistry<T> {
    values: Vec<T>,
    ids: HashMap<T, u64>,
}

impl<T> Default for HandleRegistry<T> {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            ids: HashMap::new(),
        }
    }
}

impl<T> HandleRegistry<T>
where
    T: Copy + Eq + Hash,
{
    fn intern(&mut self, value: T) -> Result<u64, NativeError> {
        if let Some(id) = self.ids.get(&value) {
            return Ok(*id);
        }
        let id = u64::try_from(self.values.len())
            .ok()
            .and_then(|value| value.checked_add(1))
            .ok_or_else(|| NativeError {
                status: LAVRET_STATUS_RESOURCE_LIMIT,
                message: "native handle capacity is exhausted".to_owned(),
            })?;
        self.values.push(value);
        self.ids.insert(value, id);
        Ok(id)
    }

    fn resolve(&self, id: u64, kind: &'static str) -> Result<T, NativeError> {
        let index = id
            .checked_sub(1)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| NativeError::invalid_handle(kind))?;
        self.values
            .get(index)
            .copied()
            .ok_or_else(|| NativeError::invalid_handle(kind))
    }
}

pub(crate) struct NativeError {
    status: u32,
    message: String,
}

impl NativeError {
    pub(crate) fn invalid(message: impl Into<String>) -> Self {
        Self {
            status: LAVRET_STATUS_INVALID_ARGUMENT,
            message: message.into(),
        }
    }

    fn invalid_handle(kind: &'static str) -> Self {
        Self {
            status: LAVRET_STATUS_INVALID_HANDLE,
            message: format!("unknown native {kind} handle"),
        }
    }
}

impl From<CoreError> for NativeError {
    fn from(error: CoreError) -> Self {
        let status = match &error {
            CoreError::InvalidArgument { .. } => LAVRET_STATUS_INVALID_ARGUMENT,
            CoreError::WrongThread => LAVRET_STATUS_WRONG_THREAD,
            CoreError::Handle(HandleError::Invalid) => LAVRET_STATUS_INVALID_HANDLE,
            CoreError::Handle(HandleError::Stale) => LAVRET_STATUS_STALE_HANDLE,
            CoreError::Handle(HandleError::Foreign) => LAVRET_STATUS_FOREIGN_HANDLE,
            CoreError::Handle(HandleError::CrossDocument) => LAVRET_STATUS_CROSS_DOCUMENT_HANDLE,
            CoreError::Handle(HandleError::Destroyed) => LAVRET_STATUS_DESTROYED_HANDLE,
            CoreError::InvalidMutation { .. } => LAVRET_STATUS_INVALID_MUTATION,
            CoreError::UnsupportedResourceScheme { .. } => {
                LAVRET_STATUS_UNSUPPORTED_RESOURCE_SCHEME
            }
            CoreError::ResourceLimit { .. } => LAVRET_STATUS_RESOURCE_LIMIT,
            CoreError::ResourceDecode { .. } => LAVRET_STATUS_RESOURCE_DECODE,
            CoreError::UnresolvedDocument => LAVRET_STATUS_UNRESOLVED_DOCUMENT,
            CoreError::DocumentFailure { .. } => LAVRET_STATUS_DOCUMENT_FAILURE,
            CoreError::AdapterUnavailable => LAVRET_STATUS_ADAPTER_UNAVAILABLE,
            CoreError::DeviceCreation { .. } => LAVRET_STATUS_DEVICE_CREATION,
            CoreError::GpuExecution { .. } => LAVRET_STATUS_GPU_EXECUTION,
            CoreError::GpuReadback { .. } => LAVRET_STATUS_GPU_READBACK,
            CoreError::DeviceLost => LAVRET_STATUS_DEVICE_LOST,
            CoreError::UnsupportedOperation { .. } => LAVRET_STATUS_UNSUPPORTED_OPERATION,
        };
        Self {
            status,
            message: error.to_string(),
        }
    }
}

pub(crate) fn ffi_status(operation: impl FnOnce() -> Result<(), NativeError>) -> u32 {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(())) => {
            LAST_ERROR.with(|slot| *slot.borrow_mut() = None);
            LAVRET_STATUS_OK
        }
        Ok(Err(error)) => {
            let status = error.status;
            LAST_ERROR.with(|slot| *slot.borrow_mut() = Some(error.message));
            status
        }
        Err(payload) => {
            std::mem::forget(payload);
            LAST_ERROR.with(|slot| {
                *slot.borrow_mut() = Some("panic at the Lavret C ABI boundary".to_owned());
            });
            LAVRET_STATUS_PANIC
        }
    }
}

fn power_preference(value: u32) -> Result<PowerPreference, NativeError> {
    match value {
        LAVRET_POWER_LOW => Ok(PowerPreference::LowPower),
        LAVRET_POWER_HIGH => Ok(PowerPreference::HighPerformance),
        _ => Err(NativeError::invalid("unknown power preference")),
    }
}

fn owned_bytes(bytes: Vec<u8>) -> LavretBytes {
    if bytes.is_empty() {
        return LavretBytes {
            data: ptr::null_mut(),
            length: 0,
        };
    }
    let mut bytes = bytes.into_boxed_slice();
    let result = LavretBytes {
        data: bytes.as_mut_ptr(),
        length: bytes.len(),
    };
    let _ = Box::leak(bytes);
    result
}

fn owned_handles(handles: Vec<u64>) -> LavretHandleArray {
    if handles.is_empty() {
        return LavretHandleArray {
            data: ptr::null_mut(),
            length: 0,
        };
    }
    let mut handles = handles.into_boxed_slice();
    let result = LavretHandleArray {
        data: handles.as_mut_ptr(),
        length: handles.len(),
    };
    let _ = Box::leak(handles);
    result
}

pub(crate) unsafe fn write_output<T>(output: *mut T, value: T) -> Result<(), NativeError> {
    if output.is_null() {
        return Err(NativeError::invalid("output pointer is null"));
    }
    // SAFETY: the caller provides writable, aligned storage for one T.
    unsafe { output.write(value) };
    Ok(())
}

pub(crate) unsafe fn core_mut<'core>(
    core: *mut LavretCore,
) -> Result<&'core mut LavretCore, NativeError> {
    if core.is_null() {
        return Err(NativeError::invalid("core pointer is null"));
    }
    // SAFETY: each exported function requires a live, uniquely borrowed core pointer.
    Ok(unsafe { &mut *core })
}

pub(crate) unsafe fn string(view: LavretStringView) -> Result<String, NativeError> {
    let bytes = unsafe { input_slice(view.data, view.length, "string")? };
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|_| NativeError::invalid("string is not valid UTF-8"))
}

pub(crate) unsafe fn optional_string(
    view: LavretStringView,
) -> Result<Option<String>, NativeError> {
    if view.data.is_null() {
        if view.length == 0 {
            return Ok(None);
        }
        return Err(NativeError::invalid(
            "optional string has a null pointer and a nonzero length",
        ));
    }
    unsafe { string(view) }.map(Some)
}

pub(crate) unsafe fn input_slice<'input, T>(
    data: *const T,
    length: usize,
    name: &'static str,
) -> Result<&'input [T], NativeError> {
    if length == 0 {
        return Ok(&[]);
    }
    if data.is_null() {
        return Err(NativeError::invalid(format!(
            "{name} pointer is null with a nonzero length"
        )));
    }
    // SAFETY: exported function contracts require a valid aligned array for the call.
    Ok(unsafe { from_raw_parts(data, length) })
}

#[unsafe(no_mangle)]
pub extern "C" fn lavret_abi_version() -> u32 {
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn lavret_last_error_message() -> LavretBytes {
    let message = LAST_ERROR.with(|slot| slot.borrow().clone().unwrap_or_default());
    owned_bytes(message.into_bytes())
}

/// # Safety
///
/// `bytes` must be empty or have been returned by Lavret and not freed before.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_bytes_free(bytes: LavretBytes) {
    if bytes.data.is_null() {
        return;
    }
    // SAFETY: the pointer and length originate from owned_bytes.
    let slice = slice_from_raw_parts_mut(bytes.data, bytes.length);
    // SAFETY: owned_bytes leaked exactly one boxed slice for this pointer.
    drop(unsafe { Box::from_raw(slice) });
}

/// # Safety
///
/// `handles` must be empty or have been returned by Lavret and not freed before.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_handles_free(handles: LavretHandleArray) {
    if handles.data.is_null() {
        return;
    }
    // SAFETY: the pointer and length originate from owned_handles.
    let slice = slice_from_raw_parts_mut(handles.data, handles.length);
    // SAFETY: owned_handles leaked exactly one boxed slice for this pointer.
    drop(unsafe { Box::from_raw(slice) });
}

/// # Safety
///
/// `output` must point to writable storage for one `LavretCore` pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_core_new(
    options: LavretCoreOptions,
    output: *mut *mut LavretCore,
) -> u32 {
    ffi_status(|| {
        let options = CoreOptions {
            power_preference: power_preference(options.power_preference)?,
        };
        let core = LavretCore {
            inner: Core::new(options)?,
            documents: HandleRegistry::default(),
            nodes: HandleRegistry::default(),
        };
        // SAFETY: required by the exported function contract.
        unsafe { write_output(output, Box::into_raw(Box::new(core))) }
    })
}

/// # Safety
///
/// `core` must be null or a live pointer returned by `lavret_core_new`. A live
/// pointer must be freed once, on the thread that created it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_core_free(core: *mut LavretCore) -> u32 {
    if core.is_null() {
        return LAVRET_STATUS_OK;
    }
    ffi_status(|| {
        // Calling a public operation verifies the core's thread before it is dropped.
        // SAFETY: required by the exported function contract.
        unsafe { &mut *core }.inner.drain_diagnostics()?;
        // SAFETY: lavret_core_new allocated this pointer and it has not been freed.
        drop(unsafe { Box::from_raw(core) });
        Ok(())
    })
}
