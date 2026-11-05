use std::time::Duration;

use lavret_core::{
    ColorScheme, Core, CoreError, CoreOptions, DocumentSource, HandleError, Point, Viewport,
};

fn viewport() -> Viewport {
    Viewport::new(400.0, 300.0, 1.0)
}

fn source(html: &str, css: &str) -> DocumentSource {
    let mut source = DocumentSource::new(html);
    source.author_css.push(css.to_owned());
    source
}

#[test]
fn layout_pipeline_exposes_hand_derived_geometry() {
    let css = r#"
        html, body { margin: 0; }
        #block { position: absolute; left: 10px; top: 10px; width: 100px; height: 20px; }
        #flex { position: absolute; left: 10px; top: 40px; width: 120px; height: 20px; display: flex; }
        #flex-fixed { width: 30px; }
        #flex-fill { flex: 1; }
        #grid { position: absolute; left: 10px; top: 70px; width: 100px; height: 20px; display: grid; grid-template-columns: 40px 60px; }
        #table { position: absolute; left: 10px; top: 100px; width: 100px; height: 20px; display: table; table-layout: fixed; }
        #table-row { display: table-row; }
        .cell { display: table-cell; width: 50px; }
        #float-box { position: absolute; left: 10px; top: 130px; width: 100px; }
        #float { float: left; width: 40px; height: 20px; }
        #vertical { position: absolute; left: 10px; top: 170px; width: 40px; height: 80px; writing-mode: vertical-rl; }
        #vertical-child { width: 10px; height: 20px; }
    "#;
    let html = r#"
        <div id="block"></div>
        <div id="flex"><div id="flex-fixed"></div><div id="flex-fill"></div></div>
        <div id="grid"><div id="grid-first"></div><div></div></div>
        <div id="table"><div id="table-row"><div id="cell-a" class="cell"></div><div class="cell"></div></div></div>
        <div id="float-box"><div id="float"></div><span>flow</span></div>
        <div id="vertical"><div id="vertical-child"></div></div>
    "#;
    let mut core = Core::new(CoreOptions::default()).unwrap();
    let document = core.create_document(source(html, css), viewport()).unwrap();
    core.resolve(document, Duration::ZERO).unwrap();

    let block = core.find_element_by_id(document, "block").unwrap().unwrap();
    let flex_fixed = core
        .find_element_by_id(document, "flex-fixed")
        .unwrap()
        .unwrap();
    let flex_fill = core
        .find_element_by_id(document, "flex-fill")
        .unwrap()
        .unwrap();
    let grid_first = core
        .find_element_by_id(document, "grid-first")
        .unwrap()
        .unwrap();
    let cell = core
        .find_element_by_id(document, "cell-a")
        .unwrap()
        .unwrap();
    let float = core.find_element_by_id(document, "float").unwrap().unwrap();
    let vertical = core
        .find_element_by_id(document, "vertical")
        .unwrap()
        .unwrap();

    assert_eq!(
        core.border_box(document, block).unwrap(),
        lavret_core::Rect::new(10.0, 10.0, 100.0, 20.0)
    );
    assert_eq!(
        core.border_box(document, flex_fixed).unwrap().size.width,
        30.0
    );
    assert_eq!(
        core.border_box(document, flex_fill).unwrap().size.width,
        90.0
    );
    assert_eq!(
        core.border_box(document, grid_first).unwrap().size.width,
        40.0
    );
    assert_eq!(core.border_box(document, cell).unwrap().size.width, 50.0);
    assert_eq!(
        core.border_box(document, float).unwrap(),
        lavret_core::Rect::new(10.0, 130.0, 40.0, 20.0)
    );
    assert_eq!(
        core.border_box(document, vertical).unwrap().size,
        lavret_core::Size::new(40.0, 80.0)
    );
}

#[test]
fn mutations_are_batched_until_explicit_resolve() {
    let css =
        "html,body{margin:0}.wide{width:120px}#host{height:20px}#child{width:40px;height:10px}";
    let html = "<div id='host'><div id='child'></div></div><div id='other'></div>";
    let mut core = Core::new(CoreOptions::default()).unwrap();
    let document = core.create_document(source(html, css), viewport()).unwrap();
    core.resolve(document, Duration::ZERO).unwrap();
    let host = core.find_element_by_id(document, "host").unwrap().unwrap();
    let child = core.find_element_by_id(document, "child").unwrap().unwrap();
    let other = core.find_element_by_id(document, "other").unwrap().unwrap();

    core.set_attribute(document, host, "class", "wide").unwrap();
    assert_eq!(
        core.border_box(document, host),
        Err(CoreError::UnresolvedDocument)
    );
    core.resolve(document, Duration::ZERO).unwrap();
    assert_eq!(core.border_box(document, host).unwrap().size.width, 120.0);
    core.remove_attribute(document, host, "class").unwrap();
    core.resolve(document, Duration::ZERO).unwrap();
    assert_eq!(core.border_box(document, host).unwrap().size.width, 400.0);

    let inserted = core.create_element(document, "div").unwrap();
    core.set_attribute(document, inserted, "class", "wide")
        .unwrap();
    let inserted_text = core.create_text(document, "inserted").unwrap();
    core.append_node(document, inserted, inserted_text).unwrap();
    core.append_node(document, host, inserted).unwrap();
    let leading = core.create_element(document, "span").unwrap();
    let leading_text = core.create_text(document, "leading").unwrap();
    core.append_node(document, leading, leading_text).unwrap();
    core.insert_node_before(document, inserted, leading)
        .unwrap();
    core.resolve(document, Duration::ZERO).unwrap();
    assert_eq!(
        core.border_box(document, inserted).unwrap().size.width,
        120.0
    );
    assert!(core.border_box(document, leading).is_ok());

    core.move_node(document, child, other, None).unwrap();
    core.replace_text_content(document, child, "updated")
        .unwrap();
    core.resolve(document, Duration::ZERO).unwrap();
    assert!(core.border_box(document, child).is_ok());

    core.remove_node(document, child).unwrap();
    assert_eq!(
        core.set_attribute(document, child, "class", "gone"),
        Err(CoreError::Handle(HandleError::Destroyed))
    );
}

#[test]
fn viewport_values_must_fit_the_document_engine() {
    let mut core = Core::new(CoreOptions::default()).unwrap();
    for viewport in [
        Viewport::new(0.0, 100.0, 1.0),
        Viewport::new(f64::NAN, 100.0, 1.0),
        Viewport::new(100.0, f64::INFINITY, 1.0),
        Viewport::new(100.0, 100.0, f64::MIN_POSITIVE),
    ] {
        assert!(matches!(
            core.create_document(DocumentSource::new("<div></div>"), viewport),
            Err(CoreError::InvalidArgument { .. })
        ));
    }
}

#[test]
fn viewport_color_scheme_updates_media_queries() {
    let css = "#box{width:40px;height:10px}@media(prefers-color-scheme:dark){#box{width:80px}}";
    let mut core = Core::new(CoreOptions::default()).unwrap();
    let document = core
        .create_document(source("<div id='box'></div>", css), viewport())
        .unwrap();
    core.resolve(document, Duration::ZERO).unwrap();
    let box_node = core.find_element_by_id(document, "box").unwrap().unwrap();
    assert_eq!(
        core.border_box(document, box_node).unwrap().size.width,
        40.0
    );

    core.set_viewport(document, viewport().with_color_scheme(ColorScheme::Dark))
        .unwrap();
    core.resolve(document, Duration::ZERO).unwrap();
    assert_eq!(
        core.border_box(document, box_node).unwrap().size.width,
        80.0
    );
}

#[test]
fn handles_enforce_owner_document_and_generation() {
    let mut first = Core::new(CoreOptions::default()).unwrap();
    let mut second = Core::new(CoreOptions::default()).unwrap();
    let first_document = first
        .create_document(DocumentSource::new("<div id='node'></div>"), viewport())
        .unwrap();
    let second_document = first
        .create_document(DocumentSource::new("<div></div>"), viewport())
        .unwrap();
    let foreign_document = second
        .create_document(DocumentSource::new("<div></div>"), viewport())
        .unwrap();
    let node = first
        .find_element_by_id(first_document, "node")
        .unwrap()
        .unwrap();

    assert_eq!(
        first.find_element_by_id(foreign_document, "node"),
        Err(CoreError::Handle(HandleError::Foreign))
    );
    assert_eq!(
        first.set_attribute(second_document, node, "class", "x"),
        Err(CoreError::Handle(HandleError::CrossDocument))
    );

    first
        .replace_document_source(first_document, DocumentSource::new("<p id='next'></p>"))
        .unwrap();
    assert_eq!(
        first.set_attribute(first_document, node, "class", "x"),
        Err(CoreError::Handle(HandleError::Stale))
    );

    first.destroy_document(first_document).unwrap();
    assert_eq!(
        first.resolve(first_document, Duration::ZERO),
        Err(CoreError::Handle(HandleError::Destroyed))
    );
    let replacement = first
        .create_document(DocumentSource::new("<div></div>"), viewport())
        .unwrap();
    assert_ne!(replacement, first_document);
    assert_eq!(
        first.resolve(first_document, Duration::ZERO),
        Err(CoreError::Handle(HandleError::Stale))
    );
}

#[test]
fn handles_from_a_dropped_core_remain_foreign_to_later_cores() {
    let old_document = {
        let mut core = Core::new(CoreOptions::default()).unwrap();
        core.create_document(DocumentSource::new("<div></div>"), viewport())
            .unwrap()
    };
    let mut later = Core::new(CoreOptions::default()).unwrap();
    let _ = later
        .create_document(DocumentSource::new("<div></div>"), viewport())
        .unwrap();
    assert_eq!(
        later.resolve(old_document, Duration::ZERO),
        Err(CoreError::Handle(HandleError::Foreign))
    );
}

#[test]
fn hit_testing_returns_lavret_handles_in_topmost_order() {
    let css = "html,body{margin:0}#back,#front{position:absolute;left:10px;top:10px;width:100px;height:100px}#front{left:20px;top:20px;width:40px;height:40px}";
    let html = "<div id='back'><div id='front'></div></div>";
    let mut core = Core::new(CoreOptions::default()).unwrap();
    let document = core.create_document(source(html, css), viewport()).unwrap();
    core.resolve(document, Duration::ZERO).unwrap();
    let front = core.find_element_by_id(document, "front").unwrap().unwrap();
    let back = core.find_element_by_id(document, "back").unwrap().unwrap();
    let hits = core.hit_test(document, Point::new(30.0, 30.0)).unwrap();
    assert_eq!(&hits[..2], &[front, back]);
}

#[test]
fn text_stack_lays_out_bidi_combining_fallback_and_variation_content() {
    let css = r#"
        html,body{margin:0}
        #mixed{width:150px;font:24px "Segoe UI Variable Text","Segoe UI Emoji",sans-serif;font-variation-settings:"wght" 650;line-height:1.2}
        #vertical-text{position:absolute;left:200px;top:0;height:120px;writing-mode:vertical-rl;font:22px sans-serif}
    "#;
    let html = r#"
        <div id="mixed" dir="auto">Lavret العربية e&#x301; 😀 Latin العربية</div>
        <div id="vertical-text">縦書き مرحبا</div>
    "#;
    let mut core = Core::new(CoreOptions::default()).unwrap();
    let document = core.create_document(source(html, css), viewport()).unwrap();
    core.resolve(document, Duration::ZERO).unwrap();
    let mixed = core.find_element_by_id(document, "mixed").unwrap().unwrap();
    let vertical = core
        .find_element_by_id(document, "vertical-text")
        .unwrap()
        .unwrap();
    let mixed_box = core.border_box(document, mixed).unwrap();
    let vertical_box = core.border_box(document, vertical).unwrap();
    assert_eq!(mixed_box.size.width, 150.0);
    assert!(mixed_box.size.height > 28.0);
    assert!(vertical_box.size.width > 0.0);
    assert!(
        core.hit_test(document, Point::new(10.0, 10.0))
            .unwrap()
            .contains(&mixed)
    );
}
