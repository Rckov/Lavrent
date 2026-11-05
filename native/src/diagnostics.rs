use std::ptr::{self, slice_from_raw_parts_mut};

use lavret_core::{DiagnosticCode, DiagnosticSeverity};

use crate::{
    LAVRET_DIAGNOSTIC_CAPABILITY_UNAVAILABLE, LAVRET_DIAGNOSTIC_DEVICE_LOST,
    LAVRET_DIAGNOSTIC_ERROR, LAVRET_DIAGNOSTIC_INFORMATION, LAVRET_DIAGNOSTIC_MISSING_RESOURCE,
    LAVRET_DIAGNOSTIC_RENDERER_INITIALIZED, LAVRET_DIAGNOSTIC_RESOURCE_DECODE_FAILED,
    LAVRET_DIAGNOSTIC_RESOURCE_LIMIT_EXCEEDED, LAVRET_DIAGNOSTIC_UNSUPPORTED_OPERATION,
    LAVRET_DIAGNOSTIC_UNSUPPORTED_PAINT, LAVRET_DIAGNOSTIC_WARNING, LavretCore, LavretDiagnostic,
    LavretDiagnosticArray, LavretDiagnosticContext, core_mut, ffi_status, free_owned_bytes,
    owned_bytes, require_output, write_output,
};

fn diagnostic_code(code: DiagnosticCode) -> u32 {
    match code {
        DiagnosticCode::RendererInitialized => LAVRET_DIAGNOSTIC_RENDERER_INITIALIZED,
        DiagnosticCode::MissingResource => LAVRET_DIAGNOSTIC_MISSING_RESOURCE,
        DiagnosticCode::ResourceDecodeFailed => LAVRET_DIAGNOSTIC_RESOURCE_DECODE_FAILED,
        DiagnosticCode::ResourceLimitExceeded => LAVRET_DIAGNOSTIC_RESOURCE_LIMIT_EXCEEDED,
        DiagnosticCode::UnsupportedPaint => LAVRET_DIAGNOSTIC_UNSUPPORTED_PAINT,
        DiagnosticCode::UnsupportedOperation => LAVRET_DIAGNOSTIC_UNSUPPORTED_OPERATION,
        DiagnosticCode::CapabilityUnavailable => LAVRET_DIAGNOSTIC_CAPABILITY_UNAVAILABLE,
        DiagnosticCode::DeviceLost => LAVRET_DIAGNOSTIC_DEVICE_LOST,
    }
}

fn diagnostic_severity(severity: DiagnosticSeverity) -> u32 {
    match severity {
        DiagnosticSeverity::Information => LAVRET_DIAGNOSTIC_INFORMATION,
        DiagnosticSeverity::Warning => LAVRET_DIAGNOSTIC_WARNING,
        DiagnosticSeverity::Error => LAVRET_DIAGNOSTIC_ERROR,
    }
}

fn owned_context(context: Vec<LavretDiagnosticContext>) -> (*mut LavretDiagnosticContext, usize) {
    if context.is_empty() {
        return (ptr::null_mut(), 0);
    }
    let mut context = context.into_boxed_slice();
    let result = (context.as_mut_ptr(), context.len());
    let _ = Box::leak(context);
    result
}

fn owned_diagnostics(diagnostics: Vec<LavretDiagnostic>) -> LavretDiagnosticArray {
    if diagnostics.is_empty() {
        return LavretDiagnosticArray {
            data: ptr::null_mut(),
            length: 0,
        };
    }
    let mut diagnostics = diagnostics.into_boxed_slice();
    let result = LavretDiagnosticArray {
        data: diagnostics.as_mut_ptr(),
        length: diagnostics.len(),
    };
    let _ = Box::leak(diagnostics);
    result
}

/// # Safety
///
/// `output` must point to writable storage for one diagnostic array.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_diagnostics_drain(
    core: *mut LavretCore,
    output: *mut LavretDiagnosticArray,
) -> u32 {
    ffi_status(|| {
        require_output(output)?;
        let core = unsafe { core_mut(core)? };
        let diagnostics = core.inner.drain_diagnostics()?;
        let mut output_diagnostics = Vec::with_capacity(diagnostics.len());
        for diagnostic in diagnostics {
            let document = match diagnostic.document {
                Some(handle) => core.documents.intern(handle)?,
                None => 0,
            };
            let node = match diagnostic.node {
                Some(handle) => core.nodes.intern(handle)?,
                None => 0,
            };
            let context = diagnostic
                .context
                .into_iter()
                .map(|entry| LavretDiagnosticContext {
                    key: owned_bytes(entry.key.into_bytes()),
                    value: owned_bytes(entry.value.into_bytes()),
                })
                .collect();
            let (context, context_length) = owned_context(context);
            output_diagnostics.push(LavretDiagnostic {
                code: diagnostic_code(diagnostic.code),
                severity: diagnostic_severity(diagnostic.severity),
                document,
                node,
                message: owned_bytes(diagnostic.message.into_bytes()),
                context,
                context_length,
            });
        }
        unsafe { write_output(output, owned_diagnostics(output_diagnostics)) }
    })
}

/// # Safety
///
/// `diagnostics` must be empty or have been returned by Lavret and not freed before.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_diagnostics_free(diagnostics: LavretDiagnosticArray) {
    if diagnostics.data.is_null() {
        return;
    }
    // SAFETY: the pointer and length originate from owned_diagnostics.
    let slice = slice_from_raw_parts_mut(diagnostics.data, diagnostics.length);
    // SAFETY: owned_diagnostics leaked exactly one boxed slice for this pointer.
    let diagnostics = unsafe { Box::from_raw(slice) };
    for diagnostic in &diagnostics {
        unsafe { free_owned_bytes(diagnostic.message) };
        if diagnostic.context.is_null() {
            continue;
        }
        // SAFETY: the pointer and length originate from owned_context.
        let context = slice_from_raw_parts_mut(diagnostic.context, diagnostic.context_length);
        // SAFETY: owned_context leaked exactly one boxed slice for this pointer.
        let context = unsafe { Box::from_raw(context) };
        for entry in &context {
            unsafe {
                free_owned_bytes(entry.key);
                free_owned_bytes(entry.value);
            }
        }
    }
}
