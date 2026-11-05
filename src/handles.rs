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

impl DocumentHandle {
    pub(crate) const fn new(owner: usize, slot: u32, generation: u32) -> Self {
        Self {
            owner,
            slot,
            generation,
        }
    }
}

impl NodeHandle {
    pub(crate) const fn new(
        owner: usize,
        document: DocumentHandle,
        slot: u32,
        generation: u32,
    ) -> Self {
        Self {
            owner,
            document_slot: document.slot,
            document_generation: document.generation,
            slot,
            generation,
        }
    }
}

impl ResourceHandle {
    pub(crate) const fn new(owner: usize, slot: u32, generation: u32) -> Self {
        Self {
            owner,
            slot,
            generation,
        }
    }
}
