use std::time::Duration;

use engine_core::{ColorScheme, DocumentSource, Point, Viewport};

use crate::{
    ENGINE_COLOR_DARK, ENGINE_COLOR_LIGHT, Engine, EngineDocumentSource, EngineHandleArray,
    EnginePoint, EngineRect, EngineStringView, EngineViewport, NativeCallError, borrow_core,
    into_handle_array, read_optional_utf8, read_slice, read_utf8, require_output, run_ffi_call,
    write_output,
};

fn convert_viewport(viewport: EngineViewport) -> Result<Viewport, NativeCallError> {
    let color_scheme = match viewport.color_scheme {
        ENGINE_COLOR_LIGHT => ColorScheme::Light,
        ENGINE_COLOR_DARK => ColorScheme::Dark,
        _ => return Err(NativeCallError::invalid("unknown color scheme")),
    };
    Ok(Viewport {
        width: viewport.width,
        height: viewport.height,
        scale_factor: viewport.scale_factor,
        color_scheme,
    })
}

unsafe fn convert_document_source(
    source: EngineDocumentSource,
) -> Result<DocumentSource, NativeCallError> {
    let css_views = unsafe {
        read_slice(
            source.author_css,
            source.author_css_length,
            "author stylesheet array",
        )?
    };
    let mut author_css = Vec::with_capacity(css_views.len());
    for css in css_views {
        author_css.push(unsafe { read_utf8(*css)? });
    }
    Ok(DocumentSource {
        html: unsafe { read_utf8(source.html)? },
        author_css,
        base_resource_uri: unsafe { read_optional_utf8(source.base_resource_uri)? },
    })
}

/// # Safety
///
/// All pointers must remain valid for this call. `output` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_document_create(
    core: *mut Engine,
    source: EngineDocumentSource,
    viewport_value: EngineViewport,
    output: *mut u64,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        require_output(output)?;
        let source = unsafe { convert_document_source(source)? };
        let viewport = convert_viewport(viewport_value)?;
        let handle = core.inner.create_document(source, viewport)?;
        let handle = core.documents.intern(handle)?;
        unsafe { write_output(output, handle) }
    })
}

/// # Safety
///
/// `core` must be a live pointer returned by `engine_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_document_destroy(core: *mut Engine, document: u64) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        let document = core.documents.resolve(document, "document")?;
        core.inner.destroy_document(document)?;
        Ok(())
    })
}

/// # Safety
///
/// All pointers must remain valid for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_document_replace_source(
    core: *mut Engine,
    document: u64,
    source: EngineDocumentSource,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        let source = unsafe { convert_document_source(source)? };
        let document = core.documents.resolve(document, "document")?;
        core.inner.replace_document_source(document, source)?;
        Ok(())
    })
}

/// # Safety
///
/// `id` must remain valid for this call. `output` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_document_find_element_by_id(
    core: *mut Engine,
    document: u64,
    id: EngineStringView,
    output: *mut u64,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        require_output(output)?;
        let id = unsafe { read_utf8(id)? };
        let document = core.documents.resolve(document, "document")?;
        let handle = match core.inner.find_element_by_id(document, &id)? {
            Some(handle) => core.nodes.intern(handle)?,
            None => 0,
        };
        unsafe { write_output(output, handle) }
    })
}

/// # Safety
///
/// `name` must remain valid for this call. `output` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_node_create_element(
    core: *mut Engine,
    document: u64,
    name: EngineStringView,
    output: *mut u64,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        require_output(output)?;
        let name = unsafe { read_utf8(name)? };
        let document = core.documents.resolve(document, "document")?;
        let handle = core.inner.create_element(document, &name)?;
        let handle = core.nodes.intern(handle)?;
        unsafe { write_output(output, handle) }
    })
}

/// # Safety
///
/// `text` must remain valid for this call. `output` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_node_create_text(
    core: *mut Engine,
    document: u64,
    text: EngineStringView,
    output: *mut u64,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        require_output(output)?;
        let text = unsafe { read_utf8(text)? };
        let document = core.documents.resolve(document, "document")?;
        let handle = core.inner.create_text(document, &text)?;
        let handle = core.nodes.intern(handle)?;
        unsafe { write_output(output, handle) }
    })
}

/// # Safety
///
/// `core` must be a live pointer returned by `engine_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_node_append(
    core: *mut Engine,
    document: u64,
    parent: u64,
    child: u64,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        let document = core.documents.resolve(document, "document")?;
        let parent = core.nodes.resolve(parent, "node")?;
        let child = core.nodes.resolve(child, "node")?;
        core.inner.append_node(document, parent, child)?;
        Ok(())
    })
}

/// # Safety
///
/// `core` must be a live pointer returned by `engine_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_node_insert_before(
    core: *mut Engine,
    document: u64,
    anchor: u64,
    node: u64,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        let document = core.documents.resolve(document, "document")?;
        let anchor = core.nodes.resolve(anchor, "node")?;
        let node = core.nodes.resolve(node, "node")?;
        core.inner.insert_node_before(document, anchor, node)?;
        Ok(())
    })
}

/// # Safety
///
/// `core` must be a live pointer returned by `engine_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_node_move(
    core: *mut Engine,
    document: u64,
    node: u64,
    new_parent: u64,
    before: u64,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        let document = core.documents.resolve(document, "document")?;
        let node = core.nodes.resolve(node, "node")?;
        let new_parent = core.nodes.resolve(new_parent, "node")?;
        let before = if before == 0 {
            None
        } else {
            Some(core.nodes.resolve(before, "node")?)
        };
        core.inner.move_node(document, node, new_parent, before)?;
        Ok(())
    })
}

/// # Safety
///
/// `core` must be a live pointer returned by `engine_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_node_remove(core: *mut Engine, document: u64, node: u64) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        let document = core.documents.resolve(document, "document")?;
        let node = core.nodes.resolve(node, "node")?;
        core.inner.remove_node(document, node)?;
        Ok(())
    })
}

/// # Safety
///
/// String pointers must remain valid for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_node_set_attribute(
    core: *mut Engine,
    document: u64,
    node: u64,
    name: EngineStringView,
    value: EngineStringView,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        let name = unsafe { read_utf8(name)? };
        let value = unsafe { read_utf8(value)? };
        let document = core.documents.resolve(document, "document")?;
        let node = core.nodes.resolve(node, "node")?;
        core.inner.set_attribute(document, node, &name, &value)?;
        Ok(())
    })
}

/// # Safety
///
/// `name` must remain valid for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_node_remove_attribute(
    core: *mut Engine,
    document: u64,
    node: u64,
    name: EngineStringView,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        let name = unsafe { read_utf8(name)? };
        let document = core.documents.resolve(document, "document")?;
        let node = core.nodes.resolve(node, "node")?;
        core.inner.remove_attribute(document, node, &name)?;
        Ok(())
    })
}

/// # Safety
///
/// `text` must remain valid for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_node_replace_text(
    core: *mut Engine,
    document: u64,
    node: u64,
    text: EngineStringView,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        let text = unsafe { read_utf8(text)? };
        let document = core.documents.resolve(document, "document")?;
        let node = core.nodes.resolve(node, "node")?;
        core.inner.replace_text_content(document, node, &text)?;
        Ok(())
    })
}

/// # Safety
///
/// `core` must be a live pointer returned by `engine_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_document_set_viewport(
    core: *mut Engine,
    document: u64,
    viewport_value: EngineViewport,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        let viewport = convert_viewport(viewport_value)?;
        let document = core.documents.resolve(document, "document")?;
        core.inner.set_viewport(document, viewport)?;
        Ok(())
    })
}

/// # Safety
///
/// `core` must be a live pointer returned by `engine_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_document_resolve(
    core: *mut Engine,
    document: u64,
    animation_time_nanoseconds: u64,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        let document = core.documents.resolve(document, "document")?;
        core.inner
            .resolve(document, Duration::from_nanos(animation_time_nanoseconds))?;
        Ok(())
    })
}

/// # Safety
///
/// `output` must point to writable storage for one rectangle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_node_border_box(
    core: *mut Engine,
    document: u64,
    node: u64,
    output: *mut EngineRect,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        require_output(output)?;
        let document = core.documents.resolve(document, "document")?;
        let node = core.nodes.resolve(node, "node")?;
        let rect = core.inner.border_box(document, node)?;
        unsafe {
            write_output(
                output,
                EngineRect {
                    x: rect.origin.x,
                    y: rect.origin.y,
                    width: rect.size.width,
                    height: rect.size.height,
                },
            )
        }
    })
}

/// # Safety
///
/// `output` must point to writable storage for one handle array.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_document_hit_test(
    core: *mut Engine,
    document: u64,
    point: EnginePoint,
    output: *mut EngineHandleArray,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        require_output(output)?;
        let document = core.documents.resolve(document, "document")?;
        let nodes = core
            .inner
            .hit_test(document, Point::new(point.x, point.y))?;
        let mut handles = Vec::with_capacity(nodes.len());
        for node in nodes {
            handles.push(core.nodes.intern(node)?);
        }
        unsafe { write_output(output, into_handle_array(handles)) }
    })
}
