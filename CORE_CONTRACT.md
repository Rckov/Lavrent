# Lavret Core Contract

## Ownership, threading, and handles

One `Core` owns every document, node handle, resource, render target, event, diagnostic, and renderer object created through it. The core is single-thread-affine: every public call checks the creating thread and returns `CoreError::WrongThread` if a violation reaches the interface. The pinned Blitz document type also makes `Core` non-`Send` in safe Rust.

Handles are opaque, copyable Lavret values. Owner, kind, slot, document, and generation checks distinguish foreign, invalid, destroyed, stale, and cross-document use. Replacing a document source preserves its document handle and invalidates all node handles for that document. Removing a subtree invalidates its handles; moving a live subtree preserves them. Destroyed slots may be reused only with a new generation.

Dropping `Core` releases offscreen textures before GPU state, renderer resources before the device, documents before the shared resource provider, and every remaining owned object without external cleanup calls.

## Documents and stage ordering

Documents are created from owned UTF-8 HTML, zero or more author CSS strings, an optional `lavret:` base URI, and a `Viewport`. Blitz remains the only source of truth for DOM, computed style, layout, text, selection, focus, scroll, and paint state. Lavret keeps no mirror DOM, style tree, layout tree, text model, or paint tree.

Direct mutations and input-driven state changes mark a document dirty. `resolve` consumes pending changes using the caller-supplied monotonic animation duration. Geometry, hit testing, and rendering reject a dirty document; Lavret does not resolve or render implicitly.

Viewport width and height are logical CSS pixels. Physical viewport dimensions are `round(logical * scale_factor)` and must fit `u32`; all three input values must also be finite, positive, and representable by the pinned document engine. `ColorScheme::Light` and `ColorScheme::Dark` are passed to Stylo through the Blitz viewport. Reduced-motion state is not present in the pinned viewport contract and is not simulated.

Border boxes and input points use document coordinates. Hit testing converts them through the current viewport scroll and returns Lavret node handles in upstream paint order.

## Resources and events

The resource registry owns in-memory author stylesheet, raster image, SVG document, and font bytes under normalized `lavret:` URIs. It rejects file paths and every other URI scheme. A resource value can be replaced while retaining its handle; removal destroys the handle. Replacement and removal invalidate documents and use Blitz's resource reload path. Missing resources produce structured diagnostics. System-font discovery and fallback remain owned by Fontique.

Normalized pointer, wheel, keyboard, committed text, IME, and core-focus input is dispatched through Blitz's event driver. Lavret returns `Click`, `Input`, `Focus`, and `Blur` events exposed by the pinned `DomEventData` contract. That revision has no `Change` or `Submit` event data to capture, so Lavret does not invent those semantics.

## Rendering

GPU state is initialized lazily when the first offscreen target is created. The selected adapter name, backend, device type, driver, `max_texture_dimension_2d`, `max_buffer_size`, normalized image-atlas dimensions, and maximum atlas count are emitted once as a Lavret information diagnostic. The AnyRender image cache uses the same normalized dimensions as the Vello GPU atlas. Adapter, device, queue, texture, and upstream renderer types remain private.

Targets have explicit physical-pixel dimensions. Width and height must be positive, fit Vello GPU's `u16` render extent, and not exceed the active wgpu `max_texture_dimension_2d`. Rendering requires a resolved document, builds an AnyRender scene through Blitz Paint, renders with Vello GPU into `Rgba8Unorm`, submits the frame, and returns dimensions without mapping GPU memory.

`readback_rgba8` is a separate diagnostic operation. It waits for the submitted frame, maps a staging buffer, removes row padding, and returns tightly packed RGBA8 pixels. Normal `render` never calls it. Content is treated as sRGB with the premultiplied-alpha behavior of the selected AnyRender/Vello path; Lavret does not claim ICC, HDR, or wide-gamut support.

## Confirmed upstream limitations

The pinned stack is not a full browser engine. Material known limitations include overflow hit testing outside clips, specific `display: contents` inheritance behavior, inline-text opacity, incomplete advanced borders and filters, ignored backdrop filters, incomplete SVG filter primitives, bounded image-atlas dimensions, incomplete emoji-presentation control, and incomplete bidi-aware editing commands.

Blitz exposes reload for linked stylesheets, but its decoded image and font caches have no public eviction operation. Replacing or removing a registered image, SVG, or font updates the provider and marks documents dirty, but a document that already cached that URI can retain the decoded old value. Lavret emits `CapabilityUnavailable` for those operations. Full same-URI replacement and deterministic removal would require an upstream change, URL/source rewriting, or a second decoding pipeline.

Blitz Paint drops unsupported URL filters while building its private scene, and the pinned AnyRender/Vello image manager does not expose why an image failed to decode or enter its private atlas. Blitz's provider interface likewise has no callback for returning downstream image, SVG, or font decode failures to Lavret. Custom backend-paint support, decode failures, and precise atlas-fit failures therefore have no public reporting hook on this path. Detecting those cases would require decoding or parsing input a second time or modifying upstream, so Lavret does neither and cannot emit a reliable `ResourceDecodeFailed`, `ResourceLimitExceeded`, or `UnsupportedPaint` diagnostic for them.

Lavret does not rewrite HTML or CSS, duplicate layout or shaping, post-process rendered pixels, silently substitute another renderer, or downscale resources to hide these limitations.

## Validation

The authoritative locked validation commands and qualified dependency graph are recorded in `DEPENDENCIES.md`.
