# Lavret Core Contract

## Ownership and lifecycle

One `Core` owns every document, node handle, resource, render target, diagnostic, and renderer object created through it. A core is single-thread-affine and accepts calls only from its creating thread. Dropping the core releases renderer state before the device and all document-owned objects before shared resources.

Handles are opaque Lavret values. They carry enough owner, kind, slot, and generation identity to reject stale, foreign, destroyed, cross-kind, and cross-document use. Replacing a document source preserves its document handle and invalidates all node handles for that document. Removing a subtree invalidates removed-node handles; moving a live subtree preserves them.

## Document stages

Documents are created from owned UTF-8 HTML, author CSS sources, and an optional `lavret:` base URI. Mutations mark a document dirty. `resolve` is the only operation that consumes pending style, layout, text, media, and interaction changes. `render` rejects dirty documents and never performs CPU readback.

Blitz is the sole internal source of truth for DOM, computed style, layout, text, selection, focus, scroll, and paint state. Lavret does not keep a mirror DOM, layout tree, text representation, or paint tree.

## Resources

Resources are immutable in-memory stylesheet, raster-image, SVG, or font payloads identified by `lavret:` URIs. The registry rejects file paths and non-`lavret` schemes. System fonts remain owned by Fontique. Replacing or removing a resource marks dependent documents dirty; missing resources produce structured diagnostics.

## Rendering

Rendering uses Vello GPU through AnyRender into an offscreen `Rgba8Unorm` texture. Content is treated as sRGB with the premultiplied-alpha semantics of the selected AnyRender/Vello path. The normal render path submits GPU work and returns without mapping texture memory. Diagnostic readback is an explicit separate operation that waits for completion and returns tightly packed RGBA8 pixels.

## Confirmed upstream limitations

The pinned stack is not a full browser engine. Material known limitations include overflow hit testing outside clips, specific `display: contents` inheritance behavior, inline-text opacity, incomplete advanced border and filter mapping, ignored backdrop filters, only the first node of a filter graph, unsupported custom backend paints, bounded image-atlas dimensions, no complete ICC/HDR/wide-gamut pipeline, incomplete emoji-presentation control, and incomplete bidi-aware editing commands.

Lavret does not rewrite source text, duplicate layout or shaping, post-process rendered pixels, or silently substitute another renderer to hide these limitations.

## Validation

Run the locked metadata/tree checks, `cargo fmt --all --check`, Clippy with warnings denied, the all-feature test suite, release build, and rustdoc with warnings denied exactly as recorded in `DEPENDENCIES.md`.
