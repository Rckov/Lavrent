#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DocumentHandle {
    pub(crate) owner: usize,
    pub(crate) slot: u32,
    pub(crate) generation: u32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeHandle {
    pub(crate) owner: usize,
    pub(crate) document_slot: u32,
    pub(crate) document_generation: u32,
    pub(crate) slot: u32,
    pub(crate) generation: u32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ResourceHandle {
    pub(crate) owner: usize,
    pub(crate) slot: u32,
    pub(crate) generation: u32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RenderTargetHandle {
    pub(crate) owner: usize,
    pub(crate) slot: u32,
    pub(crate) generation: u32,
}
