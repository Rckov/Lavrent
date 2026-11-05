use std::ptr::{self, slice_from_raw_parts_mut};

use engine_core::{
    InputEvent, Key, KeyLocation, Modifiers, NamedKey, PhysicalKey, PointerButton, PointerButtons,
    PointerId, PointerKind, UiEventData, UiEventKind,
};

use crate::{
    ENGINE_INPUT_FOCUS_GAINED, ENGINE_INPUT_FOCUS_LOST, ENGINE_INPUT_IME_CANCEL,
    ENGINE_INPUT_IME_COMMIT, ENGINE_INPUT_IME_PREEDIT, ENGINE_INPUT_KEY_DOWN, ENGINE_INPUT_KEY_UP,
    ENGINE_INPUT_POINTER_CANCEL, ENGINE_INPUT_POINTER_DOWN, ENGINE_INPUT_POINTER_LEAVE,
    ENGINE_INPUT_POINTER_MOVE, ENGINE_INPUT_POINTER_UP, ENGINE_INPUT_TEXT, ENGINE_INPUT_WHEEL,
    ENGINE_KEY_CHARACTER, ENGINE_KEY_LOCATION_LEFT, ENGINE_KEY_LOCATION_NUMPAD,
    ENGINE_KEY_LOCATION_RIGHT, ENGINE_KEY_LOCATION_STANDARD, ENGINE_KEY_NAMED,
    ENGINE_KEY_UNIDENTIFIED, ENGINE_MODIFIER_ALT, ENGINE_MODIFIER_CONTROL, ENGINE_MODIFIER_META,
    ENGINE_MODIFIER_SHIFT, ENGINE_NAMED_KEY_ALT, ENGINE_NAMED_KEY_ARROW_DOWN,
    ENGINE_NAMED_KEY_ARROW_LEFT, ENGINE_NAMED_KEY_ARROW_RIGHT, ENGINE_NAMED_KEY_ARROW_UP,
    ENGINE_NAMED_KEY_BACKSPACE, ENGINE_NAMED_KEY_CONTROL, ENGINE_NAMED_KEY_DELETE,
    ENGINE_NAMED_KEY_END, ENGINE_NAMED_KEY_ENTER, ENGINE_NAMED_KEY_ESCAPE, ENGINE_NAMED_KEY_HOME,
    ENGINE_NAMED_KEY_INSERT, ENGINE_NAMED_KEY_META, ENGINE_NAMED_KEY_PAGE_DOWN,
    ENGINE_NAMED_KEY_PAGE_UP, ENGINE_NAMED_KEY_SHIFT, ENGINE_NAMED_KEY_SPACE, ENGINE_NAMED_KEY_TAB,
    ENGINE_POINTER_BUTTON_AUXILIARY, ENGINE_POINTER_BUTTON_BACK, ENGINE_POINTER_BUTTON_FORWARD,
    ENGINE_POINTER_BUTTON_PRIMARY, ENGINE_POINTER_BUTTON_SECONDARY,
    ENGINE_POINTER_BUTTONS_AUXILIARY, ENGINE_POINTER_BUTTONS_BACK, ENGINE_POINTER_BUTTONS_FORWARD,
    ENGINE_POINTER_BUTTONS_PRIMARY, ENGINE_POINTER_BUTTONS_SECONDARY, ENGINE_POINTER_MOUSE,
    ENGINE_POINTER_PEN, ENGINE_POINTER_TOUCH, ENGINE_UI_DATA_NONE, ENGINE_UI_DATA_TEXT,
    ENGINE_UI_EVENT_BLUR, ENGINE_UI_EVENT_CHANGE, ENGINE_UI_EVENT_CLICK, ENGINE_UI_EVENT_FOCUS,
    ENGINE_UI_EVENT_INPUT, ENGINE_UI_EVENT_SUBMIT, Engine, EngineInputEvent, EngineUiEvent,
    EngineUiEventArray, NativeCallError, borrow_core, free_bytes, into_owned_bytes,
    read_optional_utf8, read_utf8, require_output, run_ffi_call, write_output,
};

const POINTER_BUTTON_FLAGS: u32 = ENGINE_POINTER_BUTTONS_PRIMARY
    | ENGINE_POINTER_BUTTONS_SECONDARY
    | ENGINE_POINTER_BUTTONS_AUXILIARY
    | ENGINE_POINTER_BUTTONS_BACK
    | ENGINE_POINTER_BUTTONS_FORWARD;
const MODIFIER_FLAGS: u32 =
    ENGINE_MODIFIER_SHIFT | ENGINE_MODIFIER_CONTROL | ENGINE_MODIFIER_ALT | ENGINE_MODIFIER_META;

fn parse_bool(value: u8, name: &'static str) -> Result<bool, NativeCallError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(NativeCallError::invalid(format!("{name} must be 0 or 1"))),
    }
}

fn convert_pointer_kind(value: u32) -> Result<PointerKind, NativeCallError> {
    match value {
        ENGINE_POINTER_MOUSE => Ok(PointerKind::Mouse),
        ENGINE_POINTER_TOUCH => Ok(PointerKind::Touch),
        ENGINE_POINTER_PEN => Ok(PointerKind::Pen),
        _ => Err(NativeCallError::invalid("unknown pointer kind")),
    }
}

fn convert_pointer_button(value: u32) -> Result<PointerButton, NativeCallError> {
    match value {
        ENGINE_POINTER_BUTTON_PRIMARY => Ok(PointerButton::Primary),
        ENGINE_POINTER_BUTTON_SECONDARY => Ok(PointerButton::Secondary),
        ENGINE_POINTER_BUTTON_AUXILIARY => Ok(PointerButton::Auxiliary),
        ENGINE_POINTER_BUTTON_BACK => Ok(PointerButton::Back),
        ENGINE_POINTER_BUTTON_FORWARD => Ok(PointerButton::Forward),
        _ => Err(NativeCallError::invalid("unknown pointer button")),
    }
}

fn convert_pointer_buttons(value: u32) -> Result<PointerButtons, NativeCallError> {
    if value & !POINTER_BUTTON_FLAGS != 0 {
        return Err(NativeCallError::invalid("unknown pointer button flag"));
    }
    Ok(PointerButtons {
        primary: value & ENGINE_POINTER_BUTTONS_PRIMARY != 0,
        secondary: value & ENGINE_POINTER_BUTTONS_SECONDARY != 0,
        auxiliary: value & ENGINE_POINTER_BUTTONS_AUXILIARY != 0,
        back: value & ENGINE_POINTER_BUTTONS_BACK != 0,
        forward: value & ENGINE_POINTER_BUTTONS_FORWARD != 0,
    })
}

fn convert_modifiers(value: u32) -> Result<Modifiers, NativeCallError> {
    if value & !MODIFIER_FLAGS != 0 {
        return Err(NativeCallError::invalid("unknown modifier flag"));
    }
    Ok(Modifiers {
        shift: value & ENGINE_MODIFIER_SHIFT != 0,
        control: value & ENGINE_MODIFIER_CONTROL != 0,
        alt: value & ENGINE_MODIFIER_ALT != 0,
        meta: value & ENGINE_MODIFIER_META != 0,
    })
}

fn convert_named_key(value: u32) -> Result<NamedKey, NativeCallError> {
    match value {
        ENGINE_NAMED_KEY_ALT => Ok(NamedKey::Alt),
        ENGINE_NAMED_KEY_ARROW_DOWN => Ok(NamedKey::ArrowDown),
        ENGINE_NAMED_KEY_ARROW_LEFT => Ok(NamedKey::ArrowLeft),
        ENGINE_NAMED_KEY_ARROW_RIGHT => Ok(NamedKey::ArrowRight),
        ENGINE_NAMED_KEY_ARROW_UP => Ok(NamedKey::ArrowUp),
        ENGINE_NAMED_KEY_BACKSPACE => Ok(NamedKey::Backspace),
        ENGINE_NAMED_KEY_CONTROL => Ok(NamedKey::Control),
        ENGINE_NAMED_KEY_DELETE => Ok(NamedKey::Delete),
        ENGINE_NAMED_KEY_END => Ok(NamedKey::End),
        ENGINE_NAMED_KEY_ENTER => Ok(NamedKey::Enter),
        ENGINE_NAMED_KEY_ESCAPE => Ok(NamedKey::Escape),
        ENGINE_NAMED_KEY_HOME => Ok(NamedKey::Home),
        ENGINE_NAMED_KEY_INSERT => Ok(NamedKey::Insert),
        ENGINE_NAMED_KEY_META => Ok(NamedKey::Meta),
        ENGINE_NAMED_KEY_PAGE_DOWN => Ok(NamedKey::PageDown),
        ENGINE_NAMED_KEY_PAGE_UP => Ok(NamedKey::PageUp),
        ENGINE_NAMED_KEY_SHIFT => Ok(NamedKey::Shift),
        ENGINE_NAMED_KEY_SPACE => Ok(NamedKey::Space),
        ENGINE_NAMED_KEY_TAB => Ok(NamedKey::Tab),
        _ => Err(NativeCallError::invalid("unknown named key")),
    }
}

unsafe fn convert_key(event: &EngineInputEvent) -> Result<Key, NativeCallError> {
    match event.key_kind {
        ENGINE_KEY_UNIDENTIFIED => Ok(Key::Unidentified),
        ENGINE_KEY_NAMED => convert_named_key(event.named_key).map(Key::Named),
        ENGINE_KEY_CHARACTER => unsafe { read_utf8(event.key_text) }.map(Key::Character),
        _ => Err(NativeCallError::invalid("unknown key kind")),
    }
}

unsafe fn convert_physical_key(event: &EngineInputEvent) -> Result<PhysicalKey, NativeCallError> {
    match unsafe { read_optional_utf8(event.physical_key)? } {
        Some(code) => Ok(PhysicalKey::Code(code)),
        None => Ok(PhysicalKey::Unidentified),
    }
}

fn convert_key_location(value: u32) -> Result<KeyLocation, NativeCallError> {
    match value {
        ENGINE_KEY_LOCATION_STANDARD => Ok(KeyLocation::Standard),
        ENGINE_KEY_LOCATION_LEFT => Ok(KeyLocation::Left),
        ENGINE_KEY_LOCATION_RIGHT => Ok(KeyLocation::Right),
        ENGINE_KEY_LOCATION_NUMPAD => Ok(KeyLocation::Numpad),
        _ => Err(NativeCallError::invalid("unknown key location")),
    }
}

unsafe fn convert_input_event(event: EngineInputEvent) -> Result<InputEvent, NativeCallError> {
    let position = engine_core::Point::new(event.position.x, event.position.y);
    match event.kind {
        ENGINE_INPUT_POINTER_MOVE => Ok(InputEvent::PointerMove {
            id: PointerId(event.pointer_id),
            kind: convert_pointer_kind(event.pointer_kind)?,
            is_primary: parse_bool(event.is_primary, "is_primary")?,
            position,
            buttons: convert_pointer_buttons(event.buttons)?,
            modifiers: convert_modifiers(event.modifiers)?,
        }),
        ENGINE_INPUT_POINTER_DOWN => Ok(InputEvent::PointerDown {
            id: PointerId(event.pointer_id),
            kind: convert_pointer_kind(event.pointer_kind)?,
            is_primary: parse_bool(event.is_primary, "is_primary")?,
            position,
            button: convert_pointer_button(event.button)?,
            buttons: convert_pointer_buttons(event.buttons)?,
            modifiers: convert_modifiers(event.modifiers)?,
        }),
        ENGINE_INPUT_POINTER_UP => Ok(InputEvent::PointerUp {
            id: PointerId(event.pointer_id),
            kind: convert_pointer_kind(event.pointer_kind)?,
            is_primary: parse_bool(event.is_primary, "is_primary")?,
            position,
            button: convert_pointer_button(event.button)?,
            buttons: convert_pointer_buttons(event.buttons)?,
            modifiers: convert_modifiers(event.modifiers)?,
        }),
        ENGINE_INPUT_POINTER_CANCEL => Ok(InputEvent::PointerCancel {
            id: PointerId(event.pointer_id),
            kind: convert_pointer_kind(event.pointer_kind)?,
            is_primary: parse_bool(event.is_primary, "is_primary")?,
            position,
            buttons: convert_pointer_buttons(event.buttons)?,
            modifiers: convert_modifiers(event.modifiers)?,
        }),
        ENGINE_INPUT_POINTER_LEAVE => Ok(InputEvent::PointerLeave {
            id: PointerId(event.pointer_id),
            kind: convert_pointer_kind(event.pointer_kind)?,
            is_primary: parse_bool(event.is_primary, "is_primary")?,
            position,
            buttons: convert_pointer_buttons(event.buttons)?,
            modifiers: convert_modifiers(event.modifiers)?,
        }),
        ENGINE_INPUT_WHEEL => Ok(InputEvent::Wheel {
            position,
            delta_x: event.delta_x,
            delta_y: event.delta_y,
            modifiers: convert_modifiers(event.modifiers)?,
        }),
        ENGINE_INPUT_KEY_DOWN => Ok(InputEvent::KeyDown {
            key: unsafe { convert_key(&event)? },
            physical_key: unsafe { convert_physical_key(&event)? },
            location: convert_key_location(event.key_location)?,
            modifiers: convert_modifiers(event.modifiers)?,
            repeat: parse_bool(event.repeat, "repeat")?,
        }),
        ENGINE_INPUT_KEY_UP => Ok(InputEvent::KeyUp {
            key: unsafe { convert_key(&event)? },
            physical_key: unsafe { convert_physical_key(&event)? },
            location: convert_key_location(event.key_location)?,
            modifiers: convert_modifiers(event.modifiers)?,
        }),
        ENGINE_INPUT_TEXT => Ok(InputEvent::TextInput {
            text: unsafe { read_utf8(event.text)? },
        }),
        ENGINE_INPUT_IME_PREEDIT => Ok(InputEvent::ImePreedit {
            text: unsafe { read_utf8(event.text)? },
            selection: if parse_bool(event.has_ime_selection, "has_ime_selection")? {
                Some((event.ime_selection_start, event.ime_selection_end))
            } else {
                None
            },
        }),
        ENGINE_INPUT_IME_COMMIT => Ok(InputEvent::ImeCommit {
            text: unsafe { read_utf8(event.text)? },
        }),
        ENGINE_INPUT_IME_CANCEL => Ok(InputEvent::ImeCancel),
        ENGINE_INPUT_FOCUS_GAINED => Ok(InputEvent::FocusGained),
        ENGINE_INPUT_FOCUS_LOST => Ok(InputEvent::FocusLost),
        _ => Err(NativeCallError::invalid("unknown input event kind")),
    }
}

fn convert_ui_event_kind(kind: UiEventKind) -> u32 {
    match kind {
        UiEventKind::Click => ENGINE_UI_EVENT_CLICK,
        UiEventKind::Input => ENGINE_UI_EVENT_INPUT,
        UiEventKind::Change => ENGINE_UI_EVENT_CHANGE,
        UiEventKind::Submit => ENGINE_UI_EVENT_SUBMIT,
        UiEventKind::Focus => ENGINE_UI_EVENT_FOCUS,
        UiEventKind::Blur => ENGINE_UI_EVENT_BLUR,
    }
}

fn into_ui_event_array(events: Vec<EngineUiEvent>) -> EngineUiEventArray {
    if events.is_empty() {
        return EngineUiEventArray {
            data: ptr::null_mut(),
            length: 0,
        };
    }
    let mut events = events.into_boxed_slice();
    let result = EngineUiEventArray {
        data: events.as_mut_ptr(),
        length: events.len(),
    };
    let _ = Box::leak(events);
    result
}

/// # Safety
///
/// String pointers in `event` must remain valid for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_input_dispatch(
    core: *mut Engine,
    document: u64,
    event: EngineInputEvent,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        let event = unsafe { convert_input_event(event)? };
        let document = core.documents.resolve(document, "document")?;
        core.inner.dispatch_input(document, event)?;
        Ok(())
    })
}

/// # Safety
///
/// `output` must point to writable storage for one event array.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_ui_events_drain(
    core: *mut Engine,
    output: *mut EngineUiEventArray,
) -> u32 {
    run_ffi_call(core, || {
        let core = unsafe { borrow_core(core)? };
        require_output(output)?;
        let events = core.inner.drain_ui_events()?;
        let mut resolved_events = Vec::with_capacity(events.len());
        for event in events {
            let target = core.nodes.intern(event.target)?;
            resolved_events.push((target, event));
        }
        let mut output_events = Vec::with_capacity(resolved_events.len());
        for (target, event) in resolved_events {
            let (data_kind, text) = match event.data {
                UiEventData::None => (ENGINE_UI_DATA_NONE, into_owned_bytes(Vec::new())),
                UiEventData::Text(text) => {
                    (ENGINE_UI_DATA_TEXT, into_owned_bytes(text.into_bytes()))
                }
            };
            output_events.push(EngineUiEvent {
                target,
                kind: convert_ui_event_kind(event.kind),
                data_kind,
                text,
            });
        }
        unsafe { write_output(output, into_ui_event_array(output_events)) }
    })
}

/// # Safety
///
/// `events` must be empty or have been returned by the native library and not freed before.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn engine_ui_events_free(events: EngineUiEventArray) {
    if events.data.is_null() {
        return;
    }
    // SAFETY: the pointer and length originate from owned_ui_events.
    let slice = slice_from_raw_parts_mut(events.data, events.length);
    // SAFETY: owned_ui_events leaked exactly one boxed slice for this pointer.
    let events = unsafe { Box::from_raw(slice) };
    for event in &events {
        unsafe { free_bytes(event.text) };
    }
}
