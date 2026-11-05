use std::str::FromStr;
use std::sync::Arc;

use blitz_dom::{Document, EventHandler};
use blitz_traits::NodeId;
use blitz_traits::events::{
    BlitzImeEvent, BlitzKeyEvent, BlitzPointerEvent, BlitzPointerId, BlitzWheelDelta,
    BlitzWheelEvent, DomEvent, DomEventData, EventState, KeyState, MouseEventButton,
    MouseEventButtons, Point as BlitzPoint, PointerCoords, PointerDetails, UiEvent as BlitzUiEvent,
};
use keyboard_types::{
    Code, Key as BlitzKey, Location, Modifiers as BlitzModifiers, NamedKey as BlitzNamedKey,
};

use crate::{
    CoreError, InputEvent, Key, KeyLocation, Modifiers, NamedKey, PhysicalKey, Point,
    PointerButton, PointerButtons, PointerId, PointerKind, UiEventData, UiEventKind,
};

pub(crate) struct SemanticEvent {
    pub(crate) target: NodeId,
    pub(crate) kind: UiEventKind,
    pub(crate) data: UiEventData,
}

pub(crate) struct SemanticCapture<'events> {
    events: &'events mut Vec<SemanticEvent>,
}

impl<'events> SemanticCapture<'events> {
    pub(crate) fn new(events: &'events mut Vec<SemanticEvent>) -> Self {
        Self { events }
    }
}

impl EventHandler for SemanticCapture<'_> {
    fn handle_event(
        &mut self,
        _chain: &[NodeId],
        event: &mut DomEvent,
        _document: &mut dyn Document,
        _event_state: &mut EventState,
    ) {
        let semantic = match &event.data {
            DomEventData::Click(_) => Some((UiEventKind::Click, UiEventData::None)),
            DomEventData::Input(data) => {
                Some((UiEventKind::Input, UiEventData::Text(data.value.clone())))
            }
            DomEventData::Focus(_) => Some((UiEventKind::Focus, UiEventData::None)),
            DomEventData::Blur(_) => Some((UiEventKind::Blur, UiEventData::None)),
            _ => None,
        };
        if let Some((kind, data)) = semantic {
            self.events.push(SemanticEvent {
                target: event.target,
                kind,
                data,
            });
        }
    }
}

pub(crate) enum Dispatch {
    Upstream(BlitzUiEvent),
    FocusGained,
    FocusLost,
}

struct PointerIdentity {
    id: PointerId,
    kind: PointerKind,
    is_primary: bool,
}

pub(crate) fn translate(event: InputEvent, scroll: Point) -> Result<Dispatch, CoreError> {
    match event {
        InputEvent::PointerMove {
            id,
            kind,
            is_primary,
            position,
            buttons,
            modifiers,
        } => Ok(Dispatch::Upstream(BlitzUiEvent::PointerMove(
            pointer_event(
                PointerIdentity {
                    id,
                    kind,
                    is_primary,
                },
                position,
                PointerButton::Primary,
                buttons,
                modifiers,
                scroll,
            )?,
        ))),
        InputEvent::PointerDown {
            id,
            kind,
            is_primary,
            position,
            button,
            buttons,
            modifiers,
        } => Ok(Dispatch::Upstream(BlitzUiEvent::PointerDown(
            pointer_event(
                PointerIdentity {
                    id,
                    kind,
                    is_primary,
                },
                position,
                button,
                buttons,
                modifiers,
                scroll,
            )?,
        ))),
        InputEvent::PointerUp {
            id,
            kind,
            is_primary,
            position,
            button,
            buttons,
            modifiers,
        } => Ok(Dispatch::Upstream(BlitzUiEvent::PointerUp(pointer_event(
            PointerIdentity {
                id,
                kind,
                is_primary,
            },
            position,
            button,
            buttons,
            modifiers,
            scroll,
        )?))),
        InputEvent::PointerCancel {
            id,
            kind,
            is_primary,
            position,
            buttons,
            modifiers,
        } => Ok(Dispatch::Upstream(BlitzUiEvent::PointerCancel(
            pointer_event(
                PointerIdentity {
                    id,
                    kind,
                    is_primary,
                },
                position,
                PointerButton::Primary,
                buttons,
                modifiers,
                scroll,
            )?,
        ))),
        InputEvent::PointerLeave {
            id,
            kind,
            is_primary,
            position,
            buttons,
            modifiers,
        } => Ok(Dispatch::Upstream(BlitzUiEvent::PointerLeave(
            pointer_event(
                PointerIdentity {
                    id,
                    kind,
                    is_primary,
                },
                position,
                PointerButton::Primary,
                buttons,
                modifiers,
                scroll,
            )?,
        ))),
        InputEvent::Wheel {
            position,
            delta_x,
            delta_y,
            modifiers,
        } => {
            validate_point(position)?;
            if !delta_x.is_finite() || !delta_y.is_finite() {
                return Err(CoreError::InvalidArgument {
                    name: "wheel delta",
                    reason: "deltas must be finite".to_owned(),
                });
            }
            Ok(Dispatch::Upstream(BlitzUiEvent::Wheel(BlitzWheelEvent {
                delta: BlitzWheelDelta::Pixels(delta_x, delta_y),
                coords: pointer_coords(position, scroll)?,
                buttons: MouseEventButtons::None,
                mods: modifiers_to_blitz(modifiers),
                element: BlitzPoint::default(),
            })))
        }
        InputEvent::KeyDown {
            key,
            physical_key,
            location,
            modifiers,
            repeat,
        } => Ok(Dispatch::Upstream(BlitzUiEvent::KeyDown(key_event(
            key,
            physical_key,
            location,
            modifiers,
            repeat,
            KeyState::Pressed,
        )?))),
        InputEvent::KeyUp {
            key,
            physical_key,
            location,
            modifiers,
        } => Ok(Dispatch::Upstream(BlitzUiEvent::KeyUp(key_event(
            key,
            physical_key,
            location,
            modifiers,
            false,
            KeyState::Released,
        )?))),
        InputEvent::TextInput { text } | InputEvent::ImeCommit { text } => Ok(Dispatch::Upstream(
            BlitzUiEvent::Ime(BlitzImeEvent::Commit(text)),
        )),
        InputEvent::ImePreedit { text, selection } => {
            if let Some((start, end)) = selection
                && (start > end
                    || end > text.len()
                    || !text.is_char_boundary(start)
                    || !text.is_char_boundary(end))
            {
                return Err(CoreError::InvalidArgument {
                    name: "IME selection",
                    reason: "selection must be an ordered UTF-8 byte range within pre-edit text"
                        .to_owned(),
                });
            }
            Ok(Dispatch::Upstream(BlitzUiEvent::Ime(
                BlitzImeEvent::Preedit(text, selection),
            )))
        }
        InputEvent::ImeCancel => Ok(Dispatch::Upstream(BlitzUiEvent::Ime(
            BlitzImeEvent::Preedit(String::new(), None),
        ))),
        InputEvent::FocusGained => Ok(Dispatch::FocusGained),
        InputEvent::FocusLost => Ok(Dispatch::FocusLost),
    }
}

fn pointer_event(
    identity: PointerIdentity,
    position: Point,
    button: PointerButton,
    buttons: PointerButtons,
    modifiers: Modifiers,
    scroll: Point,
) -> Result<BlitzPointerEvent, CoreError> {
    validate_point(position)?;
    Ok(BlitzPointerEvent {
        id: match identity.kind {
            PointerKind::Mouse => BlitzPointerId::Mouse,
            PointerKind::Touch => BlitzPointerId::Finger(identity.id.0),
            PointerKind::Pen => BlitzPointerId::Pen,
        },
        is_primary: identity.is_primary,
        coords: pointer_coords(position, scroll)?,
        button: button_to_blitz(button),
        buttons: buttons_to_blitz(buttons),
        mods: modifiers_to_blitz(modifiers),
        details: PointerDetails::default(),
        element: BlitzPoint::default(),
        active_pointers: Arc::default(),
    })
}

fn pointer_coords(position: Point, scroll: Point) -> Result<PointerCoords, CoreError> {
    let client_x = position.x - scroll.x;
    let client_y = position.y - scroll.y;
    if !client_x.is_finite()
        || !client_y.is_finite()
        || client_x.abs() > f64::from(f32::MAX)
        || client_y.abs() > f64::from(f32::MAX)
    {
        return Err(CoreError::InvalidArgument {
            name: "input position",
            reason: "viewport-relative coordinates are not representable by the document engine"
                .to_owned(),
        });
    }
    Ok(PointerCoords {
        page_x: position.x as f32,
        page_y: position.y as f32,
        screen_x: client_x as f32,
        screen_y: client_y as f32,
        client_x: client_x as f32,
        client_y: client_y as f32,
    })
}

fn button_to_blitz(button: PointerButton) -> MouseEventButton {
    match button {
        PointerButton::Primary => MouseEventButton::Main,
        PointerButton::Secondary => MouseEventButton::Secondary,
        PointerButton::Auxiliary => MouseEventButton::Auxiliary,
        PointerButton::Back => MouseEventButton::Fourth,
        PointerButton::Forward => MouseEventButton::Fifth,
    }
}

fn buttons_to_blitz(buttons: PointerButtons) -> MouseEventButtons {
    let mut result = MouseEventButtons::None;
    if buttons.primary {
        result.insert(MouseEventButtons::Primary);
    }
    if buttons.secondary {
        result.insert(MouseEventButtons::Secondary);
    }
    if buttons.auxiliary {
        result.insert(MouseEventButtons::Auxiliary);
    }
    if buttons.back {
        result.insert(MouseEventButtons::Fourth);
    }
    if buttons.forward {
        result.insert(MouseEventButtons::Fifth);
    }
    result
}

fn modifiers_to_blitz(modifiers: Modifiers) -> BlitzModifiers {
    let mut result = BlitzModifiers::empty();
    if modifiers.shift {
        result.insert(BlitzModifiers::SHIFT);
    }
    if modifiers.control {
        result.insert(BlitzModifiers::CONTROL);
    }
    if modifiers.alt {
        result.insert(BlitzModifiers::ALT);
    }
    if modifiers.meta {
        result.insert(BlitzModifiers::META);
    }
    result
}

fn key_event(
    key: Key,
    physical_key: PhysicalKey,
    location: KeyLocation,
    modifiers: Modifiers,
    repeat: bool,
    state: KeyState,
) -> Result<BlitzKeyEvent, CoreError> {
    Ok(BlitzKeyEvent {
        key: key_to_blitz(key),
        code: match physical_key {
            PhysicalKey::Code(code) => {
                Code::from_str(&code).map_err(|_| CoreError::InvalidArgument {
                    name: "physical key",
                    reason: format!("unknown keyboard code {code}"),
                })?
            }
            PhysicalKey::Unidentified => Code::Unidentified,
        },
        modifiers: modifiers_to_blitz(modifiers),
        location: match location {
            KeyLocation::Standard => Location::Standard,
            KeyLocation::Left => Location::Left,
            KeyLocation::Right => Location::Right,
            KeyLocation::Numpad => Location::Numpad,
        },
        is_auto_repeating: repeat,
        is_composing: false,
        state,
        text: None,
    })
}

fn key_to_blitz(key: Key) -> BlitzKey {
    match key {
        Key::Character(character) => BlitzKey::Character(character),
        Key::Named(NamedKey::Space) => BlitzKey::Character(" ".to_owned()),
        Key::Named(named) => BlitzKey::Named(match named {
            NamedKey::Alt => BlitzNamedKey::Alt,
            NamedKey::ArrowDown => BlitzNamedKey::ArrowDown,
            NamedKey::ArrowLeft => BlitzNamedKey::ArrowLeft,
            NamedKey::ArrowRight => BlitzNamedKey::ArrowRight,
            NamedKey::ArrowUp => BlitzNamedKey::ArrowUp,
            NamedKey::Backspace => BlitzNamedKey::Backspace,
            NamedKey::Control => BlitzNamedKey::Control,
            NamedKey::Delete => BlitzNamedKey::Delete,
            NamedKey::End => BlitzNamedKey::End,
            NamedKey::Enter => BlitzNamedKey::Enter,
            NamedKey::Escape => BlitzNamedKey::Escape,
            NamedKey::Home => BlitzNamedKey::Home,
            NamedKey::Insert => BlitzNamedKey::Insert,
            NamedKey::Meta => BlitzNamedKey::Meta,
            NamedKey::PageDown => BlitzNamedKey::PageDown,
            NamedKey::PageUp => BlitzNamedKey::PageUp,
            NamedKey::Shift => BlitzNamedKey::Shift,
            NamedKey::Tab => BlitzNamedKey::Tab,
            NamedKey::Space => BlitzNamedKey::Unidentified,
        }),
        Key::Unidentified => BlitzKey::Named(BlitzNamedKey::Unidentified),
    }
}

fn validate_point(point: Point) -> Result<(), CoreError> {
    if point.x.is_finite()
        && point.y.is_finite()
        && point.x.abs() <= f64::from(f32::MAX)
        && point.y.abs() <= f64::from(f32::MAX)
    {
        Ok(())
    } else {
        Err(CoreError::InvalidArgument {
            name: "input position",
            reason: "coordinates must be finite and representable by the document engine"
                .to_owned(),
        })
    }
}
