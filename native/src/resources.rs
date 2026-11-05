use engine_core::{ResourceKind, ResourceSource};

use crate::{
    ENGINE_RESOURCE_AUTHOR_STYLESHEET, ENGINE_RESOURCE_FONT, ENGINE_RESOURCE_RASTER_IMAGE,
    ENGINE_RESOURCE_SVG_DOCUMENT, Engine, EngineResourceSource, NativeCallError, borrow_core,
    read_slice, read_utf8, require_output, run_ffi_call, write_output,
};

unsafe fn convert_resource_source(
    source: EngineResourceSource,
) -> Result<ResourceSource, NativeCallError> {
    let kind = match source.kind {
        ENGINE_RESOURCE_AUTHOR_STYLESHEET => ResourceKind::AuthorStylesheet,
        ENGINE_RESOURCE_RASTER_IMAGE => ResourceKind::RasterImage,
        ENGINE_RESOURCE_SVG_DOCUMENT => ResourceKind::SvgDocument,
        ENGINE_RESOURCE_FONT => ResourceKind::Font,
        _ => return Err(NativeCallError::invalid("unknown resource kind")),
    };
    Ok(ResourceSource {
        uri: unsafe { read_utf8(source.uri)? },
        media_type: unsafe { read_utf8(source.media_type)? },
        kind,
        bytes: unsafe { read_slice(source.data, source.length, "resource data")? }.to_vec(),
    })
}

/// # Safety
///
/// Source pointers must remain valid for this call. `output` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_resource_register(
    core: *mut Engine,
    source: EngineResourceSource,
    output: *mut u64,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        require_output(output)?;
        let source = unsafe { convert_resource_source(source)? };
        let handle = core.inner.register_resource(source)?;
        let handle = core.resources.intern(handle)?;
        unsafe { write_output(output, handle) }
    })
}

/// # Safety
///
/// Source pointers must remain valid for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_resource_replace(
    core: *mut Engine,
    resource: u64,
    source: EngineResourceSource,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        let source = unsafe { convert_resource_source(source)? };
        let resource = core.resources.resolve(resource, "resource")?;
        core.inner.replace_resource(resource, source)?;
        Ok(())
    })
}

/// # Safety
///
/// `core` must be a live pointer returned by `engine_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_resource_remove(core: *mut Engine, resource: u64) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        let resource = core.resources.resolve(resource, "resource")?;
        core.inner.remove_resource(resource)?;
        Ok(())
    })
}
