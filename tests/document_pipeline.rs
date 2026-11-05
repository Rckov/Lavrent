use std::time::Duration;

use lavret_core::{Core, CoreError, CoreOptions, DocumentSource, HandleError, Point, Viewport};

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
