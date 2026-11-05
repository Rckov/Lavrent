use std::time::Duration;

use lavret_core::{
    Core, CoreError, CoreOptions, DocumentSource, InputEvent, Key, KeyLocation, Modifiers,
    NamedKey, PhysicalKey, Point, PointerButton, PointerButtons, PointerId, PointerKind,
    UiEventData, UiEventKind, Viewport,
};

fn mouse_down(position: Point) -> InputEvent {
    InputEvent::PointerDown {
        id: PointerId(1),
        kind: PointerKind::Mouse,
        is_primary: true,
        position,
        button: PointerButton::Primary,
        buttons: PointerButtons {
            primary: true,
            ..PointerButtons::default()
        },
        modifiers: Modifiers::default(),
    }
}

fn mouse_up(position: Point) -> InputEvent {
    InputEvent::PointerUp {
        id: PointerId(1),
        kind: PointerKind::Mouse,
        is_primary: true,
        position,
        button: PointerButton::Primary,
        buttons: PointerButtons::default(),
        modifiers: Modifiers::default(),
    }
}

fn setup() -> (Core, lavret_core::DocumentHandle) {
    let mut source = DocumentSource::new(
        "<input id='editor'><button id='button'>Go</button><div id='overflow'><div></div></div>",
    );
    source.author_css.push(
        "html,body{margin:0}#editor{position:absolute;left:10px;top:10px;width:120px;height:30px}#button{position:absolute;left:10px;top:50px;width:80px;height:30px}#overflow{position:absolute;left:160px;top:10px;width:80px;height:40px;overflow:auto}#overflow>div{height:160px}"
            .to_owned(),
    );
    let mut core = Core::new(CoreOptions::default()).unwrap();
    let document = core
        .create_document(source, Viewport::new(300.0, 200.0, 1.0))
        .unwrap();
    core.resolve(document, Duration::ZERO).unwrap();
    (core, document)
}

#[test]
fn pointer_focus_click_and_text_input_return_semantic_events() {
    let (mut core, document) = setup();
    let editor = core
        .find_element_by_id(document, "editor")
        .unwrap()
        .unwrap();
    let position = Point::new(20.0, 20.0);

    core.dispatch_input(document, mouse_down(position)).unwrap();
    core.dispatch_input(document, mouse_up(position)).unwrap();
    let events = core.drain_ui_events().unwrap();
    assert!(
        events
            .iter()
            .any(|event| event.target == editor && event.kind == UiEventKind::Focus)
    );
    assert!(
        events
            .iter()
            .any(|event| event.target == editor && event.kind == UiEventKind::Click)
    );

    core.dispatch_input(
        document,
        InputEvent::TextInput {
            text: "A".to_owned(),
        },
    )
    .unwrap();
    let events = core.drain_ui_events().unwrap();
    assert!(events.iter().any(|event| {
        event.target == editor
            && event.kind == UiEventKind::Input
            && event.data == UiEventData::Text("A".to_owned())
    }));
    assert_eq!(
        core.border_box(document, editor),
        Err(CoreError::UnresolvedDocument)
    );
    core.resolve(document, Duration::ZERO).unwrap();
    assert!(core.border_box(document, editor).is_ok());
}

#[test]
fn keyboard_navigation_and_ime_use_blitz_editing_state() {
    let (mut core, document) = setup();
    let editor = core
        .find_element_by_id(document, "editor")
        .unwrap()
        .unwrap();
    core.dispatch_input(document, mouse_down(Point::new(20.0, 20.0)))
        .unwrap();
    core.dispatch_input(document, mouse_up(Point::new(20.0, 20.0)))
        .unwrap();
    core.drain_ui_events().unwrap();

    core.dispatch_input(
        document,
        InputEvent::ImePreedit {
            text: "Ж".to_owned(),
            selection: Some((0, "Ж".len())),
        },
    )
    .unwrap();
    core.dispatch_input(
        document,
        InputEvent::ImeCommit {
            text: "Ж".to_owned(),
        },
    )
    .unwrap();
    let events = core.drain_ui_events().unwrap();
    assert!(events.iter().any(|event| {
        event.target == editor
            && event.kind == UiEventKind::Input
            && event.data == UiEventData::Text("Ж".to_owned())
    }));

    core.dispatch_input(
        document,
        InputEvent::KeyDown {
            key: Key::Named(NamedKey::Tab),
            physical_key: PhysicalKey::Code("Tab".to_owned()),
            location: KeyLocation::Standard,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    )
    .unwrap();
    let button = core
        .find_element_by_id(document, "button")
        .unwrap()
        .unwrap();
    let events = core.drain_ui_events().unwrap();
    assert!(
        events
            .iter()
            .any(|event| event.target == editor && event.kind == UiEventKind::Blur)
    );
    assert!(
        events
            .iter()
            .any(|event| event.target == button && event.kind == UiEventKind::Focus)
    );
}

#[test]
fn wheel_input_scrolls_through_the_upstream_event_driver() {
    let (mut core, document) = setup();
    core.dispatch_input(
        document,
        InputEvent::PointerMove {
            id: PointerId(1),
            kind: PointerKind::Mouse,
            is_primary: true,
            position: Point::new(170.0, 20.0),
            buttons: PointerButtons::default(),
            modifiers: Modifiers::default(),
        },
    )
    .unwrap();
    core.dispatch_input(
        document,
        InputEvent::Wheel {
            position: Point::new(170.0, 20.0),
            delta_x: 0.0,
            delta_y: 30.0,
            modifiers: Modifiers::default(),
        },
    )
    .unwrap();
    assert!(
        core.drain_ui_events()
            .unwrap()
            .iter()
            .all(|event| event.kind != UiEventKind::Click)
    );
    core.resolve(document, Duration::ZERO).unwrap();
}
