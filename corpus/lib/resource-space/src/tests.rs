use crate::{
    AddressCasePolicy, ResourceAddress, ResourceAddressError, ResourceEntry, ResourceKey,
    ResourceRootDescriptor, ResourceRootId, ResourceVisibility, StoreId, VisibilityQuery,
};

fn name(value: &str) -> crate::ResourceName {
    crate::ResourceName::parse(value, AddressCasePolicy::Sensitive).expect("resource name")
}

fn space_with_root() -> (
    crate::InMemoryResourceSpace,
    ResourceRootId,
    crate::FolderId,
) {
    let root = ResourceRootId::from_u128(10);
    let root_folder = crate::FolderId::from_u128(100);
    let mut space =
        crate::InMemoryResourceSpace::new(StoreId::from_u128(1), AddressCasePolicy::Sensitive);
    space
        .create_root(
            ResourceRootDescriptor::new(root, "Project assets"),
            root_folder,
            crate::ResourceMetadata::default(),
        )
        .expect("root creation");
    (space, root, root_folder)
}

#[test]
fn logical_separators_normalize_to_one_address() {
    let slash = ResourceAddress::parse("models/ship.glb", AddressCasePolicy::Sensitive)
        .expect("slash address");
    let backslash = ResourceAddress::parse("models\\ship.glb", AddressCasePolicy::Sensitive)
        .expect("backslash address");

    assert_eq!(slash, backslash);
    assert_eq!(slash.to_string(), "models/ship.glb");
}

#[test]
fn address_ordering_and_prefixes_are_normalized_and_segment_aware() {
    let policy = AddressCasePolicy::Insensitive;
    let assets = ResourceAddress::parse("Assets", policy).expect("assets prefix");
    let model = ResourceAddress::parse("assets/models", policy).expect("model prefix");
    let ship = ResourceAddress::parse("ASSETS\\models/ship.glb", policy).expect("ship");
    let sibling = ResourceAddress::parse("assets-old/ship.glb", policy).expect("sibling");

    assert!(ship.has_prefix(&assets));
    assert!(ship.has_prefix(&model));
    assert!(!sibling.has_prefix(&assets));
    assert!(assets < model);
    assert!(model < ship);
}

#[test]
fn case_policy_is_explicit() {
    let sensitive = ResourceAddress::parse("Images/Hull.PNG", AddressCasePolicy::Sensitive)
        .expect("sensitive address");
    let insensitive = ResourceAddress::parse("Images/Hull.PNG", AddressCasePolicy::Insensitive)
        .expect("insensitive address");

    assert_eq!(sensitive.to_string(), "Images/Hull.PNG");
    assert_eq!(insensitive.to_string(), "images/hull.png");
    assert_ne!(sensitive, insensitive);
}

#[test]
fn roots_and_stores_qualify_identical_relative_addresses() {
    let address = ResourceAddress::parse("scene.bin", AddressCasePolicy::Sensitive)
        .expect("resource address");
    let first = ResourceKey::new(
        StoreId::from_u128(1),
        ResourceRootId::from_u128(10),
        address.clone(),
    );
    let other_root = ResourceKey::new(
        StoreId::from_u128(1),
        ResourceRootId::from_u128(11),
        address.clone(),
    );
    let other_store = ResourceKey::new(
        StoreId::from_u128(2),
        ResourceRootId::from_u128(10),
        address,
    );

    assert_ne!(first, other_root);
    assert_ne!(first, other_store);
}

#[test]
fn changing_a_root_display_name_does_not_change_identity() {
    let root_id = ResourceRootId::from_u128(42);
    let mut root = ResourceRootDescriptor::new(root_id, "Imported files");

    root.rename("Project assets");

    assert_eq!(root.id(), root_id);
    assert_eq!(root.display_name(), "Project assets");
}

#[test]
fn ambiguous_or_provider_owned_addresses_are_rejected() {
    let cases = [
        ("", ResourceAddressError::EmptyAddress),
        ("/asset.bin", ResourceAddressError::AbsoluteAddress),
        (
            "assets//asset.bin",
            ResourceAddressError::EmptySegment { index: 1 },
        ),
        (
            "assets/./asset.bin",
            ResourceAddressError::CurrentSegment { index: 1 },
        ),
        (
            "assets/../asset.bin",
            ResourceAddressError::ParentTraversal { index: 1 },
        ),
        (
            "mem:root/asset.bin",
            ResourceAddressError::ProviderQualifier { index: 0 },
        ),
        (
            "C:\\assets\\asset.bin",
            ResourceAddressError::ProviderQualifier { index: 0 },
        ),
        (
            "https://example.invalid/asset.bin",
            ResourceAddressError::ProviderQualifier { index: 0 },
        ),
    ];

    for (value, expected) in cases {
        assert_eq!(
            ResourceAddress::parse(value, AddressCasePolicy::Sensitive),
            Err(expected),
            "unexpected result for {value:?}"
        );
    }
}

#[test]
fn hidden_visibility_is_query_policy_not_address_inference() {
    assert!(VisibilityQuery::VisibleOnly.includes(ResourceVisibility::Visible));
    assert!(!VisibilityQuery::VisibleOnly.includes(ResourceVisibility::Hidden));
    assert!(VisibilityQuery::HiddenOnly.includes(ResourceVisibility::Hidden));
    assert!(VisibilityQuery::All.includes(ResourceVisibility::Visible));
    assert!(VisibilityQuery::All.includes(ResourceVisibility::Hidden));
}

#[test]
fn resource_metadata_is_visible_by_explicit_default() {
    let metadata = crate::ResourceMetadata::default();

    assert_eq!(metadata.visibility, ResourceVisibility::Visible);
}

#[test]
fn resource_visibility_changes_without_replacing_retained_bytes() {
    let (mut space, _, root_folder) = space_with_root();
    let resource_name = name("hidden.svg");
    space
        .insert_resource(
            root_folder,
            resource_name.clone(),
            &b"<svg/>"[..],
            Default::default(),
        )
        .expect("resource");

    let updated = space
        .set_resource_visibility(root_folder, &resource_name, ResourceVisibility::Hidden)
        .expect("visibility update");

    assert_eq!(updated.metadata().visibility, ResourceVisibility::Hidden);
    assert_eq!(updated.bytes().as_ref(), b"<svg/>");
    assert!(space
        .list_resources(root_folder, VisibilityQuery::VisibleOnly)
        .expect("visible resources")
        .is_empty());
    assert_eq!(
        space
            .list_resources(root_folder, VisibilityQuery::HiddenOnly)
            .expect("hidden resources")
            .len(),
        1
    );
}

#[test]
fn roots_have_distinguished_immutable_folder_nodes() {
    let (mut space, root, root_folder) = space_with_root();

    assert_eq!(space.root_folder(root), Some(root_folder));
    assert!(space
        .folder(root_folder)
        .expect("root folder")
        .is_root_folder());
    assert_eq!(
        space.remove_empty_folder(root_folder),
        Err(crate::ResourceSpaceError::RootFolderImmutable {
            folder: root_folder,
        })
    );
}

#[test]
fn empty_and_hidden_folders_are_navigable_by_explicit_query() {
    let (mut space, _, root_folder) = space_with_root();
    let visible = crate::FolderId::from_u128(101);
    let hidden = crate::FolderId::from_u128(102);
    let hidden_metadata = crate::ResourceMetadata {
        visibility: ResourceVisibility::Hidden,
        ..Default::default()
    };
    space
        .create_folder(visible, root_folder, name("alpha"), Default::default())
        .expect("visible folder");
    space
        .create_folder(hidden, root_folder, name("beta"), hidden_metadata)
        .expect("hidden folder");

    let visible_children = space
        .list_children(root_folder, VisibilityQuery::VisibleOnly)
        .expect("visible children");
    let all_children = space
        .list_children(root_folder, VisibilityQuery::All)
        .expect("all children");

    assert_eq!(visible_children.len(), 1);
    assert_eq!(visible_children[0].id(), visible);
    assert_eq!(
        all_children
            .iter()
            .map(crate::FolderEntry::id)
            .collect::<Vec<_>>(),
        [visible, hidden]
    );
    assert!(space.folder(hidden).is_some());
}

#[test]
fn sibling_folder_names_share_one_deterministic_namespace() {
    let (mut space, _, root_folder) = space_with_root();
    space
        .create_folder(
            crate::FolderId::from_u128(101),
            root_folder,
            name("assets"),
            Default::default(),
        )
        .expect("first folder");

    assert_eq!(
        space.create_folder(
            crate::FolderId::from_u128(102),
            root_folder,
            name("assets"),
            Default::default(),
        ),
        Err(crate::ResourceSpaceError::ChildNameConflict {
            parent: root_folder,
            name: name("assets"),
        })
    );
}

#[test]
fn folder_move_rejects_cycles_without_changing_hierarchy() {
    let (mut space, _, root_folder) = space_with_root();
    let parent = crate::FolderId::from_u128(101);
    let child = crate::FolderId::from_u128(102);
    space
        .create_folder(parent, root_folder, name("parent"), Default::default())
        .expect("parent folder");
    space
        .create_folder(child, parent, name("child"), Default::default())
        .expect("child folder");

    assert_eq!(
        space.move_folder(parent, child),
        Err(crate::ResourceSpaceError::FolderMoveCycle {
            folder: parent,
            new_parent: child,
        })
    );
    assert_eq!(
        space.folder(parent).expect("parent").parent(),
        Some(root_folder)
    );
    assert_eq!(space.folder(child).expect("child").parent(), Some(parent));
}

#[test]
fn navigation_fixture_preserves_empty_hidden_folders_and_subtrees_after_a_rejected_move() {
    let (mut space, _, root_folder) = space_with_root();
    let source = crate::FolderId::from_u128(101);
    let archive = crate::FolderId::from_u128(102);
    let empty_hidden = crate::FolderId::from_u128(103);
    let hidden_metadata = crate::ResourceMetadata {
        visibility: ResourceVisibility::Hidden,
        ..Default::default()
    };

    space
        .create_folder(source, root_folder, name("source"), Default::default())
        .expect("source folder");
    space
        .create_folder(archive, source, name("archive"), Default::default())
        .expect("archive folder");
    space
        .create_folder(empty_hidden, root_folder, name("drafts"), hidden_metadata)
        .expect("empty hidden folder");
    space
        .insert_resource(
            archive,
            name("payload.bin"),
            [1_u8, 2, 3],
            Default::default(),
        )
        .expect("nested resource");
    space
        .insert_resource(root_folder, name("archive"), [4_u8], Default::default())
        .expect("conflicting root resource");

    assert_eq!(
        space
            .list_children(root_folder, VisibilityQuery::VisibleOnly)
            .expect("visible root children")
            .iter()
            .map(crate::FolderEntry::id)
            .collect::<Vec<_>>(),
        [source]
    );
    assert!(space.folder(empty_hidden).is_some());

    assert_eq!(
        space.move_folder(archive, root_folder),
        Err(crate::ResourceSpaceError::ChildNameConflict {
            parent: root_folder,
            name: name("archive"),
        })
    );

    assert_eq!(
        space.folder(archive).expect("archive").parent(),
        Some(source)
    );
    let nested = space
        .resource(archive, &name("payload.bin"))
        .expect("nested lookup")
        .expect("nested resource");
    assert_eq!(
        nested.key().address().to_string(),
        "source/archive/payload.bin"
    );
}

#[test]
fn cross_root_moves_are_rejected_before_mutation() {
    let (mut space, _, root_folder) = space_with_root();
    let other_root = ResourceRootId::from_u128(20);
    let other_root_folder = crate::FolderId::from_u128(200);
    let folder = crate::FolderId::from_u128(101);
    space
        .create_root(
            ResourceRootDescriptor::new(other_root, "Imported files"),
            other_root_folder,
            Default::default(),
        )
        .expect("second root");
    space
        .create_folder(folder, root_folder, name("local"), Default::default())
        .expect("folder");

    assert!(matches!(
        space.move_folder(folder, other_root_folder),
        Err(crate::ResourceSpaceError::CrossRootMove { .. })
    ));
    assert_eq!(
        space.folder(folder).expect("folder").parent(),
        Some(root_folder)
    );
}

#[test]
fn empty_root_removal_is_explicit_and_never_deletes_children() {
    let (mut space, root, root_folder) = space_with_root();
    let child = crate::FolderId::from_u128(101);
    space
        .create_folder(child, root_folder, name("assets"), Default::default())
        .expect("child folder");

    assert_eq!(
        space.remove_empty_root(root),
        Err(crate::ResourceSpaceError::RootNotEmpty { root })
    );
    assert!(space.folder(root_folder).is_some());
    assert!(space.folder(child).is_some());

    space.remove_empty_folder(child).expect("remove child");
    let descriptor = space.remove_empty_root(root).expect("remove empty root");
    assert_eq!(descriptor.id(), root);
    assert!(space.root(root).is_none());
    assert!(space.folder(root_folder).is_none());
}

#[test]
fn folder_names_must_match_the_resource_space_case_policy() {
    let root = ResourceRootId::from_u128(10);
    let root_folder = crate::FolderId::from_u128(100);
    let mut space =
        crate::InMemoryResourceSpace::new(StoreId::from_u128(1), AddressCasePolicy::Insensitive);
    space
        .create_root(
            ResourceRootDescriptor::new(root, "Project assets"),
            root_folder,
            Default::default(),
        )
        .expect("root creation");

    let mismatched = name("Assets");
    assert_eq!(
        space.create_folder(
            crate::FolderId::from_u128(101),
            root_folder,
            mismatched.clone(),
            Default::default(),
        ),
        Err(crate::ResourceSpaceError::NamePolicyMismatch { name: mismatched })
    );

    let canonical = space.resource_name("Assets").expect("canonical name");
    assert_eq!(canonical.as_str(), "assets");
    space
        .create_folder(
            crate::FolderId::from_u128(101),
            root_folder,
            canonical,
            Default::default(),
        )
        .expect("canonical folder name");
}

#[test]
fn resources_retain_shared_immutable_bytes_and_qualified_addresses() {
    let (mut space, root, root_folder) = space_with_root();
    let assets = crate::FolderId::from_u128(101);
    space
        .create_folder(assets, root_folder, name("assets"), Default::default())
        .expect("assets folder");

    let bytes = std::sync::Arc::<[u8]>::from([1_u8, 2, 3]);
    let inserted = space
        .insert_resource(assets, name("ship.glb"), bytes.clone(), Default::default())
        .expect("insert resource");
    let fetched = space
        .resource(assets, &name("ship.glb"))
        .expect("resource lookup")
        .expect("stored resource");

    assert_eq!(inserted.key().store(), StoreId::from_u128(1));
    assert_eq!(inserted.key().root(), root);
    assert_eq!(inserted.key().address().to_string(), "assets/ship.glb");
    assert!(std::sync::Arc::ptr_eq(inserted.bytes(), &bytes));
    assert!(std::sync::Arc::ptr_eq(inserted.bytes(), fetched.bytes()));
}

#[test]
fn resource_and_folder_names_share_a_conflict_namespace() {
    let (mut space, _, root_folder) = space_with_root();
    space
        .insert_resource(root_folder, name("scene"), [1_u8], Default::default())
        .expect("resource");

    assert_eq!(
        space.create_folder(
            crate::FolderId::from_u128(101),
            root_folder,
            name("scene"),
            Default::default(),
        ),
        Err(crate::ResourceSpaceError::ChildNameConflict {
            parent: root_folder,
            name: name("scene"),
        })
    );
}

#[test]
fn resource_replacement_is_atomic_and_empty_folder_removal_rejects_resources() {
    let (mut space, _, root_folder) = space_with_root();
    let folder = crate::FolderId::from_u128(101);
    space
        .create_folder(folder, root_folder, name("assets"), Default::default())
        .expect("folder");
    space
        .insert_resource(folder, name("ship.glb"), [1_u8], Default::default())
        .expect("resource");

    assert_eq!(
        space.remove_empty_folder(folder),
        Err(crate::ResourceSpaceError::FolderNotEmpty { folder })
    );
    let replacement = space
        .replace_resource(folder, &name("ship.glb"), [2_u8, 3], Default::default())
        .expect("replacement");
    assert_eq!(replacement.bytes().as_ref(), &[2, 3]);
    let removed = space
        .remove_resource(folder, &name("ship.glb"))
        .expect("resource removal");
    assert_eq!(removed.bytes().as_ref(), &[2, 3]);
    space.remove_empty_folder(folder).expect("empty folder");
}

#[test]
fn document_bundle_preserves_explicit_navigation_and_replacement_intent() {
    let (mut space, _, root_folder) = space_with_root();
    let common = crate::FolderId::from_u128(101);
    let assets = crate::FolderId::from_u128(102);
    let drafts = crate::FolderId::from_u128(103);
    space
        .create_folder(common, root_folder, name("common"), Default::default())
        .expect("common folder");
    space
        .create_folder(assets, root_folder, name("assets"), Default::default())
        .expect("assets folder");
    // Empty folders are navigable state, not a prefix inferred from stored bytes.
    space
        .create_folder(drafts, root_folder, name("drafts"), Default::default())
        .expect("drafts folder");

    space
        .insert_resource(
            root_folder,
            name("data.xml"),
            b"<data/>".as_slice(),
            Default::default(),
        )
        .expect("document source");
    space
        .insert_resource(
            root_folder,
            name("transform.xsl"),
            b"<xsl:stylesheet/>".as_slice(),
            Default::default(),
        )
        .expect("transform source");
    space
        .insert_resource(
            common,
            name("utilities.xsl"),
            b"<xsl:stylesheet/>".as_slice(),
            Default::default(),
        )
        .expect("shared utility");
    space
        .insert_resource(
            assets,
            name("logo.png"),
            [0x89_u8, b'P', b'N', b'G'],
            Default::default(),
        )
        .expect("image asset");

    assert_eq!(
        space
            .list_children(root_folder, VisibilityQuery::VisibleOnly)
            .expect("root folders")
            .iter()
            .map(|entry| entry.name().expect("named child").as_str())
            .collect::<Vec<_>>(),
        ["assets", "common", "drafts"]
    );
    assert!(space.folder(drafts).is_some());
    assert_eq!(
        space
            .list_resources(root_folder, VisibilityQuery::VisibleOnly)
            .expect("root documents")
            .iter()
            .map(|entry| entry.name().as_str())
            .collect::<Vec<_>>(),
        ["data.xml", "transform.xsl"]
    );
    assert_eq!(
        space
            .resource(common, &name("utilities.xsl"))
            .expect("same-folder utility")
            .expect("utility exists")
            .bytes()
            .as_ref(),
        b"<xsl:stylesheet/>"
    );

    assert_eq!(
        space.insert_resource(
            root_folder,
            name("data.xml"),
            b"replacement".as_slice(),
            Default::default()
        ),
        Err(crate::ResourceSpaceError::ChildNameConflict {
            parent: root_folder,
            name: name("data.xml"),
        })
    );
    let replacement = space
        .replace_resource(
            root_folder,
            &name("data.xml"),
            b"<data version=\"2\"/>".as_slice(),
            Default::default(),
        )
        .expect("explicit replacement");
    assert_eq!(replacement.bytes().as_ref(), b"<data version=\"2\"/>");
}

#[test]
fn a_root_with_a_direct_resource_cannot_be_removed() {
    let (mut space, root, root_folder) = space_with_root();
    space
        .insert_resource(
            root_folder,
            name("manifest.toml"),
            [1_u8],
            Default::default(),
        )
        .expect("resource");

    assert_eq!(
        space.remove_empty_root(root),
        Err(crate::ResourceSpaceError::RootNotEmpty { root })
    );
}

#[test]
fn folder_mutation_requalifies_descendant_resource_addresses() {
    let (mut space, _, root_folder) = space_with_root();
    let assets = crate::FolderId::from_u128(101);
    let archive = crate::FolderId::from_u128(102);
    space
        .create_folder(assets, root_folder, name("assets"), Default::default())
        .expect("assets folder");
    space
        .create_folder(archive, root_folder, name("archive"), Default::default())
        .expect("archive folder");
    space
        .insert_resource(assets, name("ship.glb"), [1_u8], Default::default())
        .expect("resource");

    space.rename_folder(assets, name("models")).expect("rename");
    let after_rename = space
        .resource(assets, &name("ship.glb"))
        .expect("lookup")
        .expect("resource");
    assert_eq!(after_rename.key().address().to_string(), "models/ship.glb");

    space.move_folder(assets, archive).expect("move");
    let after_move = space
        .resource(assets, &name("ship.glb"))
        .expect("lookup")
        .expect("resource");
    assert_eq!(
        after_move.key().address().to_string(),
        "archive/models/ship.glb"
    );
}

#[test]
fn resource_enumeration_copy_and_move_are_deterministic() {
    let (mut space, _, root_folder) = space_with_root();
    let source = crate::FolderId::from_u128(101);
    let destination = crate::FolderId::from_u128(102);
    space
        .create_folder(source, root_folder, name("source"), Default::default())
        .expect("source folder");
    space
        .create_folder(
            destination,
            root_folder,
            name("destination"),
            Default::default(),
        )
        .expect("destination folder");
    let bytes: std::sync::Arc<[u8]> = std::sync::Arc::from([1_u8, 2, 3]);
    space
        .insert_resource(source, name("zeta.bin"), bytes.clone(), Default::default())
        .expect("zeta");
    space
        .insert_resource(source, name("alpha.bin"), [4_u8], Default::default())
        .expect("alpha");

    let listed = space
        .list_resources(source, VisibilityQuery::VisibleOnly)
        .expect("resource list");
    assert_eq!(
        listed
            .iter()
            .map(|entry| entry.name().as_str())
            .collect::<Vec<_>>(),
        vec!["alpha.bin", "zeta.bin"]
    );
    assert!(space
        .contains_resource(source, &name("zeta.bin"))
        .expect("contains"));

    let copied = space
        .copy_resource(source, &name("zeta.bin"), destination, name("copy.bin"))
        .expect("copy");
    assert!(std::sync::Arc::ptr_eq(copied.bytes(), &bytes));
    assert_eq!(copied.key().address().to_string(), "destination/copy.bin");

    let moved = space
        .move_resource(source, &name("alpha.bin"), destination, name("moved.bin"))
        .expect("move");
    assert_eq!(moved.key().address().to_string(), "destination/moved.bin");
    assert!(!space
        .contains_resource(source, &name("alpha.bin"))
        .expect("source removal"));
}

#[test]
fn recursive_search_is_literal_bounded_and_distinct_from_direct_navigation() {
    let (mut space, _, root_folder) = space_with_root();
    let images = crate::FolderId::from_u128(101);
    let nested = crate::FolderId::from_u128(102);
    space
        .create_folder(images, root_folder, name("images"), Default::default())
        .expect("images folder");
    space
        .create_folder(nested, images, name("generated"), Default::default())
        .expect("nested folder");
    space
        .insert_resource(
            images,
            name("icon.png"),
            [1_u8],
            crate::ResourceMetadata {
                media_type: Some("image/png".to_owned()),
                ..Default::default()
            },
        )
        .expect("direct PNG");
    space
        .insert_resource(
            nested,
            name("icon-dark.png"),
            [2_u8],
            crate::ResourceMetadata {
                media_type: Some("image/png".to_owned()),
                ..Default::default()
            },
        )
        .expect("nested PNG");
    space
        .insert_resource(
            nested,
            name("draft.svg"),
            [3_u8],
            crate::ResourceMetadata {
                visibility: ResourceVisibility::Hidden,
                media_type: Some("image/svg+xml".to_owned()),
                ..Default::default()
            },
        )
        .expect("hidden SVG");

    assert_eq!(
        space
            .list_resources(images, VisibilityQuery::VisibleOnly)
            .expect("direct list")
            .len(),
        1
    );
    let found = space
        .search_resources(
            images,
            &crate::ResourceSearchQuery::new(8)
                .with_name_prefix("icon")
                .with_name_suffix(".png")
                .with_media_type("image/png"),
        )
        .expect("recursive literal search");
    assert_eq!(
        found
            .iter()
            .map(|entry| entry.key().address().to_string())
            .collect::<Vec<_>>(),
        ["images/generated/icon-dark.png", "images/icon.png"]
    );

    let hidden = space
        .search_resources(
            images,
            &crate::ResourceSearchQuery::new(1)
                .visibility(VisibilityQuery::HiddenOnly)
                .with_name_suffix(".svg"),
        )
        .expect("hidden search");
    assert_eq!(
        hidden[0].key().address().to_string(),
        "images/generated/draft.svg"
    );
    assert_eq!(
        space.search_resources(images, &crate::ResourceSearchQuery::new(0)),
        Err(crate::ResourceSpaceError::SearchResultLimitZero)
    );
}

#[test]
fn hidden_resources_remain_directly_addressable_without_leaking_into_visible_lists() {
    let (mut space, _, root_folder) = space_with_root();
    let hidden_metadata = crate::ResourceMetadata {
        visibility: ResourceVisibility::Hidden,
        ..Default::default()
    };
    let resource_name = name("draft.bin");
    space
        .insert_resource(
            root_folder,
            resource_name.clone(),
            [7_u8, 8, 9],
            hidden_metadata,
        )
        .expect("hidden resource");

    let direct = space
        .resource(root_folder, &resource_name)
        .expect("direct lookup")
        .expect("hidden resource exists");
    assert_eq!(direct.bytes().as_ref(), &[7, 8, 9]);
    assert_eq!(direct.metadata().visibility, ResourceVisibility::Hidden);

    assert!(space
        .list_resources(root_folder, VisibilityQuery::VisibleOnly)
        .expect("visible resources")
        .is_empty());
    assert_eq!(
        space
            .list_resources(root_folder, VisibilityQuery::HiddenOnly)
            .expect("hidden resources")
            .iter()
            .map(|entry| entry.name().as_str())
            .collect::<Vec<_>>(),
        ["draft.bin"]
    );
}

#[test]
fn content_fingerprints_are_named_diagnostics_not_resource_identity() {
    let (mut space, _, root_folder) = space_with_root();
    let left = space
        .insert_resource(
            root_folder,
            name("left.bin"),
            [1_u8, 2, 3],
            Default::default(),
        )
        .expect("left resource");
    let right = space
        .insert_resource(
            root_folder,
            name("right.bin"),
            [1_u8, 2, 3],
            Default::default(),
        )
        .expect("right resource");
    let different = space
        .insert_resource(
            root_folder,
            name("different.bin"),
            [1_u8, 2, 4],
            Default::default(),
        )
        .expect("different resource");

    let fingerprint = left.content_fingerprint();
    assert_eq!(
        fingerprint.algorithm(),
        crate::ContentFingerprintAlgorithm::Blake3
    );
    assert!(fingerprint.matches_bytes(left.bytes()));
    assert_eq!(fingerprint, right.content_fingerprint());
    assert_ne!(fingerprint, different.content_fingerprint());
    assert_ne!(left.key(), right.key());
    assert!(left.has_same_content_as(&right));
    assert!(!left.has_same_content_as(&different));
}

#[test]
fn mutation_observation_is_opt_in_bounded_and_locally_ordered() {
    let (mut space, _, root_folder) = space_with_root();
    assert!(!space.mutation_observations_enabled());
    assert_eq!(space.mutation_observation_capacity(), None);
    assert_eq!(space.mutation_observations().len(), 0);
    assert_eq!(
        space.enable_mutation_observations(0),
        Err(crate::ResourceSpaceError::MutationObservationCapacityZero)
    );

    space
        .enable_mutation_observations(3)
        .expect("enable bounded observation");
    assert_eq!(space.mutation_observation_capacity(), Some(3));
    let folder = crate::FolderId::from_u128(101);
    space
        .create_folder(folder, root_folder, name("assets"), Default::default())
        .expect("create folder");
    space
        .insert_resource(folder, name("ship.bin"), [1_u8, 2], Default::default())
        .expect("insert resource");

    // Failed mutations do not produce successful mutation outcomes.
    assert!(space
        .insert_resource(folder, name("ship.bin"), [3_u8], Default::default())
        .is_err());
    space
        .replace_resource(folder, &name("ship.bin"), [3_u8, 4, 5], Default::default())
        .expect("replace resource");
    space
        .remove_resource(folder, &name("ship.bin"))
        .expect("remove resource");

    let observations = space.mutation_observations().cloned().collect::<Vec<_>>();
    assert_eq!(
        observations
            .iter()
            .map(crate::ResourceMutationObservation::sequence)
            .collect::<Vec<_>>(),
        [2, 3, 4]
    );
    assert!(matches!(
        observations[0].outcome(),
        crate::ResourceMutationOutcome::ResourceInserted { byte_len: 2, .. }
    ));
    assert!(matches!(
        observations[1].outcome(),
        crate::ResourceMutationOutcome::ResourceReplaced {
            previous_byte_len: 2,
            byte_len: 3,
            ..
        }
    ));
    assert!(matches!(
        observations[2].outcome(),
        crate::ResourceMutationOutcome::ResourceRemoved { byte_len: 3, .. }
    ));

    assert_eq!(space.drain_mutation_observations(), observations);
    assert_eq!(space.mutation_observations().len(), 0);
    space.disable_mutation_observations();
    assert!(!space.mutation_observations_enabled());
    assert_eq!(space.mutation_observation_capacity(), None);
}

#[test]
fn no_op_resource_move_does_not_emit_a_mutation() {
    let (mut space, _, root_folder) = space_with_root();
    space
        .insert_resource(root_folder, name("stable.bin"), [1_u8], Default::default())
        .expect("resource");
    space
        .enable_mutation_observations(2)
        .expect("enable observation");

    space
        .move_resource(
            root_folder,
            &name("stable.bin"),
            root_folder,
            name("stable.bin"),
        )
        .expect("no-op move");

    assert_eq!(space.mutation_observations().len(), 0);
}

#[test]
fn summary_reports_current_retained_hierarchy_without_lifetime_counters() {
    let (mut space, _, root_folder) = space_with_root();
    let assets = crate::FolderId::from_u128(101);
    space
        .create_folder(assets, root_folder, name("assets"), Default::default())
        .expect("assets folder");
    space
        .insert_resource(
            root_folder,
            name("manifest.json"),
            [1_u8, 2],
            Default::default(),
        )
        .expect("manifest");
    space
        .insert_resource(assets, name("ship.glb"), [3_u8, 4, 5], Default::default())
        .expect("ship");

    assert_eq!(
        space.summary(),
        crate::ResourceSpaceSummary::new(1, 2, 2, 5)
    );

    space
        .remove_resource(assets, &name("ship.glb"))
        .expect("remove ship");
    assert_eq!(space.summary().resources(), 1);
    assert_eq!(space.summary().retained_bytes(), 2);
}

#[test]
fn byte_and_entry_limits_reject_mutations_without_partial_state() {
    let root = ResourceRootId::from_u128(10);
    let root_folder = crate::FolderId::from_u128(100);
    let mut space = crate::InMemoryResourceSpace::with_limits(
        StoreId::from_u128(1),
        AddressCasePolicy::Sensitive,
        crate::ResourceSpaceLimits {
            max_entries: Some(1),
            max_total_bytes: Some(3),
            max_bytes_per_entry: Some(2),
        },
    );
    space
        .create_root(
            ResourceRootDescriptor::new(root, "Limited"),
            root_folder,
            crate::ResourceMetadata::default(),
        )
        .expect("root");

    assert_eq!(
        space.insert_resource(
            root_folder,
            name("large.bin"),
            [1_u8, 2, 3],
            Default::default()
        ),
        Err(crate::ResourceSpaceError::EntryByteLimitExceeded {
            limit: 2,
            attempted: 3,
        })
    );
    space
        .insert_resource(
            root_folder,
            name("first.bin"),
            [1_u8, 2],
            Default::default(),
        )
        .expect("first entry");
    assert_eq!(
        space.insert_resource(root_folder, name("second.bin"), [3_u8], Default::default()),
        Err(crate::ResourceSpaceError::EntryLimitExceeded {
            limit: 1,
            attempted: 2,
        })
    );
    assert_eq!(
        space
            .resource(root_folder, &name("first.bin"))
            .expect("lookup")
            .expect("entry")
            .bytes()
            .as_ref(),
        &[1, 2]
    );
}

#[test]
fn total_byte_limit_rejects_a_new_resource_without_removing_existing_content() {
    let root = ResourceRootId::from_u128(10);
    let root_folder = crate::FolderId::from_u128(100);
    let mut space = crate::InMemoryResourceSpace::with_limits(
        StoreId::from_u128(1),
        AddressCasePolicy::Sensitive,
        crate::ResourceSpaceLimits {
            max_entries: Some(3),
            max_total_bytes: Some(3),
            max_bytes_per_entry: Some(2),
        },
    );
    space
        .create_root(
            ResourceRootDescriptor::new(root, "Limited"),
            root_folder,
            crate::ResourceMetadata::default(),
        )
        .expect("root");
    space
        .insert_resource(
            root_folder,
            name("first.bin"),
            [1_u8, 2],
            Default::default(),
        )
        .expect("first entry");
    space
        .insert_resource(root_folder, name("second.bin"), [3_u8], Default::default())
        .expect("second entry");

    assert_eq!(
        space.insert_resource(root_folder, name("third.bin"), [4_u8], Default::default()),
        Err(crate::ResourceSpaceError::TotalByteLimitExceeded {
            limit: 3,
            attempted: 4,
        })
    );
    assert!(space
        .contains_resource(root_folder, &name("first.bin"))
        .expect("first entry remains"));
    assert!(!space
        .contains_resource(root_folder, &name("third.bin"))
        .expect("failed entry absent"));
}

#[test]
fn registry_uses_stable_store_identity_not_display_name_or_content() {
    let first = StoreId::from_u128(1);
    let second = StoreId::from_u128(2);
    let mut registry = crate::InMemoryResourceSpaceRegistry::new();
    registry
        .create_new(
            crate::ResourceStoreDescriptor::new(first, "Project"),
            AddressCasePolicy::Sensitive,
        )
        .expect("first store");
    registry
        .create_new(
            crate::ResourceStoreDescriptor::new(second, "Project"),
            AddressCasePolicy::Sensitive,
        )
        .expect("same display name with distinct identity");

    assert_eq!(
        registry.create_new(
            crate::ResourceStoreDescriptor::new(first, "Different content"),
            AddressCasePolicy::Sensitive,
        ),
        Err(crate::ResourceStoreRegistryError::StoreAlreadyExists { store: first })
    );
    assert_eq!(
        registry
            .create_or_open(
                crate::ResourceStoreDescriptor::new(first, "Ignored display name"),
                AddressCasePolicy::Sensitive,
            )
            .expect("open existing"),
        crate::StoreOpenOutcome::OpenedExisting { store: first }
    );
    assert_eq!(
        registry
            .descriptor(first)
            .expect("descriptor")
            .display_name(),
        "Project"
    );
}

#[test]
fn store_provenance_is_advisory_but_available_for_conflict_diagnostics() {
    let first = StoreId::from_u128(41);
    let second = StoreId::from_u128(42);
    let fixture = crate::ResourceStoreDescriptor::new(first, "Project").with_provenance(
        crate::ResourceStoreProvenance::with_label(
            crate::ResourceStoreOrigin::Fixture,
            "w3c-svg-selection-v1",
        ),
    );
    let imported = crate::ResourceStoreDescriptor::new(second, "Project").with_provenance(
        crate::ResourceStoreProvenance::new(crate::ResourceStoreOrigin::Imported),
    );
    let mut registry = crate::InMemoryResourceSpaceRegistry::new();

    registry
        .create_new(fixture, AddressCasePolicy::Sensitive)
        .expect("fixture store");
    registry
        .create_new(imported, AddressCasePolicy::Sensitive)
        .expect("same display name remains valid");

    assert_eq!(
        registry.create_new(
            crate::ResourceStoreDescriptor::new(first, "Conflicting name").with_provenance(
                crate::ResourceStoreProvenance::new(crate::ResourceStoreOrigin::Generated),
            ),
            AddressCasePolicy::Sensitive,
        ),
        Err(crate::ResourceStoreRegistryError::StoreAlreadyExists { store: first })
    );

    let retained = registry.descriptor(first).expect("retained descriptor");
    assert_eq!(retained.display_name(), "Project");
    assert_eq!(
        retained.provenance().origin(),
        crate::ResourceStoreOrigin::Fixture
    );
    assert_eq!(retained.provenance().label(), Some("w3c-svg-selection-v1"));
}

#[test]
fn synchronized_registry_creation_preserves_one_store_per_stable_identity() {
    let store = StoreId::from_u128(51);
    let registry = std::sync::Arc::new(std::sync::Mutex::new(
        crate::InMemoryResourceSpaceRegistry::new(),
    ));
    let callers = (0..2)
        .map(|index| {
            let registry = std::sync::Arc::clone(&registry);
            std::thread::spawn(move || {
                registry.lock().expect("registry lock").create_new(
                    crate::ResourceStoreDescriptor::new(store, format!("Caller {index}")),
                    AddressCasePolicy::Sensitive,
                )
            })
        })
        .collect::<Vec<_>>();

    let outcomes = callers
        .into_iter()
        .map(|caller| caller.join().expect("caller thread"))
        .collect::<Vec<_>>();
    assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
    assert_eq!(
        outcomes
            .iter()
            .filter(|outcome| {
                matches!(
                    outcome,
                    Err(crate::ResourceStoreRegistryError::StoreAlreadyExists { store: duplicate })
                        if *duplicate == store
                )
            })
            .count(),
        1
    );

    let registry = registry.lock().expect("registry lock");
    assert_eq!(
        registry.descriptor(store).expect("one retained store").id(),
        store
    );
}

#[test]
fn registry_rejects_case_policy_changes_for_an_existing_store() {
    let store = StoreId::from_u128(1);
    let mut registry = crate::InMemoryResourceSpaceRegistry::new();
    registry
        .create_new(
            crate::ResourceStoreDescriptor::new(store, "Project"),
            AddressCasePolicy::Sensitive,
        )
        .expect("store");

    assert_eq!(
        registry.create_or_open(
            crate::ResourceStoreDescriptor::new(store, "Project"),
            AddressCasePolicy::Insensitive,
        ),
        Err(crate::ResourceStoreRegistryError::StorePolicyMismatch {
            store,
            existing: AddressCasePolicy::Sensitive,
            requested: AddressCasePolicy::Insensitive,
        })
    );
}

#[test]
fn dot_names_remain_addressable() {
    let hidden_by_name =
        ResourceAddress::parse(".config/settings.json", AddressCasePolicy::Sensitive)
            .expect("dot names are valid logical names");

    assert_eq!(hidden_by_name.to_string(), ".config/settings.json");
}

#[test]
fn deterministic_mutation_sequences_preserve_public_summary_and_navigation() {
    // This is a deliberately dependency-free first property-style matrix. It
    // exercises every three-step combination of the core resource mutations
    // and verifies the public summary against direct, navigable state.
    for first in 0_u8..5 {
        for second in 0_u8..5 {
            for third in 0_u8..5 {
                let (mut space, _, root_folder) = space_with_root();
                let source = crate::FolderId::from_u128(101);
                let destination = crate::FolderId::from_u128(102);
                space
                    .create_folder(source, root_folder, name("source"), Default::default())
                    .expect("source folder");
                space
                    .create_folder(
                        destination,
                        root_folder,
                        name("destination"),
                        Default::default(),
                    )
                    .expect("destination folder");
                space
                    .insert_resource(source, name("alpha.bin"), [1_u8, 2], Default::default())
                    .expect("initial resource");

                for operation in [first, second, third] {
                    match operation {
                        0 => {
                            let _ = space.replace_resource(
                                source,
                                &name("alpha.bin"),
                                [3_u8, 4, 5],
                                Default::default(),
                            );
                        }
                        1 => {
                            let _ = space.copy_resource(
                                source,
                                &name("alpha.bin"),
                                destination,
                                name("copy.bin"),
                            );
                        }
                        2 => {
                            let _ = space.move_resource(
                                source,
                                &name("alpha.bin"),
                                destination,
                                name("moved.bin"),
                            );
                        }
                        3 => {
                            let _ = space.insert_resource(
                                source,
                                name("alpha.bin"),
                                [6_u8],
                                Default::default(),
                            );
                        }
                        4 => {
                            let _ = space.remove_resource(source, &name("alpha.bin"));
                        }
                        _ => unreachable!("the mutation matrix has five operations"),
                    }
                }

                let resources = [root_folder, source, destination]
                    .into_iter()
                    .flat_map(|folder| {
                        space
                            .list_resources(folder, VisibilityQuery::All)
                            .expect("known folder")
                    })
                    .collect::<Vec<_>>();
                let retained_bytes = resources.iter().map(ResourceEntry::byte_len).sum::<usize>();
                let summary = space.summary();

                assert_eq!(summary.roots(), 1);
                assert_eq!(summary.folders(), 3);
                assert_eq!(summary.resources(), resources.len());
                assert_eq!(summary.retained_bytes(), retained_bytes);
            }
        }
    }
}
