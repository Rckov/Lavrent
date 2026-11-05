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
