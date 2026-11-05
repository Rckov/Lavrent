use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr::{self, slice_from_raw_parts_mut};
use std::slice::from_raw_parts;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread::{self, ThreadId};

use engine_core::{
    Core, CoreError, CoreOptions, DocumentHandle, HandleError, NodeHandle, PowerPreference,
    RenderTargetHandle, ResourceHandle,
};

mod diagnostics;
mod document;
mod input;
mod rendering;
mod resources;

pub const ENGINE_STATUS_OK: u32 = 0;
pub const ENGINE_STATUS_INVALID_ARGUMENT: u32 = 1;
pub const ENGINE_STATUS_WRONG_THREAD: u32 = 2;
pub const ENGINE_STATUS_INVALID_HANDLE: u32 = 3;
pub const ENGINE_STATUS_STALE_HANDLE: u32 = 4;
pub const ENGINE_STATUS_FOREIGN_HANDLE: u32 = 5;
pub const ENGINE_STATUS_CROSS_DOCUMENT_HANDLE: u32 = 6;
pub const ENGINE_STATUS_DESTROYED_HANDLE: u32 = 7;
pub const ENGINE_STATUS_INVALID_MUTATION: u32 = 8;
pub const ENGINE_STATUS_UNSUPPORTED_RESOURCE_SCHEME: u32 = 9;
pub const ENGINE_STATUS_RESOURCE_LIMIT: u32 = 10;
pub const ENGINE_STATUS_RESOURCE_DECODE: u32 = 11;
pub const ENGINE_STATUS_UNRESOLVED_DOCUMENT: u32 = 12;
pub const ENGINE_STATUS_DOCUMENT_FAILURE: u32 = 13;
pub const ENGINE_STATUS_ADAPTER_UNAVAILABLE: u32 = 14;
pub const ENGINE_STATUS_DEVICE_CREATION: u32 = 15;
pub const ENGINE_STATUS_GPU_EXECUTION: u32 = 16;
pub const ENGINE_STATUS_GPU_READBACK: u32 = 17;
pub const ENGINE_STATUS_DEVICE_LOST: u32 = 18;
pub const ENGINE_STATUS_UNSUPPORTED_OPERATION: u32 = 19;
pub const ENGINE_STATUS_PANIC: u32 = 20;

pub const ENGINE_POWER_LOW: u32 = 0;
pub const ENGINE_POWER_HIGH: u32 = 1;

pub const ENGINE_COLOR_LIGHT: u32 = 0;
pub const ENGINE_COLOR_DARK: u32 = 1;

pub const ENGINE_RESOURCE_AUTHOR_STYLESHEET: u32 = 0;
pub const ENGINE_RESOURCE_RASTER_IMAGE: u32 = 1;
pub const ENGINE_RESOURCE_SVG_DOCUMENT: u32 = 2;
pub const ENGINE_RESOURCE_FONT: u32 = 3;

pub const ENGINE_INPUT_POINTER_MOVE: u32 = 0;
pub const ENGINE_INPUT_POINTER_DOWN: u32 = 1;
pub const ENGINE_INPUT_POINTER_UP: u32 = 2;
pub const ENGINE_INPUT_POINTER_CANCEL: u32 = 3;
pub const ENGINE_INPUT_POINTER_LEAVE: u32 = 4;
pub const ENGINE_INPUT_WHEEL: u32 = 5;
pub const ENGINE_INPUT_KEY_DOWN: u32 = 6;
pub const ENGINE_INPUT_KEY_UP: u32 = 7;
pub const ENGINE_INPUT_TEXT: u32 = 8;
pub const ENGINE_INPUT_IME_PREEDIT: u32 = 9;
pub const ENGINE_INPUT_IME_COMMIT: u32 = 10;
pub const ENGINE_INPUT_IME_CANCEL: u32 = 11;
pub const ENGINE_INPUT_FOCUS_GAINED: u32 = 12;
pub const ENGINE_INPUT_FOCUS_LOST: u32 = 13;

pub const ENGINE_POINTER_MOUSE: u32 = 0;
pub const ENGINE_POINTER_TOUCH: u32 = 1;
pub const ENGINE_POINTER_PEN: u32 = 2;

pub const ENGINE_POINTER_BUTTON_PRIMARY: u32 = 0;
pub const ENGINE_POINTER_BUTTON_SECONDARY: u32 = 1;
pub const ENGINE_POINTER_BUTTON_AUXILIARY: u32 = 2;
pub const ENGINE_POINTER_BUTTON_BACK: u32 = 3;
pub const ENGINE_POINTER_BUTTON_FORWARD: u32 = 4;

pub const ENGINE_POINTER_BUTTONS_PRIMARY: u32 = 1 << 0;
pub const ENGINE_POINTER_BUTTONS_SECONDARY: u32 = 1 << 1;
pub const ENGINE_POINTER_BUTTONS_AUXILIARY: u32 = 1 << 2;
pub const ENGINE_POINTER_BUTTONS_BACK: u32 = 1 << 3;
pub const ENGINE_POINTER_BUTTONS_FORWARD: u32 = 1 << 4;

pub const ENGINE_MODIFIER_SHIFT: u32 = 1 << 0;
pub const ENGINE_MODIFIER_CONTROL: u32 = 1 << 1;
pub const ENGINE_MODIFIER_ALT: u32 = 1 << 2;
pub const ENGINE_MODIFIER_META: u32 = 1 << 3;

pub const ENGINE_KEY_UNIDENTIFIED: u32 = 0;
pub const ENGINE_KEY_NAMED: u32 = 1;
pub const ENGINE_KEY_CHARACTER: u32 = 2;

pub const ENGINE_NAMED_KEY_ALT: u32 = 0;
pub const ENGINE_NAMED_KEY_ARROW_DOWN: u32 = 1;
pub const ENGINE_NAMED_KEY_ARROW_LEFT: u32 = 2;
pub const ENGINE_NAMED_KEY_ARROW_RIGHT: u32 = 3;
pub const ENGINE_NAMED_KEY_ARROW_UP: u32 = 4;
pub const ENGINE_NAMED_KEY_BACKSPACE: u32 = 5;
pub const ENGINE_NAMED_KEY_CONTROL: u32 = 6;
pub const ENGINE_NAMED_KEY_DELETE: u32 = 7;
pub const ENGINE_NAMED_KEY_END: u32 = 8;
pub const ENGINE_NAMED_KEY_ENTER: u32 = 9;
pub const ENGINE_NAMED_KEY_ESCAPE: u32 = 10;
pub const ENGINE_NAMED_KEY_HOME: u32 = 11;
pub const ENGINE_NAMED_KEY_INSERT: u32 = 12;
pub const ENGINE_NAMED_KEY_META: u32 = 13;
pub const ENGINE_NAMED_KEY_PAGE_DOWN: u32 = 14;
pub const ENGINE_NAMED_KEY_PAGE_UP: u32 = 15;
pub const ENGINE_NAMED_KEY_SHIFT: u32 = 16;
pub const ENGINE_NAMED_KEY_SPACE: u32 = 17;
pub const ENGINE_NAMED_KEY_TAB: u32 = 18;

pub const ENGINE_KEY_LOCATION_STANDARD: u32 = 0;
pub const ENGINE_KEY_LOCATION_LEFT: u32 = 1;
pub const ENGINE_KEY_LOCATION_RIGHT: u32 = 2;
pub const ENGINE_KEY_LOCATION_NUMPAD: u32 = 3;

pub const ENGINE_UI_EVENT_CLICK: u32 = 0;
pub const ENGINE_UI_EVENT_INPUT: u32 = 1;
pub const ENGINE_UI_EVENT_CHANGE: u32 = 2;
pub const ENGINE_UI_EVENT_SUBMIT: u32 = 3;
pub const ENGINE_UI_EVENT_FOCUS: u32 = 4;
pub const ENGINE_UI_EVENT_BLUR: u32 = 5;

pub const ENGINE_UI_DATA_NONE: u32 = 0;
pub const ENGINE_UI_DATA_TEXT: u32 = 1;

pub const ENGINE_DIAGNOSTIC_INFORMATION: u32 = 0;
pub const ENGINE_DIAGNOSTIC_WARNING: u32 = 1;
pub const ENGINE_DIAGNOSTIC_ERROR: u32 = 2;

pub const ENGINE_DIAGNOSTIC_RENDERER_INITIALIZED: u32 = 0;
pub const ENGINE_DIAGNOSTIC_MISSING_RESOURCE: u32 = 1;
pub const ENGINE_DIAGNOSTIC_RESOURCE_DECODE_FAILED: u32 = 2;
pub const ENGINE_DIAGNOSTIC_RESOURCE_LIMIT_EXCEEDED: u32 = 3;
pub const ENGINE_DIAGNOSTIC_UNSUPPORTED_PAINT: u32 = 4;
pub const ENGINE_DIAGNOSTIC_UNSUPPORTED_OPERATION: u32 = 5;
pub const ENGINE_DIAGNOSTIC_CAPABILITY_UNAVAILABLE: u32 = 6;
pub const ENGINE_DIAGNOSTIC_DEVICE_LOST: u32 = 7;

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

static NEXT_HANDLE_OWNER: AtomicU32 = AtomicU32::new(1);

const MAX_HANDLE_OWNER: u32 = 0x00ff_ffff;
const DOCUMENT_HANDLE_KIND: u8 = 1;
const NODE_HANDLE_KIND: u8 = 2;
const RESOURCE_HANDLE_KIND: u8 = 3;
const RENDER_TARGET_HANDLE_KIND: u8 = 4;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct EngineBytes {
    pub data: *mut u8,
    pub length: usize,
}

#[repr(C)]
pub struct EngineOptions {
    pub power_preference: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct EngineStringView {
    pub data: *const u8,
    pub length: usize,
}

#[repr(C)]
pub struct EngineDocumentSource {
    pub html: EngineStringView,
    pub author_css: *const EngineStringView,
    pub author_css_length: usize,
    pub base_resource_uri: EngineStringView,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct EngineViewport {
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
    pub color_scheme: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct EnginePoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct EngineRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[repr(C)]
pub struct EngineHandleArray {
    pub data: *mut u64,
    pub length: usize,
}

#[repr(C)]
pub struct EngineResourceSource {
    pub uri: EngineStringView,
    pub media_type: EngineStringView,
    pub kind: u32,
    pub data: *const u8,
    pub length: usize,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct EngineRenderResult {
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
pub struct EngineReadback {
    pub width: u32,
    pub height: u32,
    pub pixels: EngineBytes,
}

#[repr(C)]
pub struct EngineInputEvent {
    pub kind: u32,
    pub pointer_id: u64,
    pub pointer_kind: u32,
    pub is_primary: u8,
    pub position: EnginePoint,
    pub button: u32,
    pub buttons: u32,
    pub modifiers: u32,
    pub delta_x: f64,
    pub delta_y: f64,
    pub key_kind: u32,
    pub named_key: u32,
    pub key_text: EngineStringView,
    pub physical_key: EngineStringView,
    pub key_location: u32,
    pub repeat: u8,
    pub text: EngineStringView,
    pub has_ime_selection: u8,
    pub ime_selection_start: usize,
    pub ime_selection_end: usize,
}

#[repr(C)]
pub struct EngineUiEvent {
    pub target: u64,
    pub kind: u32,
    pub data_kind: u32,
    pub text: EngineBytes,
}

#[repr(C)]
pub struct EngineUiEventArray {
    pub data: *mut EngineUiEvent,
    pub length: usize,
}

#[repr(C)]
pub struct EngineDiagnosticContext {
    pub key: EngineBytes,
    pub value: EngineBytes,
}

#[repr(C)]
pub struct EngineDiagnostic {
    pub code: u32,
    pub severity: u32,
    pub document: u64,
    pub node: u64,
    pub message: EngineBytes,
    pub context: *mut EngineDiagnosticContext,
    pub context_length: usize,
}

#[repr(C)]
pub struct EngineDiagnosticArray {
    pub data: *mut EngineDiagnostic,
    pub length: usize,
}

pub struct Engine {
    state: CoreState,
}

struct CoreState {
    inner: Core,
    thread: ThreadId,
    poisoned: bool,
    documents: HandleRegistry<DocumentHandle>,
    nodes: HandleRegistry<NodeHandle>,
    resources: HandleRegistry<ResourceHandle>,
    render_targets: HandleRegistry<RenderTargetHandle>,
}

struct HandleRegistry<T> {
    values: Vec<T>,
    ids: HashMap<T, u32>,
    owner: u32,
    kind: u8,
}

impl<T> HandleRegistry<T> {
    fn new(owner: u32, kind: u8) -> Self {
        Self {
            values: Vec::new(),
            ids: HashMap::new(),
            owner,
            kind,
        }
    }
}

impl<T> HandleRegistry<T>
where
    T: Copy + Eq + Hash,
{
    fn intern(&mut self, value: T) -> Result<u64, NativeCallError> {
        if let Some(id) = self.ids.get(&value) {
            return Ok(self.encode(*id));
        }
        let id = u32::try_from(self.values.len())
            .ok()
            .and_then(|value| value.checked_add(1))
            .ok_or_else(|| {
                NativeCallError::resource_limit("native handle capacity is exhausted")
            })?;
        self.values.push(value);
        self.ids.insert(value, id);
        Ok(self.encode(id))
    }

    fn resolve(&self, id: u64, kind: &'static str) -> Result<T, NativeCallError> {
        let owner = u32::try_from(id >> 40).map_err(|_| NativeCallError::invalid_handle(kind))?;
        if owner != self.owner {
            return Err(NativeCallError::foreign_handle(kind));
        }
        let encoded_kind = ((id >> 32) & u64::from(u8::MAX)) as u8;
        if encoded_kind != self.kind {
            return Err(NativeCallError::invalid_handle(kind));
        }
        let local_id = id as u32;
        let index = local_id
            .checked_sub(1)
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(|| NativeCallError::invalid_handle(kind))?;
        self.values
            .get(index)
            .copied()
            .ok_or_else(|| NativeCallError::invalid_handle(kind))
    }

    fn encode(&self, id: u32) -> u64 {
        (u64::from(self.owner) << 40) | (u64::from(self.kind) << 32) | u64::from(id)
    }
}

pub(crate) struct NativeCallError {
    status: u32,
    message: String,
}

impl NativeCallError {
    pub(crate) fn invalid(message: impl Into<String>) -> Self {
        Self {
            status: ENGINE_STATUS_INVALID_ARGUMENT,
            message: message.into(),
        }
    }

    fn invalid_handle(kind: &'static str) -> Self {
        Self {
            status: ENGINE_STATUS_INVALID_HANDLE,
            message: format!("unknown native {kind} handle"),
        }
    }

    fn foreign_handle(kind: &'static str) -> Self {
        Self {
            status: ENGINE_STATUS_FOREIGN_HANDLE,
            message: format!("foreign native {kind} handle"),
        }
    }

    fn resource_limit(message: impl Into<String>) -> Self {
        Self {
            status: ENGINE_STATUS_RESOURCE_LIMIT,
            message: message.into(),
        }
    }

    fn wrong_thread() -> Self {
        Self {
            status: ENGINE_STATUS_WRONG_THREAD,
            message: "core called from a non-owning thread".to_owned(),
        }
    }

    fn poisoned() -> Self {
        Self {
            status: ENGINE_STATUS_PANIC,
            message: "core is unavailable after a panic".to_owned(),
        }
    }
}

impl From<CoreError> for NativeCallError {
    fn from(error: CoreError) -> Self {
        let status = match &error {
            CoreError::InvalidArgument { .. } => ENGINE_STATUS_INVALID_ARGUMENT,
            CoreError::WrongThread => ENGINE_STATUS_WRONG_THREAD,
            CoreError::Handle(HandleError::Invalid) => ENGINE_STATUS_INVALID_HANDLE,
            CoreError::Handle(HandleError::Stale) => ENGINE_STATUS_STALE_HANDLE,
            CoreError::Handle(HandleError::Foreign) => ENGINE_STATUS_FOREIGN_HANDLE,
            CoreError::Handle(HandleError::CrossDocument) => ENGINE_STATUS_CROSS_DOCUMENT_HANDLE,
            CoreError::Handle(HandleError::Destroyed) => ENGINE_STATUS_DESTROYED_HANDLE,
            CoreError::InvalidMutation { .. } => ENGINE_STATUS_INVALID_MUTATION,
            CoreError::UnsupportedResourceScheme { .. } => {
                ENGINE_STATUS_UNSUPPORTED_RESOURCE_SCHEME
            }
            CoreError::ResourceLimit { .. } => ENGINE_STATUS_RESOURCE_LIMIT,
            CoreError::ResourceDecode { .. } => ENGINE_STATUS_RESOURCE_DECODE,
            CoreError::UnresolvedDocument => ENGINE_STATUS_UNRESOLVED_DOCUMENT,
            CoreError::DocumentFailure { .. } => ENGINE_STATUS_DOCUMENT_FAILURE,
            CoreError::AdapterUnavailable => ENGINE_STATUS_ADAPTER_UNAVAILABLE,
            CoreError::DeviceCreation { .. } => ENGINE_STATUS_DEVICE_CREATION,
            CoreError::GpuExecution { .. } => ENGINE_STATUS_GPU_EXECUTION,
            CoreError::GpuReadback { .. } => ENGINE_STATUS_GPU_READBACK,
            CoreError::DeviceLost => ENGINE_STATUS_DEVICE_LOST,
            CoreError::UnsupportedOperation { .. } => ENGINE_STATUS_UNSUPPORTED_OPERATION,
        };
        Self {
            status,
            message: error.to_string(),
        }
    }
}

pub(crate) fn run_ffi_call(
    core: *mut Engine,
    operation: impl FnOnce() -> Result<(), NativeCallError>,
) -> u32 {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(())) => {
            LAST_ERROR.with(|slot| *slot.borrow_mut() = None);
            ENGINE_STATUS_OK
        }
        Ok(Err(error)) => {
            let status = error.status;
            LAST_ERROR.with(|slot| *slot.borrow_mut() = Some(error.message));
            status
        }
        Err(payload) => {
            if !core.is_null() {
                // SAFETY: status-returning core functions require a live core pointer.
                unsafe { (*core).state.poisoned = true };
            }
            std::mem::forget(payload);
            LAST_ERROR.with(|slot| {
                *slot.borrow_mut() = Some("panic at the native ABI boundary".to_owned());
            });
            ENGINE_STATUS_PANIC
        }
    }
}

fn convert_power_preference(value: u32) -> Result<PowerPreference, NativeCallError> {
    match value {
        ENGINE_POWER_LOW => Ok(PowerPreference::LowPower),
        ENGINE_POWER_HIGH => Ok(PowerPreference::HighPerformance),
        _ => Err(NativeCallError::invalid("unknown power preference")),
    }
}

fn next_handle_owner() -> Result<u32, NativeCallError> {
    NEXT_HANDLE_OWNER
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |owner| {
            (owner <= MAX_HANDLE_OWNER).then_some(owner + 1)
        })
        .map_err(|_| NativeCallError::resource_limit("native core handle ownership is exhausted"))
}

pub(crate) fn into_owned_bytes(bytes: Vec<u8>) -> EngineBytes {
    if bytes.is_empty() {
        return EngineBytes {
            data: ptr::null_mut(),
            length: 0,
        };
    }
    let mut bytes = bytes.into_boxed_slice();
    let result = EngineBytes {
        data: bytes.as_mut_ptr(),
        length: bytes.len(),
    };
    let _ = Box::leak(bytes);
    result
}

pub(crate) unsafe fn free_bytes(bytes: EngineBytes) {
    if bytes.data.is_null() {
        return;
    }
    // SAFETY: the pointer and length originate from into_owned_bytes.
    let slice = slice_from_raw_parts_mut(bytes.data, bytes.length);
    // SAFETY: into_owned_bytes leaked exactly one boxed slice for this pointer.
    drop(unsafe { Box::from_raw(slice) });
}

fn into_handle_array(handles: Vec<u64>) -> EngineHandleArray {
    if handles.is_empty() {
        return EngineHandleArray {
            data: ptr::null_mut(),
            length: 0,
        };
    }
    let mut handles = handles.into_boxed_slice();
    let result = EngineHandleArray {
        data: handles.as_mut_ptr(),
        length: handles.len(),
    };
    let _ = Box::leak(handles);
    result
}

pub(crate) unsafe fn write_output<T>(output: *mut T, value: T) -> Result<(), NativeCallError> {
    require_output(output)?;
    // SAFETY: the caller provides writable, aligned storage for one T.
    unsafe { output.write(value) };
    Ok(())
}

pub(crate) fn require_output<T>(output: *mut T) -> Result<(), NativeCallError> {
    if output.is_null() {
        Err(NativeCallError::invalid("output pointer is null"))
    } else {
        Ok(())
    }
}

pub(crate) unsafe fn borrow_core<'core>(
    core: *mut Engine,
) -> Result<&'core mut CoreState, NativeCallError> {
    if core.is_null() {
        return Err(NativeCallError::invalid("core pointer is null"));
    }
    // SAFETY: each exported function requires a live, uniquely borrowed core pointer.
    let state = &mut unsafe { &mut *core }.state;
    if thread::current().id() != state.thread {
        return Err(NativeCallError::wrong_thread());
    }
    if state.poisoned {
        return Err(NativeCallError::poisoned());
    }
    Ok(state)
}

pub(crate) unsafe fn read_utf8(view: EngineStringView) -> Result<String, NativeCallError> {
    let bytes = unsafe { read_slice(view.data, view.length, "string")? };
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|_| NativeCallError::invalid("string is not valid UTF-8"))
}

pub(crate) unsafe fn read_optional_utf8(
    view: EngineStringView,
) -> Result<Option<String>, NativeCallError> {
    if view.data.is_null() {
        if view.length == 0 {
            return Ok(None);
        }
        return Err(NativeCallError::invalid("null string has nonzero length"));
    }
    unsafe { read_utf8(view) }.map(Some)
}

pub(crate) unsafe fn read_slice<'input, T>(
    data: *const T,
    length: usize,
    name: &'static str,
) -> Result<&'input [T], NativeCallError> {
    if length == 0 {
        return Ok(&[]);
    }
    if data.is_null() {
        return Err(NativeCallError::invalid(format!(
            "{name} pointer is null with a nonzero length"
        )));
    }
    // SAFETY: exported function contracts require a valid aligned array for the call.
    Ok(unsafe { from_raw_parts(data, length) })
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_abi_version() -> u32 {
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn engine_last_error_message() -> EngineBytes {
    let message = LAST_ERROR.with(|slot| slot.borrow().clone().unwrap_or_default());
    into_owned_bytes(message.into_bytes())
}

/// # Safety
///
/// `bytes` must be empty or have been returned by the native library and not freed before.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_bytes_free(bytes: EngineBytes) {
    unsafe { free_bytes(bytes) };
}

/// # Safety
///
/// `handles` must be empty or have been returned by the native library and not freed before.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_handles_free(handles: EngineHandleArray) {
    if handles.data.is_null() {
        return;
    }
    // SAFETY: the pointer and length originate from into_handle_array.
    let slice = slice_from_raw_parts_mut(handles.data, handles.length);
    // SAFETY: into_handle_array leaked exactly one boxed slice for this pointer.
    drop(unsafe { Box::from_raw(slice) });
}

/// # Safety
///
/// `output` must point to writable storage for one `Engine` pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_new(options: EngineOptions, output: *mut *mut Engine) -> u32 {
    run_ffi_call(ptr::null_mut(), || {
        require_output(output)?;
        let handle_owner = next_handle_owner()?;
        let options = CoreOptions {
            power_preference: convert_power_preference(options.power_preference)?,
        };
        let state = CoreState {
            inner: Core::new(options)?,
            thread: thread::current().id(),
            poisoned: false,
            documents: HandleRegistry::new(handle_owner, DOCUMENT_HANDLE_KIND),
            nodes: HandleRegistry::new(handle_owner, NODE_HANDLE_KIND),
            resources: HandleRegistry::new(handle_owner, RESOURCE_HANDLE_KIND),
            render_targets: HandleRegistry::new(handle_owner, RENDER_TARGET_HANDLE_KIND),
        };
        // SAFETY: required by the exported function contract.
        unsafe { write_output(output, Box::into_raw(Box::new(Engine { state }))) }
    })
}

/// # Safety
///
/// `core` must be null or a live pointer returned by `engine_new`. A live
/// pointer must be freed once, on the thread that created it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_free(core: *mut Engine) -> u32 {
    run_ffi_call(ptr::null_mut(), || {
        if core.is_null() {
            return Ok(());
        }
        // SAFETY: required by the exported function contract.
        let state = &mut unsafe { &mut *core }.state;
        if thread::current().id() != state.thread {
            return Err(NativeCallError::wrong_thread());
        }
        // SAFETY: engine_new allocated this pointer and it has not been freed.
        drop(unsafe { Box::from_raw(core) });
        Ok(())
    })
}
