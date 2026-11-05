#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceKind {
    AuthorStylesheet,
    RasterImage,
    SvgDocument,
    Font,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceSource {
    pub uri: String,
    pub media_type: String,
    pub kind: ResourceKind,
    pub bytes: Vec<u8>,
}

impl ResourceSource {
    #[must_use]
    pub fn new(
        uri: impl Into<String>,
        media_type: impl Into<String>,
        kind: ResourceKind,
        bytes: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            uri: uri.into(),
            media_type: media_type.into(),
            kind,
            bytes: bytes.into(),
        }
    }
}
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use blitz_traits::net::{Bytes, NetHandler, NetProvider, Request};

use crate::{CoreError, HandleError, ResourceHandle};

struct ResourceSlot {
    generation: u32,
    value: Option<ResourceSource>,
}

#[derive(Default)]
struct ResourceState {
    slots: Vec<ResourceSlot>,
    by_uri: HashMap<String, u32>,
    missing_uris: Vec<String>,
}

#[derive(Default)]
pub(crate) struct ResourceStore {
    state: RwLock<ResourceState>,
}

impl ResourceStore {
    pub(crate) fn provider(self: &Arc<Self>) -> Arc<dyn NetProvider> {
        Arc::new(MemoryNetProvider {
            store: Arc::clone(self),
        })
    }

    pub(crate) fn insert(
        &self,
        owner: usize,
        source: ResourceSource,
    ) -> Result<ResourceHandle, CoreError> {
        let mut state = self.write()?;
        if state.by_uri.contains_key(&source.uri) {
            return Err(CoreError::InvalidArgument {
                name: "resource uri",
                reason: "the URI is already registered".to_owned(),
            });
        }

        let mut reusable = None;
        for (index, slot) in state.slots.iter().enumerate() {
            if slot.value.is_none() && slot.generation < u32::MAX {
                reusable = Some(index);
                break;
            }
        }

        let (slot_index, generation) = if let Some(index) = reusable {
            let slot = &mut state.slots[index];
            slot.generation += 1;
            slot.value = Some(source.clone());
            (index, slot.generation)
        } else {
            let index = state.slots.len();
            state.slots.push(ResourceSlot {
                generation: 1,
                value: Some(source.clone()),
            });
            (index, 1)
        };

        let slot = u32::try_from(slot_index).map_err(|_| CoreError::ResourceLimit {
            resource: source.uri.clone(),
            limit: "Lavret resource handle capacity".to_owned(),
        })?;
        state.by_uri.insert(source.uri, slot);
        Ok(ResourceHandle::new(owner, slot, generation))
    }

    pub(crate) fn replace(
        &self,
        owner: usize,
        handle: ResourceHandle,
        source: ResourceSource,
    ) -> Result<(String, String, ResourceKind, ResourceKind), CoreError> {
        if handle.owner != owner {
            return Err(CoreError::Handle(HandleError::Foreign));
        }

        let mut state = self.write()?;
        let slot_index =
            usize::try_from(handle.slot).map_err(|_| CoreError::Handle(HandleError::Invalid))?;
        let current = state
            .slots
            .get(slot_index)
            .ok_or(CoreError::Handle(HandleError::Invalid))?;
        if current.generation != handle.generation {
            return Err(CoreError::Handle(HandleError::Stale));
        }
        let current = current
            .value
            .as_ref()
            .ok_or(CoreError::Handle(HandleError::Destroyed))?;
        let old_uri = current.uri.clone();
        let old_kind = current.kind;
        let new_kind = source.kind;
        if old_uri != source.uri && state.by_uri.contains_key(&source.uri) {
            return Err(CoreError::InvalidArgument {
                name: "resource uri",
                reason: "the URI is already registered".to_owned(),
            });
        }

        state.by_uri.remove(&old_uri);
        let slot = state
            .slots
            .get_mut(slot_index)
            .ok_or(CoreError::Handle(HandleError::Invalid))?;
        slot.value = Some(source.clone());
        state.by_uri.insert(source.uri.clone(), handle.slot);
        Ok((old_uri, source.uri, old_kind, new_kind))
    }

    pub(crate) fn remove(
        &self,
        owner: usize,
        handle: ResourceHandle,
    ) -> Result<(String, ResourceKind), CoreError> {
        if handle.owner != owner {
            return Err(CoreError::Handle(HandleError::Foreign));
        }

        let mut state = self.write()?;
        let slot_index =
            usize::try_from(handle.slot).map_err(|_| CoreError::Handle(HandleError::Invalid))?;
        let slot = state
            .slots
            .get_mut(slot_index)
            .ok_or(CoreError::Handle(HandleError::Invalid))?;
        if slot.generation != handle.generation {
            return Err(CoreError::Handle(HandleError::Stale));
        }
        let source = slot
            .value
            .take()
            .ok_or(CoreError::Handle(HandleError::Destroyed))?;
        state.by_uri.remove(&source.uri);
        Ok((source.uri, source.kind))
    }

    pub(crate) fn take_missing(&self) -> Result<Vec<String>, CoreError> {
        let mut state = self.write()?;
        Ok(std::mem::take(&mut state.missing_uris))
    }

    fn write(&self) -> Result<std::sync::RwLockWriteGuard<'_, ResourceState>, CoreError> {
        self.state.write().map_err(|_| CoreError::DocumentFailure {
            stage: "resource registry",
            reason: "registry lock was poisoned".to_owned(),
        })
    }
}

struct MemoryNetProvider {
    store: Arc<ResourceStore>,
}

impl NetProvider for MemoryNetProvider {
    fn fetch(&self, _doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        let uri = request.url.as_str().to_owned();
        let result = self.store.state.write();
        let Ok(mut state) = result else {
            handler.error(uri, "resource registry lock was poisoned".to_owned());
            return;
        };

        let bytes = state.by_uri.get(&uri).and_then(|slot| {
            state
                .slots
                .get(*slot as usize)
                .and_then(|entry| entry.value.as_ref())
                .map(|source| source.bytes.clone())
        });
        if let Some(bytes) = bytes {
            drop(state);
            handler.bytes(uri.clone(), Bytes::copy_from_slice(&bytes));
        } else {
            if !state.missing_uris.contains(&uri) {
                state.missing_uris.push(uri.clone());
            }
            drop(state);
            handler.error(uri, "resource is not registered".to_owned());
        }
    }
}
