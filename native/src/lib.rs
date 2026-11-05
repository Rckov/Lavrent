use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr::{self, slice_from_raw_parts_mut};
use std::slice::from_raw_parts;

use lavret_core::{
    Core, CoreError, CoreOptions, DocumentHandle, HandleError, NodeHandle, PowerPreference,
    RenderTargetHandle, ResourceHandle,
};

mod diagnostics;
mod document;
mod input;
mod rendering;
mod resources;

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

pub const LAVRET_RESOURCE_AUTHOR_STYLESHEET: u32 = 0;
pub const LAVRET_RESOURCE_RASTER_IMAGE: u32 = 1;
pub const LAVRET_RESOURCE_SVG_DOCUMENT: u32 = 2;
pub const LAVRET_RESOURCE_FONT: u32 = 3;

pub const LAVRET_INPUT_POINTER_MOVE: u32 = 0;
pub const LAVRET_INPUT_POINTER_DOWN: u32 = 1;
pub const LAVRET_INPUT_POINTER_UP: u32 = 2;
pub const LAVRET_INPUT_POINTER_CANCEL: u32 = 3;
pub const LAVRET_INPUT_POINTER_LEAVE: u32 = 4;
pub const LAVRET_INPUT_WHEEL: u32 = 5;
pub const LAVRET_INPUT_KEY_DOWN: u32 = 6;
pub const LAVRET_INPUT_KEY_UP: u32 = 7;
pub const LAVRET_INPUT_TEXT: u32 = 8;
pub const LAVRET_INPUT_IME_PREEDIT: u32 = 9;
pub const LAVRET_INPUT_IME_COMMIT: u32 = 10;
pub const LAVRET_INPUT_IME_CANCEL: u32 = 11;
pub const LAVRET_INPUT_FOCUS_GAINED: u32 = 12;
pub const LAVRET_INPUT_FOCUS_LOST: u32 = 13;

pub const LAVRET_POINTER_MOUSE: u32 = 0;
pub const LAVRET_POINTER_TOUCH: u32 = 1;
pub const LAVRET_POINTER_PEN: u32 = 2;

pub const LAVRET_POINTER_BUTTON_PRIMARY: u32 = 0;
pub const LAVRET_POINTER_BUTTON_SECONDARY: u32 = 1;
pub const LAVRET_POINTER_BUTTON_AUXILIARY: u32 = 2;
pub const LAVRET_POINTER_BUTTON_BACK: u32 = 3;
pub const LAVRET_POINTER_BUTTON_FORWARD: u32 = 4;

pub const LAVRET_POINTER_BUTTONS_PRIMARY: u32 = 1 << 0;
pub const LAVRET_POINTER_BUTTONS_SECONDARY: u32 = 1 << 1;
pub const LAVRET_POINTER_BUTTONS_AUXILIARY: u32 = 1 << 2;
pub const LAVRET_POINTER_BUTTONS_BACK: u32 = 1 << 3;
pub const LAVRET_POINTER_BUTTONS_FORWARD: u32 = 1 << 4;

pub const LAVRET_MODIFIER_SHIFT: u32 = 1 << 0;
pub const LAVRET_MODIFIER_CONTROL: u32 = 1 << 1;
pub const LAVRET_MODIFIER_ALT: u32 = 1 << 2;
pub const LAVRET_MODIFIER_META: u32 = 1 << 3;

pub const LAVRET_KEY_UNIDENTIFIED: u32 = 0;
pub const LAVRET_KEY_NAMED: u32 = 1;
pub const LAVRET_KEY_CHARACTER: u32 = 2;

pub const LAVRET_NAMED_KEY_ALT: u32 = 0;
pub const LAVRET_NAMED_KEY_ARROW_DOWN: u32 = 1;
pub const LAVRET_NAMED_KEY_ARROW_LEFT: u32 = 2;
pub const LAVRET_NAMED_KEY_ARROW_RIGHT: u32 = 3;
pub const LAVRET_NAMED_KEY_ARROW_UP: u32 = 4;
pub const LAVRET_NAMED_KEY_BACKSPACE: u32 = 5;
pub const LAVRET_NAMED_KEY_CONTROL: u32 = 6;
pub const LAVRET_NAMED_KEY_DELETE: u32 = 7;
pub const LAVRET_NAMED_KEY_END: u32 = 8;
pub const LAVRET_NAMED_KEY_ENTER: u32 = 9;
pub const LAVRET_NAMED_KEY_ESCAPE: u32 = 10;
pub const LAVRET_NAMED_KEY_HOME: u32 = 11;
pub const LAVRET_NAMED_KEY_INSERT: u32 = 12;
pub const LAVRET_NAMED_KEY_META: u32 = 13;
pub const LAVRET_NAMED_KEY_PAGE_DOWN: u32 = 14;
pub const LAVRET_NAMED_KEY_PAGE_UP: u32 = 15;
pub const LAVRET_NAMED_KEY_SHIFT: u32 = 16;
pub const LAVRET_NAMED_KEY_SPACE: u32 = 17;
pub const LAVRET_NAMED_KEY_TAB: u32 = 18;

pub const LAVRET_KEY_LOCATION_STANDARD: u32 = 0;
pub const LAVRET_KEY_LOCATION_LEFT: u32 = 1;
pub const LAVRET_KEY_LOCATION_RIGHT: u32 = 2;
pub const LAVRET_KEY_LOCATION_NUMPAD: u32 = 3;

pub const LAVRET_UI_EVENT_CLICK: u32 = 0;
pub const LAVRET_UI_EVENT_INPUT: u32 = 1;
pub const LAVRET_UI_EVENT_CHANGE: u32 = 2;
pub const LAVRET_UI_EVENT_SUBMIT: u32 = 3;
pub const LAVRET_UI_EVENT_FOCUS: u32 = 4;
pub const LAVRET_UI_EVENT_BLUR: u32 = 5;

pub const LAVRET_UI_DATA_NONE: u32 = 0;
pub const LAVRET_UI_DATA_TEXT: u32 = 1;

pub const LAVRET_DIAGNOSTIC_INFORMATION: u32 = 0;
pub const LAVRET_DIAGNOSTIC_WARNING: u32 = 1;
pub const LAVRET_DIAGNOSTIC_ERROR: u32 = 2;

pub const LAVRET_DIAGNOSTIC_RENDERER_INITIALIZED: u32 = 0;
pub const LAVRET_DIAGNOSTIC_MISSING_RESOURCE: u32 = 1;
pub const LAVRET_DIAGNOSTIC_RESOURCE_DECODE_FAILED: u32 = 2;
pub const LAVRET_DIAGNOSTIC_RESOURCE_LIMIT_EXCEEDED: u32 = 3;
pub const LAVRET_DIAGNOSTIC_UNSUPPORTED_PAINT: u32 = 4;
pub const LAVRET_DIAGNOSTIC_UNSUPPORTED_OPERATION: u32 = 5;
pub const LAVRET_DIAGNOSTIC_CAPABILITY_UNAVAILABLE: u32 = 6;
pub const LAVRET_DIAGNOSTIC_DEVICE_LOST: u32 = 7;

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

#[derive(Clone, Copy)]
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

#[repr(C)]
pub struct LavretResourceSource {
    pub uri: LavretStringView,
    pub media_type: LavretStringView,
    pub kind: u32,
    pub data: *const u8,
    pub length: usize,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct LavretRenderResult {
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
pub struct LavretReadback {
    pub width: u32,
    pub height: u32,
    pub pixels: LavretBytes,
}

#[repr(C)]
pub struct LavretInputEvent {
    pub kind: u32,
    pub pointer_id: u64,
    pub pointer_kind: u32,
    pub is_primary: u8,
    pub position: LavretPoint,
    pub button: u32,
    pub buttons: u32,
    pub modifiers: u32,
    pub delta_x: f64,
    pub delta_y: f64,
    pub key_kind: u32,
    pub named_key: u32,
    pub key_text: LavretStringView,
    pub physical_key: LavretStringView,
    pub key_location: u32,
    pub repeat: u8,
    pub text: LavretStringView,
    pub has_ime_selection: u8,
    pub ime_selection_start: usize,
    pub ime_selection_end: usize,
}

#[repr(C)]
pub struct LavretUiEvent {
    pub target: u64,
    pub kind: u32,
    pub data_kind: u32,
    pub text: LavretBytes,
}

#[repr(C)]
pub struct LavretUiEventArray {
    pub data: *mut LavretUiEvent,
    pub length: usize,
}

#[repr(C)]
pub struct LavretDiagnosticContext {
    pub key: LavretBytes,
    pub value: LavretBytes,
}

#[repr(C)]
pub struct LavretDiagnostic {
    pub code: u32,
    pub severity: u32,
    pub document: u64,
    pub node: u64,
    pub message: LavretBytes,
    pub context: *mut LavretDiagnosticContext,
    pub context_length: usize,
}

#[repr(C)]
pub struct LavretDiagnosticArray {
    pub data: *mut LavretDiagnostic,
    pub length: usize,
}

pub struct LavretCore {
    inner: Core,
    documents: HandleRegistry<DocumentHandle>,
    nodes: HandleRegistry<NodeHandle>,
    resources: HandleRegistry<ResourceHandle>,
    render_targets: HandleRegistry<RenderTargetHandle>,
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

pub(crate) fn owned_bytes(bytes: Vec<u8>) -> LavretBytes {
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

pub(crate) unsafe fn free_owned_bytes(bytes: LavretBytes) {
    if bytes.data.is_null() {
        return;
    }
    // SAFETY: the pointer and length originate from owned_bytes.
    let slice = slice_from_raw_parts_mut(bytes.data, bytes.length);
    // SAFETY: owned_bytes leaked exactly one boxed slice for this pointer.
    drop(unsafe { Box::from_raw(slice) });
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
    require_output(output)?;
    // SAFETY: the caller provides writable, aligned storage for one T.
    unsafe { output.write(value) };
    Ok(())
}

pub(crate) fn require_output<T>(output: *mut T) -> Result<(), NativeError> {
    if output.is_null() {
        Err(NativeError::invalid("output pointer is null"))
    } else {
        Ok(())
    }
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
    unsafe { free_owned_bytes(bytes) };
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
        require_output(output)?;
        let options = CoreOptions {
            power_preference: power_preference(options.power_preference)?,
        };
        let core = LavretCore {
            inner: Core::new(options)?,
            documents: HandleRegistry::default(),
            nodes: HandleRegistry::default(),
            resources: HandleRegistry::default(),
            render_targets: HandleRegistry::default(),
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
