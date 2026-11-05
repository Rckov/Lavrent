# Dependencies

## Qualified toolchain and graph

The dependency qualification gate ran on 2026-11-05 with:

- Rust `1.91.0` (`f8297e351a40c1439a467bbbb6879088047f50b3`);
- Cargo `1.91.0` (`ea2d97820c16195b0ca3fadb4319fe512c199a43`);
- host `x86_64-pc-windows-msvc`;
- Blitz `3a5fdb3874ff52acac6f2eecedfbb13886f77194`;
- Stylo `2dca34592ced37c6aa01733e94206a767194cf9d`;
- Taffy `fca167cf659b8bf48019d0dea90624109a78103f`;
- AnyRender `ccb524eac2d738ec56e33d40ff89f435a0ead16d`;
- Dioxus Vello `ccdad69675882f5b75a2a365027c6febabacb5cc`;
- wgpu `30.0.1`.

Blitz names AnyRender's `main` branch transitively. Cargo treats `branch` and `rev` references from one Git repository as distinct source identities and rejects a same-source patch. The manifest therefore uses the same branch identity as Blitz and commits its exact resolved commit in `Cargo.lock`. Every validation and production command uses `--locked`; Cargo consequently fetches `ccb524eac2d738ec56e33d40ff89f435a0ead16d` and does not follow the branch head.

The normal/build graph contains one AnyRender source, one Vello GPU renderer, and wgpu 30.0.1. It contains no Dioxus application crate, `blitz-shell`, `blitz-net`, window renderer, Tokio, network client, TLS client, JavaScript engine, Servo browser crate, Skia, Vello CPU, compute-centric Vello, `pixels`, or wgpu 29.

## Qualification result

`cargo metadata --format-version 1 --locked`, both required tree commands, and `cargo check --locked --target x86_64-pc-windows-msvc` passed. The release smoke parsed in-memory HTML with inline and separate author CSS; resolved block, Flexbox, Grid, table, float, writing-mode, and mixed Latin/Arabic text; queried geometry and hit testing; built a Blitz Paint/AnyRender scene; and rendered without a window.

The selected adapter was `NVIDIA GeForce RTX 5070`, backend `Vulkan`, device type `DiscreteGpu`, driver `NVIDIA`. Vello GPU rendered a 256 by 256 `Rgba8Unorm` target. Explicit readback returned 262,144 tightly packed RGBA8 bytes and confirmed an interior red pixel. The normal render operation did not perform readback.

## Validation commands

```powershell
cargo metadata --format-version 1 --locked
cargo tree --locked -e normal,build
cargo tree --locked --all-features
cargo fmt --all --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features --no-fail-fast
cargo build --locked --release
$env:RUSTDOCFLAGS = '-D warnings'
cargo doc --locked --no-deps --all-features
Remove-Item Env:RUSTDOCFLAGS
```
