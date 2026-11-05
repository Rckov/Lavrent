use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr::{self, slice_from_raw_parts_mut};

use lavret_core::{Core, CoreError, CoreOptions, HandleError, PowerPreference};

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

pub struct LavretCore {
    inner: Core,
}

struct NativeError {
    status: u32,
    message: String,
}

impl NativeError {
    fn invalid(message: impl Into<String>) -> Self {
        Self {
            status: LAVRET_STATUS_INVALID_ARGUMENT,
            message: message.into(),
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

fn ffi_status(operation: impl FnOnce() -> Result<(), NativeError>) -> u32 {
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

unsafe fn write_output<T>(output: *mut T, value: T) -> Result<(), NativeError> {
    if output.is_null() {
        return Err(NativeError::invalid("output pointer is null"));
    }
    // SAFETY: the caller provides writable, aligned storage for one T.
    unsafe { output.write(value) };
    Ok(())
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
