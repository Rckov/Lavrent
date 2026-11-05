use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::{self, ThreadId};
use std::time::Duration;

use blitz_dom::{EventDriver, LocalName, NodeId};
use blitz_html::HtmlDocument;

use crate::diagnostics::{Diagnostic, DiagnosticCode, DiagnosticSeverity};
use crate::input::{self, Dispatch, SemanticCapture, SemanticEvent};
use crate::renderer::RendererState;
use crate::resources::ResourceStore;
use crate::upstream;
use crate::{
    CoreError, CoreOptions, DocumentHandle, DocumentSource, HandleError, InputEvent, NodeHandle,
    Point, Rect, RenderResult, RenderTargetHandle, ResourceHandle, ResourceSource, Rgba8Readback,
    UiEvent, UiEventData, UiEventKind, Viewport,
};

static NEXT_CORE_OWNER: AtomicUsize = AtomicUsize::new(1);

pub struct Core {
    owner: usize,
    thread: ThreadId,
    renderer: RendererState,
    documents: Vec<DocumentSlot>,
    resources: Arc<ResourceStore>,
    ui_events: Vec<UiEvent>,
    diagnostics: Vec<Diagnostic>,
}

struct DocumentSlot {
    generation: u32,
    value: Option<DocumentState>,
}

pub(crate) struct DocumentState {
    pub(crate) document: HtmlDocument,
    viewport: Viewport,
    nodes: NodeStore,
    pub(crate) dirty: bool,
    input_ready: bool,
}

impl DocumentState {
    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.input_ready = false;
    }

    fn mark_input_dirty(&mut self) {
        self.dirty = true;
    }

    fn mark_resolved(&mut self) {
        self.dirty = false;
        self.input_ready = true;
    }
}

struct NodeSlot {
    generation: u32,
    value: Option<NodeId>,
}

#[derive(Default)]
struct NodeStore {
    slots: Vec<NodeSlot>,
    by_id: HashMap<NodeId, u32>,
}

impl Core {
    pub fn new(options: CoreOptions) -> Result<Self, CoreError> {
        let owner = NEXT_CORE_OWNER
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map_err(|_| CoreError::ResourceLimit {
                resource: "core owner identity".to_owned(),
                limit: "core owner-identity space".to_owned(),
            })?;
        Ok(Self {
            owner,
            thread: thread::current().id(),
            renderer: RendererState::new(options.power_preference),
            documents: Vec::new(),
            resources: Arc::new(ResourceStore::default()),
            ui_events: Vec::new(),
            diagnostics: Vec::new(),
        })
    }

    pub fn create_document(
        &mut self,
        mut source: DocumentSource,
        viewport: Viewport,
    ) -> Result<DocumentHandle, CoreError> {
        self.check_thread()?;
        validate_viewport(viewport)?;
        validate_document_source(&mut source)?;
        let document = upstream::create_document(&source, viewport, self.resources.provider())?;
        let state = DocumentState {
            document,
            viewport,
            nodes: NodeStore::default(),
            dirty: true,
            input_ready: false,
        };
        self.insert_document(state)
    }

    pub fn destroy_document(&mut self, handle: DocumentHandle) -> Result<(), CoreError> {
        self.check_thread()?;
        let index = self.document_index(handle)?;
        let slot = self
            .documents
            .get_mut(index)
            .ok_or(CoreError::Handle(HandleError::Invalid))?;
        if slot.value.take().is_none() {
            return Err(CoreError::Handle(HandleError::Destroyed));
        }
        Ok(())
    }

    pub fn replace_document_source(
        &mut self,
        handle: DocumentHandle,
        mut source: DocumentSource,
    ) -> Result<(), CoreError> {
        self.check_thread()?;
        validate_document_source(&mut source)?;
        let viewport = self.document(handle)?.viewport;
        let replacement = upstream::create_document(&source, viewport, self.resources.provider())?;
        let state = self.document_mut(handle)?;
        state.document = replacement;
        state.nodes.invalidate_all();
        state.mark_dirty();
        Ok(())
    }

    pub fn find_element_by_id(
        &mut self,
        handle: DocumentHandle,
        id: &str,
    ) -> Result<Option<NodeHandle>, CoreError> {
        self.check_thread()?;
        if id.is_empty() {
            return Ok(None);
        }
        let owner = self.owner;
        let state = self.document_mut(handle)?;
        let Some(node_id) = state.document.get_element_by_id(id) else {
            return Ok(None);
        };
        state.nodes.handle_for(owner, handle, node_id).map(Some)
    }

    pub fn create_element(
        &mut self,
        document: DocumentHandle,
        name: &str,
    ) -> Result<NodeHandle, CoreError> {
        self.check_thread()?;
        validate_markup_name(name, "element name")?;
        let owner = self.owner;
        let state = self.document_mut(document)?;
        let node_id = state
            .document
            .mutate()
            .create_element(upstream::html_name(name), Vec::new());
        state.mark_dirty();
        state.nodes.handle_for(owner, document, node_id)
    }

    pub fn create_text(
        &mut self,
        document: DocumentHandle,
        text: &str,
    ) -> Result<NodeHandle, CoreError> {
        self.check_thread()?;
        let owner = self.owner;
        let state = self.document_mut(document)?;
        let node_id = state.document.mutate().create_text_node(text);
        state.mark_dirty();
        state.nodes.handle_for(owner, document, node_id)
    }

    pub fn append_node(
        &mut self,
        document: DocumentHandle,
        parent: NodeHandle,
        child: NodeHandle,
    ) -> Result<(), CoreError> {
        self.check_thread()?;
        let owner = self.owner;
        let state = self.document_mut(document)?;
        let parent_id = state.nodes.resolve(owner, document, parent)?;
        let child_id = state.nodes.resolve(owner, document, child)?;
        validate_parent(&state.document, parent_id)?;
        validate_insertion(&state.document, parent_id, child_id)?;
        if state
            .document
            .get_node(child_id)
            .and_then(|node| node.parent)
            .is_some()
        {
            return Err(CoreError::InvalidMutation {
                reason: "node already has a parent; use move_node".to_owned(),
            });
        }
        state
            .document
            .mutate()
            .append_children(parent_id, &[child_id]);
        state.mark_dirty();
        Ok(())
    }

    pub fn insert_node_before(
        &mut self,
        document: DocumentHandle,
        anchor: NodeHandle,
        node: NodeHandle,
    ) -> Result<(), CoreError> {
        self.check_thread()?;
        let owner = self.owner;
        let state = self.document_mut(document)?;
        let anchor_id = state.nodes.resolve(owner, document, anchor)?;
        let node_id = state.nodes.resolve(owner, document, node)?;
        let parent_id = state
            .document
            .get_node(anchor_id)
            .and_then(|entry| entry.parent)
            .ok_or_else(|| CoreError::InvalidMutation {
                reason: "anchor has no parent".to_owned(),
            })?;
        validate_insertion(&state.document, parent_id, node_id)?;
        if state
            .document
            .get_node(node_id)
            .and_then(|entry| entry.parent)
            .is_some()
        {
            return Err(CoreError::InvalidMutation {
                reason: "node already has a parent; use move_node".to_owned(),
            });
        }
        state
            .document
            .mutate()
            .insert_nodes_before(anchor_id, &[node_id]);
        state.mark_dirty();
        Ok(())
    }

    pub fn move_node(
        &mut self,
        document: DocumentHandle,
        node: NodeHandle,
        new_parent: NodeHandle,
        before: Option<NodeHandle>,
    ) -> Result<(), CoreError> {
        self.check_thread()?;
        let owner = self.owner;
        let state = self.document_mut(document)?;
        let node_id = state.nodes.resolve(owner, document, node)?;
        let parent_id = state.nodes.resolve(owner, document, new_parent)?;
        validate_parent(&state.document, parent_id)?;
        if state
            .document
            .get_node(node_id)
            .and_then(|entry| entry.parent)
            .is_none()
        {
            return Err(CoreError::InvalidMutation {
                reason: "detached node must be inserted before it can be moved".to_owned(),
            });
        }
        validate_insertion(&state.document, parent_id, node_id)?;
        let before_id = if let Some(before) = before {
            let before_id = state.nodes.resolve(owner, document, before)?;
            if before_id == node_id {
                return Err(CoreError::InvalidMutation {
                    reason: "node cannot be moved before itself".to_owned(),
                });
            }
            let actual_parent = state
                .document
                .get_node(before_id)
                .and_then(|entry| entry.parent);
            if actual_parent != Some(parent_id) {
                return Err(CoreError::InvalidMutation {
                    reason: "the insertion anchor is not a child of the new parent".to_owned(),
                });
            }
            Some(before_id)
        } else {
            None
        };

        let mut mutator = state.document.mutate();
        if let Some(before_id) = before_id {
            mutator.insert_nodes_before(before_id, &[node_id]);
        } else {
            mutator.append_children(parent_id, &[node_id]);
        }
        drop(mutator);
        state.mark_dirty();
        Ok(())
    }

    pub fn remove_node(
        &mut self,
        document: DocumentHandle,
        node: NodeHandle,
    ) -> Result<(), CoreError> {
        self.check_thread()?;
        let owner = self.owner;
        let state = self.document_mut(document)?;
        let node_id = state.nodes.resolve(owner, document, node)?;
        if state
            .document
            .get_node(node_id)
            .and_then(|entry| entry.parent)
            .is_none()
        {
            return Err(CoreError::InvalidMutation {
                reason: "document root or detached node cannot be removed".to_owned(),
            });
        }
        let (document_ref, nodes) = (&mut state.document, &mut state.nodes);
        let mut on_drop = |id| nodes.invalidate(id);
        let removed = document_ref
            .mutate()
            .remove_and_drop_node_with(node_id, &mut on_drop);
        if removed.is_none() {
            return Err(CoreError::DocumentFailure {
                stage: "remove node",
                reason: "Blitz did not remove the validated node".to_owned(),
            });
        }
        state.mark_dirty();
        Ok(())
    }

    pub fn set_attribute(
        &mut self,
        document: DocumentHandle,
        node: NodeHandle,
        name: &str,
        value: &str,
    ) -> Result<(), CoreError> {
        self.check_thread()?;
        validate_markup_name(name, "attribute name")?;
        let owner = self.owner;
        let state = self.document_mut(document)?;
        let node_id = state.nodes.resolve(owner, document, node)?;
        validate_element(&state.document, node_id)?;
        state
            .document
            .mutate()
            .set_attribute(node_id, upstream::attribute_name(name), value);
        state.mark_dirty();
        Ok(())
    }

    pub fn remove_attribute(
        &mut self,
        document: DocumentHandle,
        node: NodeHandle,
        name: &str,
    ) -> Result<(), CoreError> {
        self.check_thread()?;
        validate_markup_name(name, "attribute name")?;
        let owner = self.owner;
        let state = self.document_mut(document)?;
        let node_id = state.nodes.resolve(owner, document, node)?;
        validate_element(&state.document, node_id)?;
        state
            .document
            .mutate()
            .clear_attribute(node_id, upstream::attribute_name(name));
        state.mark_dirty();
        Ok(())
    }

    pub fn replace_text_content(
        &mut self,
        document: DocumentHandle,
        node: NodeHandle,
        text: &str,
    ) -> Result<(), CoreError> {
        self.check_thread()?;
        let owner = self.owner;
        let state = self.document_mut(document)?;
        let node_id = state.nodes.resolve(owner, document, node)?;
        let Some(target) = state.document.get_node(node_id) else {
            return Err(CoreError::Handle(HandleError::Destroyed));
        };
        if target.is_text_node() {
            state.document.mutate().set_node_text(node_id, text);
        } else if target.is_element() {
            let children = target.children.clone();
            let (document_ref, nodes) = (&mut state.document, &mut state.nodes);
            let mut mutator = document_ref.mutate();
            for child in children {
                let mut on_drop = |id| nodes.invalidate(id);
                let _ = mutator.remove_and_drop_node_with(child, &mut on_drop);
            }
            if !text.is_empty() {
                let text_node = mutator.create_text_node(text);
                mutator.append_children(node_id, &[text_node]);
            }
        } else {
            return Err(CoreError::InvalidMutation {
                reason: "text content can be replaced only on element or text nodes".to_owned(),
            });
        }
        state.mark_dirty();
        Ok(())
    }

    pub fn set_viewport(
        &mut self,
        document: DocumentHandle,
        viewport: Viewport,
    ) -> Result<(), CoreError> {
        self.check_thread()?;
        validate_viewport(viewport)?;
        let state = self.document_mut(document)?;
        upstream::set_viewport(&mut state.document, viewport)?;
        state.viewport = viewport;
        state.mark_dirty();
        Ok(())
    }

    pub fn create_render_target(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<RenderTargetHandle, CoreError> {
        self.check_thread()?;
        let (target, diagnostic) = self.renderer.create_target(self.owner, width, height)?;
        if let Some(diagnostic) = diagnostic {
            self.diagnostics.push(diagnostic);
        }
        Ok(target)
    }

    pub fn resize_render_target(
        &mut self,
        target: RenderTargetHandle,
        width: u32,
        height: u32,
    ) -> Result<(), CoreError> {
        self.check_thread()?;
        self.renderer
            .resize_target(self.owner, target, width, height)
    }

    pub fn destroy_render_target(&mut self, target: RenderTargetHandle) -> Result<(), CoreError> {
        self.check_thread()?;
        self.renderer.destroy_target(self.owner, target)
    }

    pub fn render(
        &mut self,
        document: DocumentHandle,
        target: RenderTargetHandle,
    ) -> Result<RenderResult, CoreError> {
        self.check_thread()?;
        let index = self.document_index(document)?;
        let state = self
            .documents
            .get_mut(index)
            .and_then(|slot| slot.value.as_mut())
            .ok_or(CoreError::Handle(HandleError::Destroyed))?;
        if state.dirty {
            return Err(CoreError::UnresolvedDocument);
        }
        self.renderer
            .render(self.owner, target, &mut state.document)
    }

    pub fn readback_rgba8(
        &mut self,
        target: RenderTargetHandle,
    ) -> Result<Rgba8Readback, CoreError> {
        self.check_thread()?;
        self.renderer.readback(self.owner, target)
    }

    pub fn resolve(
        &mut self,
        document: DocumentHandle,
        animation_time: Duration,
    ) -> Result<(), CoreError> {
        self.check_thread()?;
        let state = self.document_mut(document)?;
        state.document.resolve(animation_time.as_secs_f64());
        state.mark_resolved();
        self.collect_missing_resource_diagnostics(Some(document))
    }

    pub fn border_box(
        &self,
        document: DocumentHandle,
        node: NodeHandle,
    ) -> Result<Rect, CoreError> {
        self.check_thread()?;
        let state = self.document(document)?;
        if state.dirty {
            return Err(CoreError::UnresolvedDocument);
        }
        let node_id = state.nodes.resolve(self.owner, document, node)?;
        let rect = state
            .document
            .get_client_bounding_rect(node_id)
            .ok_or_else(|| CoreError::DocumentFailure {
                stage: "geometry query",
                reason: "node has no border-box geometry".to_owned(),
            })?;
        let scroll = state.document.viewport_scroll();
        Ok(Rect::new(
            rect.x + scroll.x,
            rect.y + scroll.y,
            rect.width,
            rect.height,
        ))
    }

    pub fn hit_test(
        &mut self,
        document: DocumentHandle,
        point: Point,
    ) -> Result<Vec<NodeHandle>, CoreError> {
        self.check_thread()?;
        validate_point(point)?;
        let owner = self.owner;
        let state = self.document_mut(document)?;
        if state.dirty {
            return Err(CoreError::UnresolvedDocument);
        }
        let scroll = state.document.viewport_scroll();
        let viewport_point = Point::new(point.x - scroll.x, point.y - scroll.y);
        validate_point(viewport_point)?;
        let ids = state
            .document
            .elements_from_point(viewport_point.x as f32, viewport_point.y as f32);
        let mut handles = Vec::with_capacity(ids.len());
        for id in ids {
            handles.push(state.nodes.handle_for(owner, document, id)?);
        }
        Ok(handles)
    }

    pub fn register_resource(
        &mut self,
        source: ResourceSource,
    ) -> Result<ResourceHandle, CoreError> {
        self.check_thread()?;
        let source = validate_resource_source(source)?;
        let uri = source.uri.clone();
        let handle = self.resources.insert(self.owner, source)?;
        self.invalidate_resource(&uri);
        Ok(handle)
    }

    pub fn dispatch_input(
        &mut self,
        document: DocumentHandle,
        event: InputEvent,
    ) -> Result<(), CoreError> {
        self.check_thread()?;
        let owner = self.owner;
        let mut translated_events = Vec::new();
        {
            let state = self.document_mut(document)?;
            if !state.input_ready {
                return Err(CoreError::UnresolvedDocument);
            }
            let scroll = state.document.viewport_scroll();
            match input::translate(event, Point::new(scroll.x, scroll.y))? {
                Dispatch::Upstream(event) => {
                    let capture = SemanticCapture::new(&mut translated_events);
                    EventDriver::new(&mut state.document, capture).handle_ui_event(event);
                    state.mark_input_dirty();
                }
                Dispatch::FocusGained => {}
                Dispatch::FocusLost => {
                    let focused = state.document.get_focussed_node_id().filter(|id| {
                        state
                            .document
                            .get_node(*id)
                            .is_some_and(|node| node.is_focussed())
                    });
                    state.document.clear_focus();
                    if let Some(target) = focused {
                        translated_events.push(SemanticEvent {
                            target,
                            kind: UiEventKind::Blur,
                            data: UiEventData::None,
                        });
                    }
                    state.mark_input_dirty();
                }
            }

            let mut public_events = Vec::with_capacity(translated_events.len());
            for event in translated_events {
                public_events.push(UiEvent {
                    target: state.nodes.handle_for(owner, document, event.target)?,
                    kind: event.kind,
                    data: event.data,
                });
            }
            self.ui_events.extend(public_events);
        }
        Ok(())
    }

    pub fn drain_ui_events(&mut self) -> Result<Vec<UiEvent>, CoreError> {
        self.check_thread()?;
        Ok(std::mem::take(&mut self.ui_events))
    }

    pub fn replace_resource(
        &mut self,
        handle: ResourceHandle,
        source: ResourceSource,
    ) -> Result<(), CoreError> {
        self.check_thread()?;
        let source = validate_resource_source(source)?;
        let (old_uri, new_uri, old_kind, new_kind) =
            self.resources.replace(self.owner, handle, source)?;
        self.invalidate_resource(&old_uri);
        if old_uri != new_uri {
            self.invalidate_resource(&new_uri);
        }
        if old_kind != crate::ResourceKind::AuthorStylesheet
            || new_kind != crate::ResourceKind::AuthorStylesheet
        {
            self.diagnostics.push(Diagnostic {
                code: DiagnosticCode::CapabilityUnavailable,
                severity: DiagnosticSeverity::Warning,
                document: None,
                node: None,
                message: "The pinned document engine cannot evict decoded resource caches"
                    .to_owned(),
                context: vec![crate::DiagnosticContext {
                    key: "uri".to_owned(),
                    value: old_uri,
                }],
            });
        }
        Ok(())
    }

    pub fn remove_resource(&mut self, handle: ResourceHandle) -> Result<(), CoreError> {
        self.check_thread()?;
        let (uri, kind) = self.resources.remove(self.owner, handle)?;
        self.invalidate_resource(&uri);
        self.diagnostics.push(Diagnostic {
            code: DiagnosticCode::MissingResource,
            severity: DiagnosticSeverity::Warning,
            document: None,
            node: None,
            message: "A registered resource was removed".to_owned(),
            context: vec![crate::DiagnosticContext {
                key: "uri".to_owned(),
                value: uri.clone(),
            }],
        });
        if kind != crate::ResourceKind::AuthorStylesheet {
            self.diagnostics.push(Diagnostic {
                code: DiagnosticCode::CapabilityUnavailable,
                severity: DiagnosticSeverity::Warning,
                document: None,
                node: None,
                message: "The pinned document engine cannot evict decoded resource caches"
                    .to_owned(),
                context: vec![crate::DiagnosticContext {
                    key: "uri".to_owned(),
                    value: uri,
                }],
            });
        }
        Ok(())
    }

    pub fn drain_diagnostics(&mut self) -> Result<Vec<Diagnostic>, CoreError> {
        self.check_thread()?;
        Ok(std::mem::take(&mut self.diagnostics))
    }

    pub(crate) fn check_thread(&self) -> Result<(), CoreError> {
        if thread::current().id() == self.thread {
            Ok(())
        } else {
            Err(CoreError::WrongThread)
        }
    }

    pub(crate) fn document(&self, handle: DocumentHandle) -> Result<&DocumentState, CoreError> {
        let index = self.document_index(handle)?;
        self.documents
            .get(index)
            .ok_or(CoreError::Handle(HandleError::Invalid))?
            .value
            .as_ref()
            .ok_or(CoreError::Handle(HandleError::Destroyed))
    }

    pub(crate) fn document_mut(
        &mut self,
        handle: DocumentHandle,
    ) -> Result<&mut DocumentState, CoreError> {
        let index = self.document_index(handle)?;
        self.documents
            .get_mut(index)
            .ok_or(CoreError::Handle(HandleError::Invalid))?
            .value
            .as_mut()
            .ok_or(CoreError::Handle(HandleError::Destroyed))
    }

    fn insert_document(&mut self, state: DocumentState) -> Result<DocumentHandle, CoreError> {
        for (index, slot) in self.documents.iter_mut().enumerate() {
            if slot.value.is_none() && slot.generation < u32::MAX {
                slot.generation += 1;
                slot.value = Some(state);
                let index = u32::try_from(index).map_err(|_| CoreError::ResourceLimit {
                    resource: "document".to_owned(),
                    limit: "document handle capacity".to_owned(),
                })?;
                return Ok(DocumentHandle::new(self.owner, index, slot.generation));
            }
        }
        let index = u32::try_from(self.documents.len()).map_err(|_| CoreError::ResourceLimit {
            resource: "document".to_owned(),
            limit: "document handle capacity".to_owned(),
        })?;
        self.documents.push(DocumentSlot {
            generation: 1,
            value: Some(state),
        });
        Ok(DocumentHandle::new(self.owner, index, 1))
    }

    fn document_index(&self, handle: DocumentHandle) -> Result<usize, CoreError> {
        if handle.owner != self.owner {
            return Err(CoreError::Handle(HandleError::Foreign));
        }
        let index =
            usize::try_from(handle.slot).map_err(|_| CoreError::Handle(HandleError::Invalid))?;
        let slot = self
            .documents
            .get(index)
            .ok_or(CoreError::Handle(HandleError::Invalid))?;
        if slot.generation != handle.generation {
            return Err(CoreError::Handle(HandleError::Stale));
        }
        Ok(index)
    }

    fn invalidate_resource(&mut self, uri: &str) {
        for slot in &mut self.documents {
            let Some(state) = slot.value.as_mut() else {
                continue;
            };
            state.document.reload_resource_by_href(uri);
            let mut attributes = Vec::new();
            for (node_id, node) in state.document.tree().iter() {
                let Some(element) = node.element_data() else {
                    continue;
                };
                for name in ["src", "href"] {
                    let local = LocalName::from(name);
                    if element.attr(local) == Some(uri) {
                        attributes.push((node_id, name));
                    }
                }
            }
            if !attributes.is_empty() {
                let mut mutator = state.document.mutate();
                for (node_id, name) in attributes {
                    mutator.set_attribute(node_id, upstream::attribute_name(name), uri);
                }
            }
            state.mark_dirty();
        }
    }

    fn collect_missing_resource_diagnostics(
        &mut self,
        document: Option<DocumentHandle>,
    ) -> Result<(), CoreError> {
        for uri in self.resources.take_missing()? {
            self.diagnostics.push(Diagnostic {
                code: DiagnosticCode::MissingResource,
                severity: DiagnosticSeverity::Warning,
                document,
                node: None,
                message: "A document resource could not be resolved".to_owned(),
                context: vec![crate::DiagnosticContext {
                    key: "uri".to_owned(),
                    value: uri,
                }],
            });
        }
        Ok(())
    }
}

impl NodeStore {
    fn handle_for(
        &mut self,
        owner: usize,
        document: DocumentHandle,
        node_id: NodeId,
    ) -> Result<NodeHandle, CoreError> {
        if let Some(slot_index) = self.by_id.get(&node_id).copied() {
            let slot = self
                .slots
                .get(slot_index as usize)
                .ok_or(CoreError::Handle(HandleError::Invalid))?;
            return Ok(NodeHandle::new(
                owner,
                document,
                slot_index,
                slot.generation,
            ));
        }

        for (index, slot) in self.slots.iter_mut().enumerate() {
            if slot.value.is_none() && slot.generation < u32::MAX {
                slot.generation += 1;
                slot.value = Some(node_id);
                let index = u32::try_from(index).map_err(|_| CoreError::ResourceLimit {
                    resource: "document nodes".to_owned(),
                    limit: "node handle capacity".to_owned(),
                })?;
                self.by_id.insert(node_id, index);
                return Ok(NodeHandle::new(owner, document, index, slot.generation));
            }
        }

        let index = u32::try_from(self.slots.len()).map_err(|_| CoreError::ResourceLimit {
            resource: "document nodes".to_owned(),
            limit: "node handle capacity".to_owned(),
        })?;
        self.slots.push(NodeSlot {
            generation: 1,
            value: Some(node_id),
        });
        self.by_id.insert(node_id, index);
        Ok(NodeHandle::new(owner, document, index, 1))
    }

    fn resolve(
        &self,
        owner: usize,
        document: DocumentHandle,
        handle: NodeHandle,
    ) -> Result<NodeId, CoreError> {
        if handle.owner != owner {
            return Err(CoreError::Handle(HandleError::Foreign));
        }
        if handle.document_slot != document.slot
            || handle.document_generation != document.generation
        {
            return Err(CoreError::Handle(HandleError::CrossDocument));
        }
        let index =
            usize::try_from(handle.slot).map_err(|_| CoreError::Handle(HandleError::Invalid))?;
        let slot = self
            .slots
            .get(index)
            .ok_or(CoreError::Handle(HandleError::Invalid))?;
        if slot.generation != handle.generation {
            return Err(CoreError::Handle(HandleError::Stale));
        }
        slot.value.ok_or(CoreError::Handle(HandleError::Destroyed))
    }

    fn invalidate(&mut self, node_id: NodeId) {
        let Some(index) = self.by_id.remove(&node_id) else {
            return;
        };
        if let Some(slot) = self.slots.get_mut(index as usize) {
            slot.value = None;
        }
    }

    fn invalidate_all(&mut self) {
        self.by_id.clear();
        for slot in &mut self.slots {
            slot.value = None;
            slot.generation = slot.generation.saturating_add(1);
        }
    }
}

fn validate_document_source(source: &mut DocumentSource) -> Result<(), CoreError> {
    if let Some(base_uri) = &mut source.base_resource_uri {
        let parsed = url::Url::parse(base_uri).map_err(|error| CoreError::InvalidArgument {
            name: "base resource URI",
            reason: error.to_string(),
        })?;
        if parsed.scheme() != "memory" {
            return Err(CoreError::UnsupportedResourceScheme {
                scheme: parsed.scheme().to_owned(),
            });
        }
        *base_uri = parsed.to_string();
    }
    Ok(())
}

fn validate_resource_source(mut source: ResourceSource) -> Result<ResourceSource, CoreError> {
    if source.media_type.trim().is_empty() {
        return Err(CoreError::InvalidArgument {
            name: "resource media type",
            reason: "media type must not be empty".to_owned(),
        });
    }
    let parsed = url::Url::parse(&source.uri).map_err(|error| CoreError::InvalidArgument {
        name: "resource URI",
        reason: error.to_string(),
    })?;
    if parsed.scheme() != "memory" {
        return Err(CoreError::UnsupportedResourceScheme {
            scheme: parsed.scheme().to_owned(),
        });
    }
    source.uri = parsed.to_string();
    Ok(source)
}

fn validate_viewport(viewport: Viewport) -> Result<(), CoreError> {
    for (name, value) in [
        ("viewport width", viewport.width),
        ("viewport height", viewport.height),
        ("viewport scale factor", viewport.scale_factor),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(CoreError::InvalidArgument {
                name,
                reason: "value must be finite and positive".to_owned(),
            });
        }
        let upstream_value = value as f32;
        if !upstream_value.is_finite() || upstream_value <= 0.0 {
            return Err(CoreError::InvalidArgument {
                name,
                reason: "value is not representable by the document engine".to_owned(),
            });
        }
    }
    Ok(())
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
            name: "point",
            reason: "coordinates must be finite and representable by the document engine"
                .to_owned(),
        })
    }
}

fn validate_markup_name(name: &str, argument: &'static str) -> Result<(), CoreError> {
    if name.is_empty()
        || name
            .chars()
            .any(|character| character.is_ascii_whitespace() || character == '\0')
    {
        Err(CoreError::InvalidArgument {
            name: argument,
            reason: "name must be non-empty and contain no ASCII whitespace or NUL".to_owned(),
        })
    } else {
        Ok(())
    }
}

fn validate_parent(document: &HtmlDocument, node_id: NodeId) -> Result<(), CoreError> {
    validate_element(document, node_id).map_err(|_| CoreError::InvalidMutation {
        reason: "parent must be an element".to_owned(),
    })
}

fn validate_element(document: &HtmlDocument, node_id: NodeId) -> Result<(), CoreError> {
    let is_element = document
        .get_node(node_id)
        .is_some_and(|node| node.is_element());
    if is_element {
        Ok(())
    } else {
        Err(CoreError::InvalidMutation {
            reason: "operation requires an element node".to_owned(),
        })
    }
}

fn validate_insertion(
    document: &HtmlDocument,
    parent_id: NodeId,
    child_id: NodeId,
) -> Result<(), CoreError> {
    if parent_id == child_id || upstream::is_descendant(document, child_id, parent_id) {
        Err(CoreError::InvalidMutation {
            reason: "node cannot be inserted into itself or its descendants".to_owned(),
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_calls_reject_a_non_owning_thread() {
        let other_thread = match thread::spawn(|| thread::current().id()).join() {
            Ok(thread_id) => thread_id,
            Err(payload) => std::panic::resume_unwind(payload),
        };
        let mut core = match Core::new(CoreOptions::default()) {
            Ok(core) => core,
            Err(error) => std::panic::resume_unwind(Box::new(error)),
        };
        core.thread = other_thread;
        assert_eq!(core.drain_diagnostics(), Err(CoreError::WrongThread));
    }
}
