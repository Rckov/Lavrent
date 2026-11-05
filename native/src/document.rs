use std::time::Duration;

use lavret_core::{ColorScheme, DocumentSource, Point, Viewport};

use crate::{
    LAVRET_COLOR_DARK, LAVRET_COLOR_LIGHT, LavretCore, LavretDocumentSource, LavretHandleArray,
    LavretPoint, LavretRect, LavretStringView, LavretViewport, NativeError, core_mut, ffi_status,
    input_slice, optional_string, owned_handles, string, write_output,
};

fn viewport(viewport: LavretViewport) -> Result<Viewport, NativeError> {
    let color_scheme = match viewport.color_scheme {
        LAVRET_COLOR_LIGHT => ColorScheme::Light,
        LAVRET_COLOR_DARK => ColorScheme::Dark,
        _ => return Err(NativeError::invalid("unknown color scheme")),
    };
    Ok(Viewport {
        width: viewport.width,
        height: viewport.height,
        scale_factor: viewport.scale_factor,
        color_scheme,
    })
}

unsafe fn document_source(source: LavretDocumentSource) -> Result<DocumentSource, NativeError> {
    let css_views = unsafe {
        input_slice(
            source.author_css,
            source.author_css_length,
            "author stylesheet array",
        )?
    };
    let mut author_css = Vec::with_capacity(css_views.len());
    for css in css_views {
        author_css.push(unsafe { string(*css)? });
    }
    Ok(DocumentSource {
        html: unsafe { string(source.html)? },
        author_css,
        base_resource_uri: unsafe { optional_string(source.base_resource_uri)? },
    })
}

/// # Safety
///
/// All pointers must remain valid for this call. `output` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_document_create(
    core: *mut LavretCore,
    source: LavretDocumentSource,
    viewport_value: LavretViewport,
    output: *mut u64,
) -> u32 {
    ffi_status(|| {
        let source = unsafe { document_source(source)? };
        let viewport = viewport(viewport_value)?;
        let core = unsafe { core_mut(core)? };
        let handle = core.inner.create_document(source, viewport)?;
        let handle = core.documents.intern(handle)?;
        unsafe { write_output(output, handle) }
    })
}

/// # Safety
///
/// `core` must be a live pointer returned by `lavret_core_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_document_destroy(core: *mut LavretCore, document: u64) -> u32 {
    ffi_status(|| {
        let core = unsafe { core_mut(core)? };
        let document = core.documents.resolve(document, "document")?;
        core.inner.destroy_document(document)?;
        Ok(())
    })
}

/// # Safety
///
/// All pointers must remain valid for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_document_replace_source(
    core: *mut LavretCore,
    document: u64,
    source: LavretDocumentSource,
) -> u32 {
    ffi_status(|| {
        let source = unsafe { document_source(source)? };
        let core = unsafe { core_mut(core)? };
        let document = core.documents.resolve(document, "document")?;
        core.inner.replace_document_source(document, source)?;
        Ok(())
    })
}

/// # Safety
///
/// `id` must remain valid for this call. `output` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_document_find_element_by_id(
    core: *mut LavretCore,
    document: u64,
    id: LavretStringView,
    output: *mut u64,
) -> u32 {
    ffi_status(|| {
        let id = unsafe { string(id)? };
        let core = unsafe { core_mut(core)? };
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
pub unsafe extern "C" fn lavret_node_create_element(
    core: *mut LavretCore,
    document: u64,
    name: LavretStringView,
    output: *mut u64,
) -> u32 {
    ffi_status(|| {
        let name = unsafe { string(name)? };
        let core = unsafe { core_mut(core)? };
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
pub unsafe extern "C" fn lavret_node_create_text(
    core: *mut LavretCore,
    document: u64,
    text: LavretStringView,
    output: *mut u64,
) -> u32 {
    ffi_status(|| {
        let text = unsafe { string(text)? };
        let core = unsafe { core_mut(core)? };
        let document = core.documents.resolve(document, "document")?;
        let handle = core.inner.create_text(document, &text)?;
        let handle = core.nodes.intern(handle)?;
        unsafe { write_output(output, handle) }
    })
}

/// # Safety
///
/// `core` must be a live pointer returned by `lavret_core_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_node_append(
    core: *mut LavretCore,
    document: u64,
    parent: u64,
    child: u64,
) -> u32 {
    ffi_status(|| {
        let core = unsafe { core_mut(core)? };
        let document = core.documents.resolve(document, "document")?;
        let parent = core.nodes.resolve(parent, "node")?;
        let child = core.nodes.resolve(child, "node")?;
        core.inner.append_node(document, parent, child)?;
        Ok(())
    })
}

/// # Safety
///
/// `core` must be a live pointer returned by `lavret_core_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_node_insert_before(
    core: *mut LavretCore,
    document: u64,
    anchor: u64,
    node: u64,
) -> u32 {
    ffi_status(|| {
        let core = unsafe { core_mut(core)? };
        let document = core.documents.resolve(document, "document")?;
        let anchor = core.nodes.resolve(anchor, "node")?;
        let node = core.nodes.resolve(node, "node")?;
        core.inner.insert_node_before(document, anchor, node)?;
        Ok(())
    })
}

/// # Safety
///
/// `core` must be a live pointer returned by `lavret_core_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_node_move(
    core: *mut LavretCore,
    document: u64,
    node: u64,
    new_parent: u64,
    before: u64,
) -> u32 {
    ffi_status(|| {
        let core = unsafe { core_mut(core)? };
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
/// `core` must be a live pointer returned by `lavret_core_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_node_remove(
    core: *mut LavretCore,
    document: u64,
    node: u64,
) -> u32 {
    ffi_status(|| {
        let core = unsafe { core_mut(core)? };
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
pub unsafe extern "C" fn lavret_node_set_attribute(
    core: *mut LavretCore,
    document: u64,
    node: u64,
    name: LavretStringView,
    value: LavretStringView,
) -> u32 {
    ffi_status(|| {
        let name = unsafe { string(name)? };
        let value = unsafe { string(value)? };
        let core = unsafe { core_mut(core)? };
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
pub unsafe extern "C" fn lavret_node_remove_attribute(
    core: *mut LavretCore,
    document: u64,
    node: u64,
    name: LavretStringView,
) -> u32 {
    ffi_status(|| {
        let name = unsafe { string(name)? };
        let core = unsafe { core_mut(core)? };
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
pub unsafe extern "C" fn lavret_node_replace_text(
    core: *mut LavretCore,
    document: u64,
    node: u64,
    text: LavretStringView,
) -> u32 {
    ffi_status(|| {
        let text = unsafe { string(text)? };
        let core = unsafe { core_mut(core)? };
        let document = core.documents.resolve(document, "document")?;
        let node = core.nodes.resolve(node, "node")?;
        core.inner.replace_text_content(document, node, &text)?;
        Ok(())
    })
}

/// # Safety
///
/// `core` must be a live pointer returned by `lavret_core_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_document_set_viewport(
    core: *mut LavretCore,
    document: u64,
    viewport_value: LavretViewport,
) -> u32 {
    ffi_status(|| {
        let viewport = viewport(viewport_value)?;
        let core = unsafe { core_mut(core)? };
        let document = core.documents.resolve(document, "document")?;
        core.inner.set_viewport(document, viewport)?;
        Ok(())
    })
}

/// # Safety
///
/// `core` must be a live pointer returned by `lavret_core_new`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_document_resolve(
    core: *mut LavretCore,
    document: u64,
    animation_time_nanoseconds: u64,
) -> u32 {
    ffi_status(|| {
        let core = unsafe { core_mut(core)? };
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
pub unsafe extern "C" fn lavret_node_border_box(
    core: *mut LavretCore,
    document: u64,
    node: u64,
    output: *mut LavretRect,
) -> u32 {
    ffi_status(|| {
        let core = unsafe { core_mut(core)? };
        let document = core.documents.resolve(document, "document")?;
        let node = core.nodes.resolve(node, "node")?;
        let rect = core.inner.border_box(document, node)?;
        unsafe {
            write_output(
                output,
                LavretRect {
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
pub unsafe extern "C" fn lavret_document_hit_test(
    core: *mut LavretCore,
    document: u64,
    point: LavretPoint,
    output: *mut LavretHandleArray,
) -> u32 {
    ffi_status(|| {
        let core = unsafe { core_mut(core)? };
        let document = core.documents.resolve(document, "document")?;
        let nodes = core
            .inner
            .hit_test(document, Point::new(point.x, point.y))?;
        let mut handles = Vec::with_capacity(nodes.len());
        for node in nodes {
            handles.push(core.nodes.intern(node)?);
        }
        unsafe { write_output(output, owned_handles(handles)) }
    })
}
