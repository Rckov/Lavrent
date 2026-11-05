# Third-Party Notices

`lavret-core` links the packages recorded in `Cargo.lock`. The following material components and license classes are part of the selected normal/build graph:

| Component | License |
|---|---|
| Blitz low-level crates, html5ever family, AnyRender, wgpu | MIT OR Apache-2.0 |
| Stylo, selectors, Stylo DOM/traits/atoms | MPL-2.0 |
| stylo_taffy | MIT OR Apache-2.0 OR MPL-2.0 |
| Taffy, HarfRust, wuff | MIT |
| Parley, Fontique, Fontations, Vello, Glifo, Peniko, Kurbo, color, usvg | Apache-2.0 OR MIT |
| ICU4X crates and compiled data | Unicode-3.0 |
| image | MIT OR Apache-2.0 |
| Windows Rust projections | MIT OR Apache-2.0 |
| DejaVu Sans WOFF2 test fixture | Bitstream Vera font license and public-domain Arev changes |

Stylo's MPL-2.0 obligations are file-level: modifications to MPL-covered upstream files must remain available under MPL-2.0 with their notices. Lavret does not modify or vendor those files. Linking Stylo does not apply MPL-2.0 to unrelated Lavret source files.

ICU4X code and compiled Unicode data are distributed under Unicode-3.0. Its copyright and permission notice must accompany distributions containing that data.

`tests/fixtures/DejaVuSans.woff2` is retained only as a font-loading integration fixture. It comes from the pinned Blitz source tree and remains under the DejaVu Fonts license derived from the Bitstream Vera font license; the fixture is not part of the Lavret library artifact.

The complete resolved package inventory, source revisions, and checksums are authoritative in `Cargo.lock`. Distribution packaging must include the corresponding upstream license texts and copyright notices. This inventory is technical information, not legal advice.
