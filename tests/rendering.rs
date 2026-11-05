use std::time::Duration;

use lavret_core::{
    Core, CoreError, CoreOptions, DiagnosticCode, DocumentSource, HandleError, ResourceKind,
    ResourceSource, Rgba8Readback, Viewport,
};

const WIDTH: u32 = 256;
const HEIGHT: u32 = 160;
const PNG_2X2: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 2, 0, 0, 0, 2, 8, 6, 0,
    0, 0, 114, 182, 13, 36, 0, 0, 0, 1, 115, 82, 71, 66, 0, 174, 206, 28, 233, 0, 0, 0, 4, 103, 65,
    77, 65, 0, 0, 177, 143, 11, 252, 97, 5, 0, 0, 0, 9, 112, 72, 89, 115, 0, 0, 14, 195, 0, 0, 14,
    195, 1, 199, 111, 168, 100, 0, 0, 0, 17, 73, 68, 65, 84, 24, 87, 99, 248, 207, 192, 240, 31,
    132, 25, 96, 12, 0, 71, 202, 7, 249, 60, 228, 9, 10, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96,
    130,
];

#[test]
fn offscreen_render_and_explicit_readback_cover_paint_features() {
    let mut core = Core::new(CoreOptions::default()).unwrap();
    core.register_resource(ResourceSource::new(
        "lavret:/pixel.png",
        "image/png",
        ResourceKind::RasterImage,
        PNG_2X2,
    ))
    .unwrap();
    core.register_resource(ResourceSource::new(
        "lavret:/shape.svg",
        "image/svg+xml",
        ResourceKind::SvgDocument,
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20"><rect width="20" height="20" fill="#00ffff"/></svg>"##,
    ))
    .unwrap();

    let html = r#"
        <div id="solid"></div><div id="border"></div>
        <img id="raster" src="lavret:/pixel.png"><img id="svg" src="lavret:/shape.svg">
        <div id="gradient"></div><div id="opacity"></div><div id="transform"></div>
        <div id="clip"><div></div></div><div id="shadow"></div><div id="text">Lavret 😀 مرحبا e&#x301;</div>
    "#;
    let css = r#"
        html,body{margin:0;width:256px;height:160px;background:rgb(10,20,30)}
        #solid{position:absolute;left:8px;top:8px;width:20px;height:20px;background:rgb(255,0,0)}
        #border{position:absolute;left:40px;top:8px;width:22px;height:22px;border:4px solid rgb(0,0,255);background:rgb(0,255,0)}
        #raster{position:absolute;left:80px;top:8px;width:20px;height:20px;image-rendering:pixelated}
        #svg{position:absolute;left:110px;top:8px;width:20px;height:20px}
        #gradient{position:absolute;left:140px;top:8px;width:40px;height:20px;background:linear-gradient(to right,rgb(255,0,0),rgb(0,0,255))}
        #opacity{position:absolute;left:190px;top:8px;width:20px;height:20px;background:rgb(255,0,0);opacity:.5}
        #transform{position:absolute;left:8px;top:50px;width:20px;height:20px;background:rgb(255,255,0);transform:translateX(20px)}
        #clip{position:absolute;left:60px;top:50px;width:20px;height:20px;overflow:hidden}
        #clip>div{width:40px;height:40px;background:rgb(128,0,128)}
        #shadow{position:absolute;left:100px;top:50px;width:20px;height:20px;background:white;box-shadow:8px 0 0 rgb(255,0,0)}
        #text{position:absolute;left:8px;top:92px;width:230px;height:40px;color:white;font:28px "Segoe UI Emoji",sans-serif}
    "#;
    let mut source = DocumentSource::new(html);
    source.author_css.push(css.to_owned());
    let document = core
        .create_document(
            source,
            Viewport::new(f64::from(WIDTH), f64::from(HEIGHT), 1.0),
        )
        .unwrap();
    core.resolve(document, Duration::ZERO).unwrap();
    let target = core.create_render_target(WIDTH, HEIGHT).unwrap();
    let diagnostics = core.drain_diagnostics().unwrap();
    let renderer = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == DiagnosticCode::RendererInitialized)
        .unwrap();
    for key in [
        "adapter",
        "backend",
        "device_type",
        "max_texture_dimension_2d",
        "max_buffer_size",
        "image_atlas_size",
        "image_atlas_max_count",
    ] {
        assert!(renderer.context.iter().any(|entry| entry.key == key));
    }

    let result = core.render(document, target).unwrap();
    assert_eq!((result.width, result.height), (WIDTH, HEIGHT));
    let readback = core.readback_rgba8(target).unwrap();
    assert_eq!((readback.width, readback.height), (WIDTH, HEIGHT));
    assert_eq!(readback.pixels.len(), (WIDTH * HEIGHT * 4) as usize);

    assert_color(pixel(&readback, 12, 12), [255, 0, 0, 255], 1);
    assert_color(pixel(&readback, 41, 12), [0, 0, 255, 255], 1);
    assert_color(pixel(&readback, 48, 16), [0, 255, 0, 255], 1);
    assert_ne!(pixel(&readback, 88, 16), [10, 20, 30, 255]);
    assert_color(pixel(&readback, 118, 16), [0, 255, 255, 255], 1);
    let gradient_left = pixel(&readback, 145, 16);
    let gradient_right = pixel(&readback, 175, 16);
    assert!(gradient_left[0] > gradient_left[2]);
    assert!(gradient_right[2] > gradient_right[0]);
    assert_color(pixel(&readback, 198, 16), [133, 10, 15, 255], 3);
    assert_color(pixel(&readback, 32, 58), [255, 255, 0, 255], 1);
    assert_color(pixel(&readback, 68, 58), [128, 0, 128, 255], 1);
    assert_color(pixel(&readback, 84, 58), [10, 20, 30, 255], 1);
    assert_color(pixel(&readback, 124, 58), [255, 0, 0, 255], 1);

    let text_pixels = (96..128)
        .flat_map(|y| (8..238).map(move |x| (x, y)))
        .filter(|&(x, y)| pixel(&readback, x, y) != [10, 20, 30, 255])
        .count();
    assert!(text_pixels > 100);
}

#[test]
fn render_requires_resolution_and_readback_requires_a_frame() {
    let mut core = Core::new(CoreOptions::default()).unwrap();
    let document = core
        .create_document(
            DocumentSource::new("<div></div>"),
            Viewport::new(16.0, 16.0, 1.0),
        )
        .unwrap();
    let target = core.create_render_target(16, 16).unwrap();
    assert_eq!(
        core.render(document, target),
        Err(CoreError::UnresolvedDocument)
    );
    assert!(matches!(
        core.readback_rgba8(target),
        Err(CoreError::GpuReadback { .. })
    ));
}

#[test]
fn render_targets_enforce_dimensions_owner_and_generation() {
    let mut first = Core::new(CoreOptions::default()).unwrap();
    let mut second = Core::new(CoreOptions::default()).unwrap();
    assert!(matches!(
        first.create_render_target(0, 16),
        Err(CoreError::InvalidArgument { .. })
    ));
    assert!(matches!(
        first.create_render_target(u32::from(u16::MAX) + 1, 16),
        Err(CoreError::ResourceLimit { .. })
    ));

    let target = first.create_render_target(16, 16).unwrap();
    assert_eq!(
        second.resize_render_target(target, 32, 32),
        Err(CoreError::Handle(HandleError::Foreign))
    );
    first.destroy_render_target(target).unwrap();
    assert_eq!(
        first.resize_render_target(target, 32, 32),
        Err(CoreError::Handle(HandleError::Destroyed))
    );
    let replacement = first.create_render_target(16, 16).unwrap();
    assert_ne!(replacement, target);
    assert_eq!(
        first.resize_render_target(target, 32, 32),
        Err(CoreError::Handle(HandleError::Stale))
    );
}

fn pixel(readback: &Rgba8Readback, x: u32, y: u32) -> [u8; 4] {
    let index = ((y * readback.width + x) * 4) as usize;
    readback.pixels[index..index + 4].try_into().unwrap()
}

fn assert_color(actual: [u8; 4], expected: [u8; 4], tolerance: u8) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!(
            actual.abs_diff(expected) <= tolerance,
            "{actual} != {expected}"
        );
    }
}
