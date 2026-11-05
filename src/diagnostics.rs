use crate::{DocumentHandle, NodeHandle};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticSeverity {
    Information,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticCode {
    MissingResource,
    ResourceDecodeFailed,
    ResourceLimitExceeded,
    UnsupportedPaint,
    UnsupportedOperation,
    CapabilityUnavailable,
    DeviceLost,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticContext {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub document: Option<DocumentHandle>,
    pub node: Option<NodeHandle>,
    pub message: String,
    pub context: Vec<DiagnosticContext>,
}
