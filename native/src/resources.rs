use lavret_core::{ResourceKind, ResourceSource};

use crate::{
    LAVRET_RESOURCE_AUTHOR_STYLESHEET, LAVRET_RESOURCE_FONT, LAVRET_RESOURCE_RASTER_IMAGE,
    LAVRET_RESOURCE_SVG_DOCUMENT, LavretCore, LavretResourceSource, NativeError, core_mut,
    ffi_status, input_slice, require_output, string, write_output,
};

unsafe fn resource_source(source: LavretResourceSource) -> Result<ResourceSource, NativeError> {
    let kind = match source.kind {
        LAVRET_RESOURCE_AUTHOR_STYLESHEET => ResourceKind::AuthorStylesheet,
        LAVRET_RESOURCE_RASTER_IMAGE => ResourceKind::RasterImage,
        LAVRET_RESOURCE_SVG_DOCUMENT => ResourceKind::SvgDocument,
        LAVRET_RESOURCE_FONT => ResourceKind::Font,
        _ => return Err(NativeError::invalid("unknown resource kind")),
    };
    Ok(ResourceSource {
        uri: unsafe { string(source.uri)? },
        media_type: unsafe { string(source.media_type)? },
        kind,
        bytes: unsafe { input_slice(source.data, source.length, "resource data")? }.to_vec(),
    })
}

/// # Safety
///
/// Source pointers must remain valid for this call. `output` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_resource_register(
    core: *mut LavretCore,
    source: LavretResourceSource,
    output: *mut u64,
) -> u32 {
    ffi_status(|| {
        require_output(output)?;
        let source = unsafe { resource_source(source)? };
        let core = unsafe { core_mut(core)? };
        let handle = core.inner.register_resource(source)?;
        let handle = core.resources.intern(handle)?;
        unsafe { write_output(output, handle) }
    })
}

/// # Safety
///
/// Source pointers must remain valid for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_resource_replace(
    core: *mut LavretCore,
    resource: u64,
    source: LavretResourceSource,
) -> u32 {
    ffi_status(|| {
        let source = unsafe { resource_source(source)? };
        let core = unsafe { core_mut(core)? };
        let resource = core.resources.resolve(resource, "resource")?;
        core.inner.replace_resource(resource, source)?;
        Ok(())
    })
}

/// # Safety
///
/// `core` must be a live pointer returned by `lavret_core_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_resource_remove(core: *mut LavretCore, resource: u64) -> u32 {
    ffi_status(|| {
        let core = unsafe { core_mut(core)? };
        let resource = core.resources.resolve(resource, "resource")?;
        core.inner.remove_resource(resource)?;
        Ok(())
    })
}
