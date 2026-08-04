//! Explicit logical-resource resolution for parser-neutral XML documents.
//!
//! `xml-tools` owns bounded XML parsing. `resource-space` owns logical names
//! and immutable bytes. This adapter connects one document to explicitly
//! selected sibling resources without making either lower layer own XML URI
//! semantics or storage-provider behavior.

use resource_space::{
    FolderId, InMemoryResourceSpace, ResourceEntry, ResourceKey, ResourceName, ResourceSpaceError,
};
use thiserror::Error;
use xml_tools::{parse_xml_bytes, ExpandedName, XmlEvent, XmlOptions, XmlSourceId};

const XLINK_NAMESPACE: &str = "http://www.w3.org/1999/xlink";

/// One external XML reference resolved inside an explicit logical folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedXmlReference {
    element: ExpandedName,
    attribute: ExpandedName,
    reference: String,
    fragment: Option<String>,
    source: ResourceEntry,
}

impl ResolvedXmlReference {
    pub fn element(&self) -> &ExpandedName {
        &self.element
    }

    pub fn attribute(&self) -> &ExpandedName {
        &self.attribute
    }

    /// Returns the original external URI value, including a fragment when one
    /// was present.
    pub fn reference(&self) -> &str {
        &self.reference
    }

    /// Returns the fragment selector. Interpreting it remains format-specific.
    pub fn fragment(&self) -> Option<&str> {
        self.fragment.as_deref()
    }

    pub fn source(&self) -> &ResourceEntry {
        &self.source
    }
}

/// Reports XML ingestion and logical sibling-resolution failures.
#[derive(Debug, Error)]
pub enum ResourceXmlBridgeError {
    #[error("XML document `{document:?}` could not be parsed: {message}")]
    Parse {
        document: ResourceKey,
        message: String,
    },
    #[error("XML document `{document:?}` has an invalid external reference `{reference}")]
    InvalidReference {
        document: ResourceKey,
        reference: String,
    },
    #[error("resource-space lookup failed while resolving `{name}` for XML document `{document:?}`: {error}")]
    Lookup {
        document: ResourceKey,
        name: ResourceName,
        #[source]
        error: Box<ResourceSpaceError>,
    },
    #[error("XML document `{document:?}` references missing sibling resource `{name}")]
    MissingResource {
        document: ResourceKey,
        name: ResourceName,
    },
}

/// Resolves explicit external `href` and `xlink:href` attributes from a single
/// XML document against one caller-selected logical folder.
///
/// Local fragments (`#shape`), network URLs, data URLs, parent traversal, and
/// nested relative paths are not silently reinterpreted. The first profile
/// deliberately admits a one-folder sibling boundary such as
/// `symbols.svg#alert`; the fragment is preserved for the format-specific
/// consumer, while the resource name is resolved here.
pub fn resolve_xml_external_references_from_resource_space(
    space: &InMemoryResourceSpace,
    folder: FolderId,
    document: &ResourceEntry,
) -> Result<Vec<ResolvedXmlReference>, ResourceXmlBridgeError> {
    let document_key = document.key().clone();
    let events = parse_xml_bytes(XmlSourceId::new(0), document.bytes(), XmlOptions::default())
        .map_err(|error| ResourceXmlBridgeError::Parse {
            document: document_key.clone(),
            message: error.to_string(),
        })?;
    let mut resolved = Vec::new();

    for event in events {
        let XmlEvent::StartElement {
            name: element,
            attributes,
            ..
        } = event
        else {
            continue;
        };
        for attribute in attributes.into_iter().filter(is_href_attribute) {
            let (resource_reference, fragment) = split_external_reference(&attribute.value)
                .ok_or_else(|| ResourceXmlBridgeError::InvalidReference {
                    document: document_key.clone(),
                    reference: attribute.value.clone(),
                })?;
            let resource_name = ResourceName::parse(resource_reference, space.case_policy())
                .map_err(|_| ResourceXmlBridgeError::InvalidReference {
                    document: document_key.clone(),
                    reference: attribute.value.clone(),
                })?;
            let source = space
                .resource(folder, &resource_name)
                .map_err(|error| ResourceXmlBridgeError::Lookup {
                    document: document_key.clone(),
                    name: resource_name.clone(),
                    error: Box::new(error),
                })?
                .ok_or_else(|| ResourceXmlBridgeError::MissingResource {
                    document: document_key.clone(),
                    name: resource_name,
                })?;
            resolved.push(ResolvedXmlReference {
                element: element.clone(),
                attribute: attribute.name,
                reference: attribute.value,
                fragment,
                source,
            });
        }
    }
    Ok(resolved)
}

fn is_href_attribute(attribute: &xml_tools::XmlAttribute) -> bool {
    attribute.name.local_name == "href"
        && (attribute.name.namespace_uri.is_none()
            || attribute.name.namespace_uri.as_deref() == Some(XLINK_NAMESPACE))
}

fn split_external_reference(value: &str) -> Option<(&str, Option<String>)> {
    let value = value.trim();
    let (resource, fragment) = match value.split_once('#') {
        Some((resource, fragment)) => (resource, Some(fragment.to_owned())),
        None => (value, None),
    };
    (!resource.is_empty() && !resource.contains(['/', '\\', '?']) && !resource.contains("://"))
        .then_some((resource, fragment))
}

#[cfg(test)]
mod tests {
    use super::*;
    use resource_space::{
        AddressCasePolicy, ResourceMetadata, ResourceRootDescriptor, ResourceRootId, StoreId,
    };

    fn fixture_space() -> (InMemoryResourceSpace, FolderId) {
        let mut space =
            InMemoryResourceSpace::new(StoreId::from_u128(1), AddressCasePolicy::Sensitive);
        let folder = FolderId::from_u128(2);
        space
            .create_root(
                ResourceRootDescriptor::new(ResourceRootId::from_u128(3), "fixtures"),
                folder,
                ResourceMetadata::default(),
            )
            .expect("root");
        (space, folder)
    }

    fn insert(
        space: &mut InMemoryResourceSpace,
        folder: FolderId,
        name: &str,
        bytes: &[u8],
    ) -> ResourceEntry {
        space
            .insert_resource(
                folder,
                ResourceName::parse(name, AddressCasePolicy::Sensitive).expect("name"),
                bytes,
                ResourceMetadata::default(),
            )
            .expect("resource")
    }

    #[test]
    fn resolves_svg_xlink_and_preserves_the_format_specific_fragment() {
        let (mut space, folder) = fixture_space();
        let document = insert(
            &mut space,
            folder,
            "scene.svg",
            br#"<svg xmlns:xlink="http://www.w3.org/1999/xlink"><use xlink:href="symbols.svg#alert"/></svg>"#,
        );
        insert(&mut space, folder, "symbols.svg", b"<svg/>");

        let resolved =
            resolve_xml_external_references_from_resource_space(&space, folder, &document)
                .expect("reference");

        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].reference(), "symbols.svg#alert");
        assert_eq!(resolved[0].fragment(), Some("alert"));
        assert_eq!(resolved[0].source().name().as_str(), "symbols.svg");
    }

    #[test]
    fn missing_sibling_is_explicit_and_leaves_document_available() {
        let (mut space, folder) = fixture_space();
        let bytes = br#"<svg><image href="missing.png"/></svg>"#;
        let document = insert(&mut space, folder, "scene.svg", bytes);

        let error = resolve_xml_external_references_from_resource_space(&space, folder, &document)
            .expect_err("missing sibling");

        assert!(matches!(
            error,
            ResourceXmlBridgeError::MissingResource { .. }
        ));
        assert_eq!(document.bytes().as_ref(), bytes);
    }

    #[test]
    fn local_and_parent_references_do_not_escape_the_explicit_folder_boundary() {
        let (mut space, folder) = fixture_space();
        let local = insert(
            &mut space,
            folder,
            "local.svg",
            br##"<svg><use href="#shape"/></svg>"##,
        );
        let parent = insert(
            &mut space,
            folder,
            "parent.svg",
            br#"<svg><image href="../image.png"/></svg>"#,
        );

        for document in [&local, &parent] {
            let error =
                resolve_xml_external_references_from_resource_space(&space, folder, document)
                    .expect_err("unadmitted reference");
            assert!(matches!(
                error,
                ResourceXmlBridgeError::InvalidReference { .. }
            ));
        }
    }
}
