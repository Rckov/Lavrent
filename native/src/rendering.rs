use crate::{
    LavretBytes, LavretCore, LavretReadback, LavretRenderResult, core_mut, ffi_status, owned_bytes,
    require_output, write_output,
};

/// # Safety
///
/// `output` must point to writable storage for one render target handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_render_target_create(
    core: *mut LavretCore,
    width: u32,
    height: u32,
    output: *mut u64,
) -> u32 {
    ffi_status(|| {
        require_output(output)?;
        let core = unsafe { core_mut(core)? };
        let handle = core.inner.create_render_target(width, height)?;
        let handle = core.render_targets.intern(handle)?;
        unsafe { write_output(output, handle) }
    })
}

/// # Safety
///
/// `core` must be a live pointer returned by `lavret_core_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_render_target_resize(
    core: *mut LavretCore,
    target: u64,
    width: u32,
    height: u32,
) -> u32 {
    ffi_status(|| {
        let core = unsafe { core_mut(core)? };
        let target = core.render_targets.resolve(target, "render target")?;
        core.inner.resize_render_target(target, width, height)?;
        Ok(())
    })
}

/// # Safety
///
/// `core` must be a live pointer returned by `lavret_core_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_render_target_destroy(core: *mut LavretCore, target: u64) -> u32 {
    ffi_status(|| {
        let core = unsafe { core_mut(core)? };
        let target = core.render_targets.resolve(target, "render target")?;
        core.inner.destroy_render_target(target)?;
        Ok(())
    })
}

/// # Safety
///
/// `output` must point to writable storage for one render result.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_render(
    core: *mut LavretCore,
    document: u64,
    target: u64,
    output: *mut LavretRenderResult,
) -> u32 {
    ffi_status(|| {
        require_output(output)?;
        let core = unsafe { core_mut(core)? };
        let document = core.documents.resolve(document, "document")?;
        let target = core.render_targets.resolve(target, "render target")?;
        let result = core.inner.render(document, target)?;
        unsafe {
            write_output(
                output,
                LavretRenderResult {
                    width: result.width,
                    height: result.height,
                },
            )
        }
    })
}

/// # Safety
///
/// `output` must point to writable storage for one readback result.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_render_target_readback_rgba8(
    core: *mut LavretCore,
    target: u64,
    output: *mut LavretReadback,
) -> u32 {
    ffi_status(|| {
        require_output(output)?;
        let core = unsafe { core_mut(core)? };
        let target = core.render_targets.resolve(target, "render target")?;
        let readback = core.inner.readback_rgba8(target)?;
        unsafe {
            write_output(
                output,
                LavretReadback {
                    width: readback.width,
                    height: readback.height,
                    pixels: owned_bytes(readback.pixels),
                },
            )
        }
    })
}

/// # Safety
///
/// `readback` must be empty or have been returned by Lavret and not freed before.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_readback_free(readback: LavretReadback) {
    unsafe {
        crate::lavret_bytes_free(LavretBytes {
            data: readback.pixels.data,
            length: readback.pixels.length,
        });
    }
}
