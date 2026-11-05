use std::sync::Arc;

use blitz_dom::{DocumentConfig, LocalName, Namespace, NodeId, QualName};
use blitz_html::HtmlDocument;
use blitz_traits::net::NetProvider;
use blitz_traits::shell::Viewport as BlitzViewport;

use crate::{ColorScheme, CoreError, DocumentSource, Viewport};

pub(crate) fn create_document(
    source: &DocumentSource,
    viewport: Viewport,
    provider: Arc<dyn NetProvider>,
) -> Result<HtmlDocument, CoreError> {
    let config = DocumentConfig {
        viewport: Some(to_blitz_viewport(viewport)?),
        base_url: Some(
            source
                .base_resource_uri
                .clone()
                .unwrap_or_else(|| "memory:/".to_owned()),
        ),
        net_provider: Some(provider),
        ..DocumentConfig::default()
    };
    let mut document = HtmlDocument::from_html(&source.html, config);
    inject_author_css(&mut document, &source.author_css)?;
    Ok(document)
}

pub(crate) fn set_viewport(
    document: &mut HtmlDocument,
    viewport: Viewport,
) -> Result<(), CoreError> {
    document.set_viewport(to_blitz_viewport(viewport)?);
    Ok(())
}

pub(crate) fn html_name(name: &str) -> QualName {
    QualName::new(
        None,
        Namespace::from("http://www.w3.org/1999/xhtml"),
        LocalName::from(name),
    )
}

pub(crate) fn attribute_name(name: &str) -> QualName {
    QualName::new(None, Namespace::from(""), LocalName::from(name))
}

fn inject_author_css(document: &mut HtmlDocument, sheets: &[String]) -> Result<(), CoreError> {
    if sheets.is_empty() {
        return Ok(());
    }

    let parent = document
        .query_selector("head")
        .ok()
        .flatten()
        .or_else(|| document.try_root_element().map(|node| node.id))
        .ok_or_else(|| CoreError::DocumentFailure {
            stage: "author stylesheet",
            reason: "parsed document has no element that can own a style element".to_owned(),
        })?;

    let mut mutator = document.mutate();
    for css in sheets {
        let style = mutator.create_element(html_name("style"), Vec::new());
        let text = mutator.create_text_node(css);
        mutator.append_children(style, &[text]);
        mutator.append_children(parent, &[style]);
    }
    Ok(())
}

fn to_blitz_viewport(viewport: Viewport) -> Result<BlitzViewport, CoreError> {
    let width = physical_dimension(viewport.width, viewport.scale_factor, "viewport width")?;
    let height = physical_dimension(viewport.height, viewport.scale_factor, "viewport height")?;
    Ok(BlitzViewport::new(
        width,
        height,
        viewport.scale_factor as f32,
        match viewport.color_scheme {
            ColorScheme::Light => blitz_traits::shell::ColorScheme::Light,
            ColorScheme::Dark => blitz_traits::shell::ColorScheme::Dark,
        },
    ))
}

fn physical_dimension(logical: f64, scale: f64, name: &'static str) -> Result<u32, CoreError> {
    let physical = (logical * scale).round();
    if physical < 1.0 || physical > f64::from(u32::MAX) {
        return Err(CoreError::InvalidArgument {
            name,
            reason: "rounded physical dimension is outside the u32 range".to_owned(),
        });
    }
    Ok(physical as u32)
}

pub(crate) fn is_descendant(document: &HtmlDocument, ancestor: NodeId, node: NodeId) -> bool {
    let mut current = document.get_node(node).and_then(|entry| entry.parent);
    while let Some(parent) = current {
        if parent == ancestor {
            return true;
        }
        current = document.get_node(parent).and_then(|entry| entry.parent);
    }
    false
}
