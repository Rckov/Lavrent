use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandleError {
    Invalid,
    Stale,
    Foreign,
    CrossDocument,
    Destroyed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoreError {
    InvalidArgument {
        name: &'static str,
        reason: String,
    },
    WrongThread,
    Handle(HandleError),
    InvalidMutation {
        reason: String,
    },
    UnsupportedResourceScheme {
        scheme: String,
    },
    ResourceLimit {
        resource: String,
        limit: String,
    },
    ResourceDecode {
        resource: String,
        reason: String,
    },
    UnresolvedDocument,
    DocumentFailure {
        stage: &'static str,
        reason: String,
    },
    AdapterUnavailable,
    DeviceCreation {
        reason: String,
    },
    GpuExecution {
        stage: &'static str,
        reason: String,
    },
    GpuReadback {
        reason: String,
    },
    DeviceLost,
    UnsupportedOperation {
        operation: &'static str,
        reason: String,
    },
}

impl Display for CoreError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArgument { name, reason } => write!(formatter, "invalid {name}: {reason}"),
            Self::WrongThread => formatter.write_str("core called from a non-owning thread"),
            Self::Handle(kind) => write!(formatter, "invalid handle: {kind:?}"),
            Self::InvalidMutation { reason } => write!(formatter, "invalid mutation: {reason}"),
            Self::UnsupportedResourceScheme { scheme } => {
                write!(formatter, "unsupported resource scheme: {scheme}")
            }
            Self::ResourceLimit { resource, limit } => {
                write!(formatter, "resource {resource} exceeds {limit}")
            }
            Self::ResourceDecode { resource, reason } => {
                write!(formatter, "cannot decode resource {resource}: {reason}")
            }
            Self::UnresolvedDocument => formatter.write_str("document has unresolved changes"),
            Self::DocumentFailure { stage, reason } => {
                write!(formatter, "document {stage} failed: {reason}")
            }
            Self::AdapterUnavailable => {
                formatter.write_str("no compatible GPU adapter is available")
            }
            Self::DeviceCreation { reason } => {
                write!(formatter, "GPU device creation failed: {reason}")
            }
            Self::GpuExecution { stage, reason } => {
                write!(formatter, "GPU {stage} failed: {reason}")
            }
            Self::GpuReadback { reason } => write!(formatter, "GPU readback failed: {reason}"),
            Self::DeviceLost => formatter.write_str("GPU device was lost"),
            Self::UnsupportedOperation { operation, reason } => {
                write!(formatter, "unsupported operation {operation}: {reason}")
            }
        }
    }
}

impl Error for CoreError {}
