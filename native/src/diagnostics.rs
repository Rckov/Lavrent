use std::ptr::{self, slice_from_raw_parts_mut};

use engine_core::{DiagnosticCode, DiagnosticSeverity};

use crate::{
    ENGINE_DIAGNOSTIC_CAPABILITY_UNAVAILABLE, ENGINE_DIAGNOSTIC_DEVICE_LOST,
    ENGINE_DIAGNOSTIC_ERROR, ENGINE_DIAGNOSTIC_INFORMATION, ENGINE_DIAGNOSTIC_MISSING_RESOURCE,
    ENGINE_DIAGNOSTIC_RENDERER_INITIALIZED, ENGINE_DIAGNOSTIC_RESOURCE_DECODE_FAILED,
    ENGINE_DIAGNOSTIC_RESOURCE_LIMIT_EXCEEDED, ENGINE_DIAGNOSTIC_UNSUPPORTED_OPERATION,
    ENGINE_DIAGNOSTIC_UNSUPPORTED_PAINT, ENGINE_DIAGNOSTIC_WARNING, Engine, EngineDiagnostic,
    EngineDiagnosticArray, EngineDiagnosticContext, borrow_core, free_bytes, into_owned_bytes,
    require_output, run_ffi_call, write_output,
};

fn convert_diagnostic_code(code: DiagnosticCode) -> u32 {
    match code {
        DiagnosticCode::RendererInitialized => ENGINE_DIAGNOSTIC_RENDERER_INITIALIZED,
        DiagnosticCode::MissingResource => ENGINE_DIAGNOSTIC_MISSING_RESOURCE,
        DiagnosticCode::ResourceDecodeFailed => ENGINE_DIAGNOSTIC_RESOURCE_DECODE_FAILED,
        DiagnosticCode::ResourceLimitExceeded => ENGINE_DIAGNOSTIC_RESOURCE_LIMIT_EXCEEDED,
        DiagnosticCode::UnsupportedPaint => ENGINE_DIAGNOSTIC_UNSUPPORTED_PAINT,
        DiagnosticCode::UnsupportedOperation => ENGINE_DIAGNOSTIC_UNSUPPORTED_OPERATION,
        DiagnosticCode::CapabilityUnavailable => ENGINE_DIAGNOSTIC_CAPABILITY_UNAVAILABLE,
        DiagnosticCode::DeviceLost => ENGINE_DIAGNOSTIC_DEVICE_LOST,
    }
}

fn convert_diagnostic_severity(severity: DiagnosticSeverity) -> u32 {
    match severity {
        DiagnosticSeverity::Information => ENGINE_DIAGNOSTIC_INFORMATION,
        DiagnosticSeverity::Warning => ENGINE_DIAGNOSTIC_WARNING,
        DiagnosticSeverity::Error => ENGINE_DIAGNOSTIC_ERROR,
    }
}

fn into_context_array(
    context: Vec<EngineDiagnosticContext>,
) -> (*mut EngineDiagnosticContext, usize) {
    if context.is_empty() {
        return (ptr::null_mut(), 0);
    }
    let mut context = context.into_boxed_slice();
    let result = (context.as_mut_ptr(), context.len());
    let _ = Box::leak(context);
    result
}

fn into_diagnostic_array(diagnostics: Vec<EngineDiagnostic>) -> EngineDiagnosticArray {
    if diagnostics.is_empty() {
        return EngineDiagnosticArray {
            data: ptr::null_mut(),
            length: 0,
        };
    }
    let mut diagnostics = diagnostics.into_boxed_slice();
    let result = EngineDiagnosticArray {
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
pub unsafe extern "C" fn engine_diagnostics_drain(
    core: *mut Engine,
    output: *mut EngineDiagnosticArray,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        require_output(output)?;
        let diagnostics = core.inner.drain_diagnostics()?;
        let mut resolved_diagnostics = Vec::with_capacity(diagnostics.len());
        for diagnostic in diagnostics {
            let document = match diagnostic.document {
                Some(handle) => core.documents.intern(handle)?,
                None => 0,
            };
            let node = match diagnostic.node {
                Some(handle) => core.nodes.intern(handle)?,
                None => 0,
            };
            resolved_diagnostics.push((diagnostic, document, node));
        }
        let mut output_diagnostics = Vec::with_capacity(resolved_diagnostics.len());
        for (diagnostic, document, node) in resolved_diagnostics {
            let context = diagnostic
                .context
                .into_iter()
                .map(|entry| EngineDiagnosticContext {
                    key: into_owned_bytes(entry.key.into_bytes()),
                    value: into_owned_bytes(entry.value.into_bytes()),
                })
                .collect();
            let (context, context_length) = into_context_array(context);
            output_diagnostics.push(EngineDiagnostic {
                code: convert_diagnostic_code(diagnostic.code),
                severity: convert_diagnostic_severity(diagnostic.severity),
                document,
                node,
                message: into_owned_bytes(diagnostic.message.into_bytes()),
                context,
                context_length,
            });
        }
        unsafe { write_output(output, into_diagnostic_array(output_diagnostics)) }
    })
}

/// # Safety
///
/// `diagnostics` must be empty or have been returned by the native library and not freed before.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_diagnostics_free(diagnostics: EngineDiagnosticArray) {
    if diagnostics.data.is_null() {
        return;
    }
    // SAFETY: the pointer and length originate from owned_diagnostics.
    let slice = slice_from_raw_parts_mut(diagnostics.data, diagnostics.length);
    // SAFETY: owned_diagnostics leaked exactly one boxed slice for this pointer.
    let diagnostics = unsafe { Box::from_raw(slice) };
    for diagnostic in &diagnostics {
        unsafe { free_bytes(diagnostic.message) };
        if diagnostic.context.is_null() {
            continue;
        }
        // SAFETY: the pointer and length originate from owned_context.
        let context = slice_from_raw_parts_mut(diagnostic.context, diagnostic.context_length);
        // SAFETY: owned_context leaked exactly one boxed slice for this pointer.
        let context = unsafe { Box::from_raw(context) };
        for entry in &context {
            unsafe {
                free_bytes(entry.key);
                free_bytes(entry.value);
            }
        }
    }
}
