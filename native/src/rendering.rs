use crate::{
    Engine, EngineBytes, EngineReadback, EngineRenderResult, borrow_core, into_owned_bytes,
    require_output, run_ffi_call, write_output,
};

/// # Safety
///
/// `output` must point to writable storage for one render target handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_render_target_create(
    core: *mut Engine,
    width: u32,
    height: u32,
    output: *mut u64,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        require_output(output)?;
        let handle = core.inner.create_render_target(width, height)?;
        let handle = core.render_targets.intern(handle)?;
        unsafe { write_output(output, handle) }
    })
}

/// # Safety
///
/// `core` must be a live pointer returned by `engine_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_render_target_resize(
    core: *mut Engine,
    target: u64,
    width: u32,
    height: u32,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        let target = core.render_targets.resolve(target, "render target")?;
        core.inner.resize_render_target(target, width, height)?;
        Ok(())
    })
}

/// # Safety
///
/// `core` must be a live pointer returned by `engine_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_render_target_destroy(core: *mut Engine, target: u64) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        let target = core.render_targets.resolve(target, "render target")?;
        core.inner.destroy_render_target(target)?;
        Ok(())
    })
}

/// # Safety
///
/// `output` must point to writable storage for one render result.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_render(
    core: *mut Engine,
    document: u64,
    target: u64,
    output: *mut EngineRenderResult,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        require_output(output)?;
        let document = core.documents.resolve(document, "document")?;
        let target = core.render_targets.resolve(target, "render target")?;
        let result = core.inner.render(document, target)?;
        unsafe {
            write_output(
                output,
                EngineRenderResult {
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
pub unsafe extern "C" fn engine_render_target_readback_rgba8(
    core: *mut Engine,
    target: u64,
    output: *mut EngineReadback,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        require_output(output)?;
        let target = core.render_targets.resolve(target, "render target")?;
        let readback = core.inner.readback_rgba8(target)?;
        unsafe {
            write_output(
                output,
                EngineReadback {
                    width: readback.width,
                    height: readback.height,
                    pixels: into_owned_bytes(readback.pixels),
                },
            )
        }
    })
}

/// # Safety
///
/// `readback` must be empty or have been returned by the native library and not freed before.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_readback_free(readback: EngineReadback) {
    unsafe {
        crate::engine_bytes_free(EngineBytes {
            data: readback.pixels.data,
            length: readback.pixels.length,
        });
    }
}
