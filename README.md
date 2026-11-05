# lavret-core

`lavret-core` is the in-memory document engine for Lavret. It owns document lifecycle, opaque handles, normalized input, diagnostics, and offscreen GPU rendering while keeping Blitz, Stylo, Taffy, Parley, AnyRender, Vello, and wgpu private.

The crate targets Rust 1.91.0 and edition 2024. Its current scope excludes language bindings, native windows, networking, JavaScript, application frameworks, samples, and alternate renderers.

Validation commands are listed in [`DEPENDENCIES.md`](DEPENDENCIES.md#validation-commands).
