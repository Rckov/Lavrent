# Lavret core contract

## Ownership and handles

`Core` is bound to the thread that created it. Calls from another thread return `CoreError::WrongThread`.

Handles are opaque values tied to their owning core. Lavret rejects foreign, stale, destroyed, and cross-document handles. Replacing a document source keeps the document handle but invalidates its node handles. Removing a subtree invalidates its handles. Moving a live subtree does not.

## Documents

Documents contain UTF-8 HTML, author stylesheets, an optional `lavret:` base URI, and a viewport. Blitz owns document, layout, text, focus, selection, scroll, and paint state. Lavret does not duplicate it.

Mutations and input can make a document dirty. Call `resolve` before reading geometry, hit testing, or rendering. Lavret does not resolve implicitly. The caller supplies animation time as a monotonic duration.

Viewport dimensions are logical CSS pixels. Physical dimensions are `round(logical * scale_factor)`. Width, height, and scale factor must be finite, positive, and accepted by Blitz. Geometry and input use document coordinates.

Light and dark color schemes are supported. Reduced motion is not exposed by the pinned Blitz viewport and is therefore absent from Lavret.

## Resources and input

The resource registry accepts in-memory stylesheets, raster images, SVG documents, and fonts under `lavret:` URIs. It rejects file paths, network URLs, and other URI schemes. Replacing a resource keeps its handle. Removing it invalidates the handle. Resource changes mark affected documents dirty.

Pointer, wheel, keyboard, text, IME, and focus input is sent through Blitz. The core returns `Click`, `Input`, `Focus`, and `Blur`. The pinned revision does not expose enough data for `Change` or `Submit`, so Lavret does not synthesize them.

## Rendering

Lavret renders to offscreen GPU textures through AnyRender and Vello. GPU state is created with the first render target. Target dimensions are physical pixels and must fit Vello and the active GPU limits.

`render` requires a resolved document and does not copy pixels to the CPU. `readback_rgba8` is a separate diagnostic operation that returns tightly packed RGBA8 pixels. Output uses the renderer's sRGB and premultiplied-alpha behavior. ICC profiles, HDR, and wide-gamut output are not supported.

## Known limitations

Blitz is not a complete browser engine. Advanced CSS and SVG filters, some border effects, emoji presentation, bidi editing, hit testing, and inheritance have known gaps. Image atlas capacity depends on the renderer and GPU.

Blitz has no public operation for evicting decoded images, SVG documents, or fonts. After a resource is replaced or removed, a document that already loaded the same URI may keep the old decoded value. Lavret reports `DiagnosticCode::CapabilityUnavailable` for this case.

Blitz Paint and AnyRender do not expose every filter, decode, or atlas failure. Lavret cannot report errors it does not receive from that pipeline.

Lavret does not hide these limitations by rewriting content, decoding resources a second time, substituting another renderer, or downscaling resources.
