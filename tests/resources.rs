use std::time::Duration;

use lavret_core::{
    Core, CoreError, CoreOptions, DiagnosticCode, DocumentSource, ResourceKind, ResourceSource,
    Viewport,
};

const PNG_2X2: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 2, 0, 0, 0, 2, 8, 6, 0,
    0, 0, 114, 182, 13, 36, 0, 0, 0, 1, 115, 82, 71, 66, 0, 174, 206, 28, 233, 0, 0, 0, 4, 103, 65,
    77, 65, 0, 0, 177, 143, 11, 252, 97, 5, 0, 0, 0, 9, 112, 72, 89, 115, 0, 0, 14, 195, 0, 0, 14,
    195, 1, 199, 111, 168, 100, 0, 0, 0, 17, 73, 68, 65, 84, 24, 87, 99, 248, 207, 192, 240, 31,
    132, 25, 96, 12, 0, 71, 202, 7, 249, 60, 228, 9, 10, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96,
    130,
];

fn resource(uri: &str, media_type: &str, kind: ResourceKind, bytes: &[u8]) -> ResourceSource {
    ResourceSource::new(uri, media_type, kind, bytes)
}

#[test]
fn registered_stylesheet_raster_svg_and_font_are_loaded_in_memory() {
    let mut core = Core::new(CoreOptions::default()).unwrap();
    core.register_resource(resource(
        "lavret:/style.css",
        "text/css",
        ResourceKind::AuthorStylesheet,
        b"html,body{margin:0}#styled{width:73px;height:11px}",
    ))
    .unwrap();
    core.register_resource(resource(
        "lavret:/pixel.png",
        "image/png",
        ResourceKind::RasterImage,
        PNG_2X2,
    ))
    .unwrap();
    core.register_resource(resource(
        "lavret:/shape.svg",
        "image/svg+xml",
        ResourceKind::SvgDocument,
        br#"<svg xmlns="http://www.w3.org/2000/svg" width="12" height="8"><rect width="12" height="8" fill="red"/></svg>"#,
    ))
    .unwrap();
    core.register_resource(resource(
        "lavret:/font.woff2",
        "font/woff2",
        ResourceKind::Font,
        include_bytes!("fixtures/DejaVuSans.woff2"),
    ))
    .unwrap();

    let html = r#"
        <link rel="stylesheet" href="lavret:/style.css">
        <style>@font-face{font-family:Fixture;src:url('lavret:/font.woff2') format('woff2')}#text{font:20px Fixture}</style>
        <div id="styled"></div>
        <img id="raster" src="lavret:/pixel.png">
        <img id="svg" src="lavret:/shape.svg">
        <div id="text">Lavret مرحبا</div>
    "#;
    let document = core
        .create_document(DocumentSource::new(html), Viewport::new(300.0, 200.0, 1.0))
        .unwrap();
    core.resolve(document, Duration::ZERO).unwrap();

    let styled = core
        .find_element_by_id(document, "styled")
        .unwrap()
        .unwrap();
    let raster = core
        .find_element_by_id(document, "raster")
        .unwrap()
        .unwrap();
    let svg = core.find_element_by_id(document, "svg").unwrap().unwrap();
    let text = core.find_element_by_id(document, "text").unwrap().unwrap();
    assert_eq!(core.border_box(document, styled).unwrap().size.width, 73.0);
    assert_eq!(core.border_box(document, raster).unwrap().size.width, 2.0);
    assert_eq!(core.border_box(document, svg).unwrap().size.width, 12.0);
    assert!(core.border_box(document, text).unwrap().size.height > 20.0);
    assert!(core.drain_diagnostics().unwrap().is_empty());
}

#[test]
fn replacing_and_removing_resources_invalidate_documents() {
    let mut core = Core::new(CoreOptions::default()).unwrap();
    let stylesheet = core
        .register_resource(resource(
            "lavret:/style.css",
            "text/css",
            ResourceKind::AuthorStylesheet,
            b"#box{width:40px;height:10px}",
        ))
        .unwrap();
    let document = core
        .create_document(
            DocumentSource::new(
                "<link rel='stylesheet' href='lavret:/style.css'><div id='box'></div>",
            ),
            Viewport::new(200.0, 100.0, 1.0),
        )
        .unwrap();
    core.resolve(document, Duration::ZERO).unwrap();
    let box_node = core.find_element_by_id(document, "box").unwrap().unwrap();
    assert_eq!(
        core.border_box(document, box_node).unwrap().size.width,
        40.0
    );

    core.replace_resource(
        stylesheet,
        resource(
            "lavret:/style.css",
            "text/css",
            ResourceKind::AuthorStylesheet,
            b"#box{width:90px;height:10px}",
        ),
    )
    .unwrap();
    assert_eq!(
        core.border_box(document, box_node),
        Err(CoreError::UnresolvedDocument)
    );
    core.resolve(document, Duration::ZERO).unwrap();
    assert_eq!(
        core.border_box(document, box_node).unwrap().size.width,
        90.0
    );

    core.remove_resource(stylesheet).unwrap();
    core.resolve(document, Duration::ZERO).unwrap();
    let diagnostics = core.drain_diagnostics().unwrap();
    assert!(
        diagnostics
            .iter()
            .any(|item| item.code == DiagnosticCode::MissingResource)
    );
}

#[test]
fn external_and_filesystem_resource_schemes_are_rejected() {
    let mut core = Core::new(CoreOptions::default()).unwrap();
    for uri in ["https://example.com/a.css", "file:///C:/a.css", "C:/a.css"] {
        assert!(matches!(
            core.register_resource(resource(
                uri,
                "text/css",
                ResourceKind::AuthorStylesheet,
                b""
            )),
            Err(CoreError::UnsupportedResourceScheme { .. })
        ));
    }
}
