//! Lavret's in-memory document and offscreen rendering core.

#![forbid(unsafe_code)]

mod core;
mod diagnostics;
mod document;
mod error;
mod geometry;
mod handles;
mod input;
mod resources;
mod upstream;

pub use core::Core;
pub use diagnostics::{Diagnostic, DiagnosticCode, DiagnosticContext, DiagnosticSeverity};
pub use document::{CoreOptions, DocumentSource, PowerPreference, Viewport};
pub use error::{CoreError, HandleError};
pub use geometry::{Point, Rect, Size};
pub use handles::{DocumentHandle, NodeHandle, RenderTargetHandle, ResourceHandle};
pub use input::{
    InputEvent, Key, KeyLocation, Modifiers, NamedKey, PhysicalKey, PointerButton, PointerButtons,
    PointerId, PointerKind, UiEvent, UiEventData, UiEventKind,
};
pub use resources::{ResourceKind, ResourceSource};
