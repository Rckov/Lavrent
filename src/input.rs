use crate::{NodeHandle, Point};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PointerId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerKind {
    Mouse,
    Touch,
    Pen,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerButton {
    Primary,
    Secondary,
    Auxiliary,
    Back,
    Forward,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PointerButtons {
    pub primary: bool,
    pub secondary: bool,
    pub auxiliary: bool,
    pub back: bool,
    pub forward: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub meta: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NamedKey {
    Alt,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    Backspace,
    Control,
    Delete,
    End,
    Enter,
    Escape,
    Home,
    Insert,
    Meta,
    PageDown,
    PageUp,
    Shift,
    Space,
    Tab,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Key {
    Named(NamedKey),
    Character(String),
    Unidentified,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PhysicalKey {
    Code(String),
    Unidentified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyLocation {
    Standard,
    Left,
    Right,
    Numpad,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InputEvent {
    PointerMove {
        id: PointerId,
        kind: PointerKind,
        position: Point,
        buttons: PointerButtons,
        modifiers: Modifiers,
    },
    PointerDown {
        id: PointerId,
        kind: PointerKind,
        position: Point,
        button: PointerButton,
        buttons: PointerButtons,
        modifiers: Modifiers,
    },
    PointerUp {
        id: PointerId,
        kind: PointerKind,
        position: Point,
        button: PointerButton,
        buttons: PointerButtons,
        modifiers: Modifiers,
    },
    PointerCancel {
        id: PointerId,
    },
    PointerLeave {
        id: PointerId,
    },
    Wheel {
        position: Point,
        delta_x: f64,
        delta_y: f64,
        modifiers: Modifiers,
    },
    KeyDown {
        key: Key,
        physical_key: PhysicalKey,
        location: KeyLocation,
        modifiers: Modifiers,
        repeat: bool,
    },
    KeyUp {
        key: Key,
        physical_key: PhysicalKey,
        location: KeyLocation,
        modifiers: Modifiers,
    },
    TextInput {
        text: String,
    },
    ImePreedit {
        text: String,
        selection: Option<(usize, usize)>,
    },
    ImeCommit {
        text: String,
    },
    ImeCancel,
    FocusGained,
    FocusLost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiEventKind {
    Click,
    Input,
    Change,
    Submit,
    Focus,
    Blur,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiEventData {
    None,
    Text(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiEvent {
    pub target: NodeHandle,
    pub kind: UiEventKind,
    pub data: UiEventData,
}
