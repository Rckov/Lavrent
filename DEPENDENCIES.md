# Dependencies

## Qualified toolchain and graph

The dependency graph was qualified on 2026-11-05 with:

- Rust `1.91.0` commit `f8297e351a40c1439a467bbbb6879088047f50b3`;
- Cargo `1.91.0` commit `ea2d97820c16195b0ca3fadb4319fe512c199a43`;
- Rustup `1.29.3`, minimal profile, with rustfmt and Clippy;
- host and validation target `x86_64-pc-windows-msvc`;
- LLVM `21.1.2`.

The locked metadata contains the root crate plus 374 dependency packages. Material selected package IDs are:

| Package | Resolved version/source |
|---|---|
| Blitz low-level crates | `0.3.0-beta.4`, `git+https://github.com/DioxusLabs/blitz?rev=3a5fdb3874ff52acac6f2eecedfbb13886f77194#3a5fdb3874ff52acac6f2eecedfbb13886f77194` |
| Stylo / selectors | `0.23.0` / `0.42.0`, `git+https://github.com/servo/stylo?rev=2dca34592ced37c6aa01733e94206a767194cf9d#2dca34592ced37c6aa01733e94206a767194cf9d` |
| Taffy | `0.15.0`, `git+https://github.com/DioxusLabs/taffy?rev=fca167cf659b8bf48019d0dea90624109a78103f#fca167cf659b8bf48019d0dea90624109a78103f` |
| Parley / Fontique / HarfRust | `0.12.0` / `0.12.0` / `0.12.0`, crates.io registry |
| Skrifa / read-fonts | `0.44.0` / `0.41.0`, crates.io registry |
| AnyRender / AnyRender SVG / Vello hybrid | `0.14.0` / `0.15.0` / `0.11.0`, `git+https://github.com/DioxusLabs/anyrender?branch=main#ccb524eac2d738ec56e33d40ff89f435a0ead16d` |
| Vello GPU / common / Glifo | `0.3.0` / `0.3.0` / `0.4.0`, `git+https://github.com/DioxusLabs/vello?rev=ccdad69675882f5b75a2a365027c6febabacb5cc#ccdad69675882f5b75a2a365027c6febabacb5cc` |
| wgpu | `30.0.1`, crates.io registry |
| image / usvg / wuff | `0.25.10` / `0.48.1` / `0.2.9`, crates.io registry |

Direct feature resolution is limited to the concrete interfaces named by Lavret:

- `blitz-dom`: `system-fonts`, `complex-scripts`, `svg`, `woff`, `text-transform-icu`, `floats`, `writing-mode`, and `scrollbars`;
- `blitz-paint`: `svg` and `scrollbars`;
- `blitz-html` and `blitz-traits`: no optional features;
- `anyrender_vello_hybrid`: no optional features, so `webgl` and frame-time logging are disabled;
- `vello_gpu`: the selected wgpu/text path; no WebGL feature;
- `keyboard-types`, `pollster`, `url`, `wgpu`, and `wgpu_context`: only because Lavret names their input, synchronous initialization, URI, GPU, or device-context interfaces.

Blitz identifies AnyRender transitively by `branch=main`. Cargo treats the branch and an exact `rev` from the same canonical Git repository as different source identities, while also rejecting a same-source patch from that branch identity to the revision. Using a direct revision would therefore create incompatible duplicate AnyRender types. The manifest retains the required branch identity, and `Cargo.lock` fixes that identity to `ccb524eac2d738ec56e33d40ff89f435a0ead16d`. Every production and validation command uses `--locked`, so Cargo cannot follow a later branch head. No other Git dependency is branch-based.

The normal/build graph has one AnyRender source, one production Vello GPU renderer, and only wgpu `30.0.1`. It contains no Dioxus application crate or `dioxus-native`, `blitz-shell`, `blitz-net`, Winit, Muda, file-dialog crate, Tokio, network or TLS client, JavaScript engine, Servo browser crate, Skia, Vello CPU, compute-centric `vello`, `pixels`, wgpu 29, or second renderer. Value-only `http`, `url`, and `bytes` packages introduce no network implementation.

## Qualification

Locked metadata resolution, both dependency-tree inspections, and `cargo check --locked --target x86_64-pc-windows-msvc` passed. The release smoke test parsed in-memory HTML with inline and author CSS, resolved block, Flexbox, Grid, table, float, writing-mode, and mixed Latin/Arabic layout, queried geometry and hit testing, built a Blitz Paint/AnyRender scene, and rendered without a window.

The selected adapter was `NVIDIA GeForce RTX 5070`, backend `Vulkan`, device type `DiscreteGpu`, driver `NVIDIA`. Vello GPU rendered a 256 by 256 `Rgba8Unorm` target. Explicit readback returned 262,144 tightly packed RGBA8 bytes and confirmed an interior red pixel. The normal render operation submitted without creating, mapping, or reading a staging buffer.

## Validation commands

Run from the crate root with the committed lockfile:

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
