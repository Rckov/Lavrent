use std::ptr::{self, slice_from_raw_parts_mut};

use lavret_core::{
    InputEvent, Key, KeyLocation, Modifiers, NamedKey, PhysicalKey, PointerButton, PointerButtons,
    PointerId, PointerKind, UiEventData, UiEventKind,
};

use crate::{
    LAVRET_INPUT_FOCUS_GAINED, LAVRET_INPUT_FOCUS_LOST, LAVRET_INPUT_IME_CANCEL,
    LAVRET_INPUT_IME_COMMIT, LAVRET_INPUT_IME_PREEDIT, LAVRET_INPUT_KEY_DOWN, LAVRET_INPUT_KEY_UP,
    LAVRET_INPUT_POINTER_CANCEL, LAVRET_INPUT_POINTER_DOWN, LAVRET_INPUT_POINTER_LEAVE,
    LAVRET_INPUT_POINTER_MOVE, LAVRET_INPUT_POINTER_UP, LAVRET_INPUT_TEXT, LAVRET_INPUT_WHEEL,
    LAVRET_KEY_CHARACTER, LAVRET_KEY_LOCATION_LEFT, LAVRET_KEY_LOCATION_NUMPAD,
    LAVRET_KEY_LOCATION_RIGHT, LAVRET_KEY_LOCATION_STANDARD, LAVRET_KEY_NAMED,
    LAVRET_KEY_UNIDENTIFIED, LAVRET_MODIFIER_ALT, LAVRET_MODIFIER_CONTROL, LAVRET_MODIFIER_META,
    LAVRET_MODIFIER_SHIFT, LAVRET_NAMED_KEY_ALT, LAVRET_NAMED_KEY_ARROW_DOWN,
    LAVRET_NAMED_KEY_ARROW_LEFT, LAVRET_NAMED_KEY_ARROW_RIGHT, LAVRET_NAMED_KEY_ARROW_UP,
    LAVRET_NAMED_KEY_BACKSPACE, LAVRET_NAMED_KEY_CONTROL, LAVRET_NAMED_KEY_DELETE,
    LAVRET_NAMED_KEY_END, LAVRET_NAMED_KEY_ENTER, LAVRET_NAMED_KEY_ESCAPE, LAVRET_NAMED_KEY_HOME,
    LAVRET_NAMED_KEY_INSERT, LAVRET_NAMED_KEY_META, LAVRET_NAMED_KEY_PAGE_DOWN,
    LAVRET_NAMED_KEY_PAGE_UP, LAVRET_NAMED_KEY_SHIFT, LAVRET_NAMED_KEY_SPACE, LAVRET_NAMED_KEY_TAB,
    LAVRET_POINTER_BUTTON_AUXILIARY, LAVRET_POINTER_BUTTON_BACK, LAVRET_POINTER_BUTTON_FORWARD,
    LAVRET_POINTER_BUTTON_PRIMARY, LAVRET_POINTER_BUTTON_SECONDARY,
    LAVRET_POINTER_BUTTONS_AUXILIARY, LAVRET_POINTER_BUTTONS_BACK, LAVRET_POINTER_BUTTONS_FORWARD,
    LAVRET_POINTER_BUTTONS_PRIMARY, LAVRET_POINTER_BUTTONS_SECONDARY, LAVRET_POINTER_MOUSE,
    LAVRET_POINTER_PEN, LAVRET_POINTER_TOUCH, LAVRET_UI_DATA_NONE, LAVRET_UI_DATA_TEXT,
    LAVRET_UI_EVENT_BLUR, LAVRET_UI_EVENT_CHANGE, LAVRET_UI_EVENT_CLICK, LAVRET_UI_EVENT_FOCUS,
    LAVRET_UI_EVENT_INPUT, LAVRET_UI_EVENT_SUBMIT, LavretCore, LavretInputEvent, LavretUiEvent,
    LavretUiEventArray, NativeError, core_mut, ffi_status, free_owned_bytes, optional_string,
    owned_bytes, require_output, string, write_output,
};

const POINTER_BUTTON_FLAGS: u32 = LAVRET_POINTER_BUTTONS_PRIMARY
    | LAVRET_POINTER_BUTTONS_SECONDARY
    | LAVRET_POINTER_BUTTONS_AUXILIARY
    | LAVRET_POINTER_BUTTONS_BACK
    | LAVRET_POINTER_BUTTONS_FORWARD;
const MODIFIER_FLAGS: u32 =
    LAVRET_MODIFIER_SHIFT | LAVRET_MODIFIER_CONTROL | LAVRET_MODIFIER_ALT | LAVRET_MODIFIER_META;

fn boolean(value: u8, name: &'static str) -> Result<bool, NativeError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(NativeError::invalid(format!("{name} must be 0 or 1"))),
    }
}

fn pointer_kind(value: u32) -> Result<PointerKind, NativeError> {
    match value {
        LAVRET_POINTER_MOUSE => Ok(PointerKind::Mouse),
        LAVRET_POINTER_TOUCH => Ok(PointerKind::Touch),
        LAVRET_POINTER_PEN => Ok(PointerKind::Pen),
        _ => Err(NativeError::invalid("unknown pointer kind")),
    }
}

fn pointer_button(value: u32) -> Result<PointerButton, NativeError> {
    match value {
        LAVRET_POINTER_BUTTON_PRIMARY => Ok(PointerButton::Primary),
        LAVRET_POINTER_BUTTON_SECONDARY => Ok(PointerButton::Secondary),
        LAVRET_POINTER_BUTTON_AUXILIARY => Ok(PointerButton::Auxiliary),
        LAVRET_POINTER_BUTTON_BACK => Ok(PointerButton::Back),
        LAVRET_POINTER_BUTTON_FORWARD => Ok(PointerButton::Forward),
        _ => Err(NativeError::invalid("unknown pointer button")),
    }
}

fn pointer_buttons(value: u32) -> Result<PointerButtons, NativeError> {
    if value & !POINTER_BUTTON_FLAGS != 0 {
        return Err(NativeError::invalid("unknown pointer button flag"));
    }
    Ok(PointerButtons {
        primary: value & LAVRET_POINTER_BUTTONS_PRIMARY != 0,
        secondary: value & LAVRET_POINTER_BUTTONS_SECONDARY != 0,
        auxiliary: value & LAVRET_POINTER_BUTTONS_AUXILIARY != 0,
        back: value & LAVRET_POINTER_BUTTONS_BACK != 0,
        forward: value & LAVRET_POINTER_BUTTONS_FORWARD != 0,
    })
}

fn modifiers(value: u32) -> Result<Modifiers, NativeError> {
    if value & !MODIFIER_FLAGS != 0 {
        return Err(NativeError::invalid("unknown modifier flag"));
    }
    Ok(Modifiers {
        shift: value & LAVRET_MODIFIER_SHIFT != 0,
        control: value & LAVRET_MODIFIER_CONTROL != 0,
        alt: value & LAVRET_MODIFIER_ALT != 0,
        meta: value & LAVRET_MODIFIER_META != 0,
    })
}

fn named_key(value: u32) -> Result<NamedKey, NativeError> {
    match value {
        LAVRET_NAMED_KEY_ALT => Ok(NamedKey::Alt),
        LAVRET_NAMED_KEY_ARROW_DOWN => Ok(NamedKey::ArrowDown),
        LAVRET_NAMED_KEY_ARROW_LEFT => Ok(NamedKey::ArrowLeft),
        LAVRET_NAMED_KEY_ARROW_RIGHT => Ok(NamedKey::ArrowRight),
        LAVRET_NAMED_KEY_ARROW_UP => Ok(NamedKey::ArrowUp),
        LAVRET_NAMED_KEY_BACKSPACE => Ok(NamedKey::Backspace),
        LAVRET_NAMED_KEY_CONTROL => Ok(NamedKey::Control),
        LAVRET_NAMED_KEY_DELETE => Ok(NamedKey::Delete),
        LAVRET_NAMED_KEY_END => Ok(NamedKey::End),
        LAVRET_NAMED_KEY_ENTER => Ok(NamedKey::Enter),
        LAVRET_NAMED_KEY_ESCAPE => Ok(NamedKey::Escape),
        LAVRET_NAMED_KEY_HOME => Ok(NamedKey::Home),
        LAVRET_NAMED_KEY_INSERT => Ok(NamedKey::Insert),
        LAVRET_NAMED_KEY_META => Ok(NamedKey::Meta),
        LAVRET_NAMED_KEY_PAGE_DOWN => Ok(NamedKey::PageDown),
        LAVRET_NAMED_KEY_PAGE_UP => Ok(NamedKey::PageUp),
        LAVRET_NAMED_KEY_SHIFT => Ok(NamedKey::Shift),
        LAVRET_NAMED_KEY_SPACE => Ok(NamedKey::Space),
        LAVRET_NAMED_KEY_TAB => Ok(NamedKey::Tab),
        _ => Err(NativeError::invalid("unknown named key")),
    }
}

unsafe fn key(event: &LavretInputEvent) -> Result<Key, NativeError> {
    match event.key_kind {
        LAVRET_KEY_UNIDENTIFIED => Ok(Key::Unidentified),
        LAVRET_KEY_NAMED => named_key(event.named_key).map(Key::Named),
        LAVRET_KEY_CHARACTER => unsafe { string(event.key_text) }.map(Key::Character),
        _ => Err(NativeError::invalid("unknown key kind")),
    }
}

unsafe fn physical_key(event: &LavretInputEvent) -> Result<PhysicalKey, NativeError> {
    match unsafe { optional_string(event.physical_key)? } {
        Some(code) => Ok(PhysicalKey::Code(code)),
        None => Ok(PhysicalKey::Unidentified),
    }
}

fn key_location(value: u32) -> Result<KeyLocation, NativeError> {
    match value {
        LAVRET_KEY_LOCATION_STANDARD => Ok(KeyLocation::Standard),
        LAVRET_KEY_LOCATION_LEFT => Ok(KeyLocation::Left),
        LAVRET_KEY_LOCATION_RIGHT => Ok(KeyLocation::Right),
        LAVRET_KEY_LOCATION_NUMPAD => Ok(KeyLocation::Numpad),
        _ => Err(NativeError::invalid("unknown key location")),
    }
}

unsafe fn input_event(event: LavretInputEvent) -> Result<InputEvent, NativeError> {
    let position = lavret_core::Point::new(event.position.x, event.position.y);
    match event.kind {
        LAVRET_INPUT_POINTER_MOVE => Ok(InputEvent::PointerMove {
            id: PointerId(event.pointer_id),
            kind: pointer_kind(event.pointer_kind)?,
            is_primary: boolean(event.is_primary, "is_primary")?,
            position,
            buttons: pointer_buttons(event.buttons)?,
            modifiers: modifiers(event.modifiers)?,
        }),
        LAVRET_INPUT_POINTER_DOWN => Ok(InputEvent::PointerDown {
            id: PointerId(event.pointer_id),
            kind: pointer_kind(event.pointer_kind)?,
            is_primary: boolean(event.is_primary, "is_primary")?,
            position,
            button: pointer_button(event.button)?,
            buttons: pointer_buttons(event.buttons)?,
            modifiers: modifiers(event.modifiers)?,
        }),
        LAVRET_INPUT_POINTER_UP => Ok(InputEvent::PointerUp {
            id: PointerId(event.pointer_id),
            kind: pointer_kind(event.pointer_kind)?,
            is_primary: boolean(event.is_primary, "is_primary")?,
            position,
            button: pointer_button(event.button)?,
            buttons: pointer_buttons(event.buttons)?,
            modifiers: modifiers(event.modifiers)?,
        }),
        LAVRET_INPUT_POINTER_CANCEL => Ok(InputEvent::PointerCancel {
            id: PointerId(event.pointer_id),
            kind: pointer_kind(event.pointer_kind)?,
            is_primary: boolean(event.is_primary, "is_primary")?,
            position,
            buttons: pointer_buttons(event.buttons)?,
            modifiers: modifiers(event.modifiers)?,
        }),
        LAVRET_INPUT_POINTER_LEAVE => Ok(InputEvent::PointerLeave {
            id: PointerId(event.pointer_id),
            kind: pointer_kind(event.pointer_kind)?,
            is_primary: boolean(event.is_primary, "is_primary")?,
            position,
            buttons: pointer_buttons(event.buttons)?,
            modifiers: modifiers(event.modifiers)?,
        }),
        LAVRET_INPUT_WHEEL => Ok(InputEvent::Wheel {
            position,
            delta_x: event.delta_x,
            delta_y: event.delta_y,
            modifiers: modifiers(event.modifiers)?,
        }),
        LAVRET_INPUT_KEY_DOWN => Ok(InputEvent::KeyDown {
            key: unsafe { key(&event)? },
            physical_key: unsafe { physical_key(&event)? },
            location: key_location(event.key_location)?,
            modifiers: modifiers(event.modifiers)?,
            repeat: boolean(event.repeat, "repeat")?,
        }),
        LAVRET_INPUT_KEY_UP => Ok(InputEvent::KeyUp {
            key: unsafe { key(&event)? },
            physical_key: unsafe { physical_key(&event)? },
            location: key_location(event.key_location)?,
            modifiers: modifiers(event.modifiers)?,
        }),
        LAVRET_INPUT_TEXT => Ok(InputEvent::TextInput {
            text: unsafe { string(event.text)? },
        }),
        LAVRET_INPUT_IME_PREEDIT => Ok(InputEvent::ImePreedit {
            text: unsafe { string(event.text)? },
            selection: if boolean(event.has_ime_selection, "has_ime_selection")? {
                Some((event.ime_selection_start, event.ime_selection_end))
            } else {
                None
            },
        }),
        LAVRET_INPUT_IME_COMMIT => Ok(InputEvent::ImeCommit {
            text: unsafe { string(event.text)? },
        }),
        LAVRET_INPUT_IME_CANCEL => Ok(InputEvent::ImeCancel),
        LAVRET_INPUT_FOCUS_GAINED => Ok(InputEvent::FocusGained),
        LAVRET_INPUT_FOCUS_LOST => Ok(InputEvent::FocusLost),
        _ => Err(NativeError::invalid("unknown input event kind")),
    }
}

fn ui_event_kind(kind: UiEventKind) -> u32 {
    match kind {
        UiEventKind::Click => LAVRET_UI_EVENT_CLICK,
        UiEventKind::Input => LAVRET_UI_EVENT_INPUT,
        UiEventKind::Change => LAVRET_UI_EVENT_CHANGE,
        UiEventKind::Submit => LAVRET_UI_EVENT_SUBMIT,
        UiEventKind::Focus => LAVRET_UI_EVENT_FOCUS,
        UiEventKind::Blur => LAVRET_UI_EVENT_BLUR,
    }
}

fn owned_ui_events(events: Vec<LavretUiEvent>) -> LavretUiEventArray {
    if events.is_empty() {
        return LavretUiEventArray {
            data: ptr::null_mut(),
            length: 0,
        };
    }
    let mut events = events.into_boxed_slice();
    let result = LavretUiEventArray {
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
pub unsafe extern "C" fn lavret_input_dispatch(
    core: *mut LavretCore,
    document: u64,
    event: LavretInputEvent,
) -> u32 {
    ffi_status(|| {
        let event = unsafe { input_event(event)? };
        let core = unsafe { core_mut(core)? };
        let document = core.documents.resolve(document, "document")?;
        core.inner.dispatch_input(document, event)?;
        Ok(())
    })
}

/// # Safety
///
/// `output` must point to writable storage for one event array.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_ui_events_drain(
    core: *mut LavretCore,
    output: *mut LavretUiEventArray,
) -> u32 {
    ffi_status(|| {
        require_output(output)?;
        let core = unsafe { core_mut(core)? };
        let events = core.inner.drain_ui_events()?;
        let mut output_events = Vec::with_capacity(events.len());
        for event in events {
            let target = core.nodes.intern(event.target)?;
            let (data_kind, text) = match event.data {
                UiEventData::None => (LAVRET_UI_DATA_NONE, owned_bytes(Vec::new())),
                UiEventData::Text(text) => (LAVRET_UI_DATA_TEXT, owned_bytes(text.into_bytes())),
            };
            output_events.push(LavretUiEvent {
                target,
                kind: ui_event_kind(event.kind),
                data_kind,
                text,
            });
        }
        unsafe { write_output(output, owned_ui_events(output_events)) }
    })
}

/// # Safety
///
/// `events` must be empty or have been returned by Lavret and not freed before.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lavret_ui_events_free(events: LavretUiEventArray) {
    if events.data.is_null() {
        return;
    }
    // SAFETY: the pointer and length originate from owned_ui_events.
    let slice = slice_from_raw_parts_mut(events.data, events.length);
    // SAFETY: owned_ui_events leaked exactly one boxed slice for this pointer.
    let events = unsafe { Box::from_raw(slice) };
    for event in &events {
        unsafe { free_owned_bytes(event.text) };
    }
}
