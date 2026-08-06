//! Versioned JSON-lines bridge for the .NET Resource Space workbench.
//!
//! It exposes only Tokimu Resource Space semantics. A later Tosumu provider
//! must preserve this envelope instead of exposing pages, WAL records, or CLI
//! inspection DTOs to the desktop host.

mod tosumu_provider;

use archive_provider::{
    ArchiveFormat, ArchiveReadLimits, SevenZipArchiveProvider, TarArchiveProvider,
    ZipArchiveProvider,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use compression_provider::{
    CompressionCodec, CompressionGoal, DecodeLimits, FlateCompressionProvider,
};
use resource_space::{
    AddressCasePolicy, FolderEntry, FolderId, InMemoryResourceSpaceRegistry, ResourceEntry,
    ResourceMetadata, ResourceRootDescriptor, ResourceRootId, ResourceStoreDescriptor,
    ResourceVisibility, StoreId, VisibilityQuery,
};
use resource_space_archive::{
    import_archive_subtree, inspect_archive_resource as inspect_resource_archive,
    ArchiveSubtreeImportObservation, ImportArchiveSubtreeRequest, InspectArchiveResourceRequest,
    ResourceArchiveInspection,
};
use resource_space_compression::{
    transform_resource, ResourceCollisionPolicy, ResourceCompressionObservation,
    ResourceCompressionRequest, ResourceCompressionTransform, ResourceTransformMutation,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use tosumu_provider::{PersistedResourceSpace, TosumuProvider};

pub const BRIDGE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
pub struct BridgeRequest {
    pub schema: u32,
    #[serde(default)]
    pub request_id: String,
    pub command: String,
    #[serde(default)]
    pub arguments: Value,
}

#[derive(Debug, Serialize)]
pub struct BridgeResponse {
    pub schema: u32,
    pub request_id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<BridgeError>,
}

#[derive(Debug, Serialize)]
pub struct BridgeError {
    pub kind: &'static str,
    pub message: String,
}

#[derive(Default)]
pub struct ResourceBridge {
    registry: InMemoryResourceSpaceRegistry,
    session: Option<Session>,
    provider: ProviderState,
}

#[derive(Debug, Clone, Copy)]
struct Session {
    store: StoreId,
    root: ResourceRootId,
    root_folder: FolderId,
    next_folder: u128,
}

#[derive(Default)]
enum ProviderState {
    #[default]
    InMemory,
    Tosumu(TosumuProvider),
}

impl ResourceBridge {
    pub fn invalid_request(message: String) -> BridgeResponse {
        BridgeResponse {
            schema: BRIDGE_SCHEMA_VERSION,
            request_id: String::new(),
            ok: false,
            result: None,
            error: Some(BridgeError {
                kind: "protocol.invalid_request",
                message,
            }),
        }
    }

    pub fn execute(&mut self, request: BridgeRequest) -> BridgeResponse {
        let request_id = request.request_id.clone();
        let outcome = if request.schema != BRIDGE_SCHEMA_VERSION {
            Err(error(
                "protocol.unsupported_schema",
                format!(
                    "bridge supports schema {BRIDGE_SCHEMA_VERSION}, not {}",
                    request.schema
                ),
            ))
        } else {
            self.execute_command(&request.command, &request.arguments)
        };
        let outcome = outcome.and_then(|result| {
            if is_mutating_command(&request.command) {
                self.persist_active_state()?;
            }
            Ok(result)
        });
        match outcome {
            Ok(result) => response(request_id, true, Some(result), None),
            Err(error) => response(request_id, false, None, Some(error)),
        }
    }

    fn execute_command(&mut self, command: &str, arguments: &Value) -> Result<Value, BridgeError> {
        match command {
            "session.create_or_open" => self.create_or_open(arguments),
            "folder.list" => self.list_folders(arguments),
            "folder.create" => self.create_folder(arguments),
            "resource.put" => self.put_resource(arguments),
            "resource.get" => self.get_resource(arguments),
            "resource.list" => self.list_resources(arguments),
            "resource.set_visibility" => self.set_resource_visibility(arguments),
            "resource.move" => self.move_resource(arguments),
            "resource.transform_compression" => self.transform_resource_compression(arguments),
            "resource.inspect_archive" => self.inspect_archive_resource(arguments),
            "resource.import_archive_subtree" => self.import_archive_subtree(arguments),
            "observation.summary" => self.summary(),
            "provider.inspect" => self.provider_inspect(),
            _ => Err(error(
                "command.unknown",
                format!("unknown Resource Space command {command:?}"),
            )),
        }
    }

    fn create_or_open(&mut self, arguments: &Value) -> Result<Value, BridgeError> {
        let store = StoreId::from_u128(number(arguments, "store_id", 1)?);
        let root = ResourceRootId::from_u128(number(arguments, "root_id", 1)?);
        let root_folder = FolderId::from_u128(number(arguments, "root_folder_id", 1)?);
        let display_name = optional_string(arguments, "display_name")
            .unwrap_or_else(|| "Tokimu Resource Workbench".to_owned());
        let root_name = optional_string(arguments, "root_display_name")
            .unwrap_or_else(|| "Resources".to_owned());
        let policy = case_policy(optional_string(arguments, "case_policy").as_deref())?;
        let provider_name =
            optional_string(arguments, "provider").unwrap_or_else(|| "in_memory".to_owned());
        if provider_name == "tosumu" {
            return self.create_or_open_tosumu(
                arguments,
                store,
                root,
                root_folder,
                display_name,
                root_name,
                policy,
            );
        }
        if provider_name != "in_memory" {
            return Err(error(
                "command.invalid_argument",
                "provider must be in_memory or tosumu",
            ));
        }
        self.provider = ProviderState::InMemory;
        let outcome = self
            .registry
            .create_or_open(ResourceStoreDescriptor::new(store, display_name), policy)
            .map_err(resource_error)?;
        let created = matches!(outcome, resource_space::StoreOpenOutcome::Created { .. });
        if created {
            self.registry
                .space_mut(store)
                .map_err(resource_error)?
                .create_root(
                    ResourceRootDescriptor::new(root, root_name),
                    root_folder,
                    ResourceMetadata::default(),
                )
                .map_err(resource_error)?;
        }
        self.session = Some(Session {
            store,
            root,
            root_folder,
            next_folder: root_folder.as_u128().saturating_add(1),
        });
        Ok(json!({
            "mode": "in_memory", "outcome": if created { "created" } else { "opened_existing" },
            "store_id": store.as_u128().to_string(), "root_id": root.as_u128().to_string(),
            "root_folder_id": root_folder.as_u128().to_string(), "case_policy": case_policy_name(policy),
        }))
    }

    fn create_or_open_tosumu(
        &mut self,
        arguments: &Value,
        store: StoreId,
        root: ResourceRootId,
        root_folder: FolderId,
        display_name: String,
        root_name: String,
        policy: AddressCasePolicy,
    ) -> Result<Value, BridgeError> {
        let path = required_string(arguments, "store_path")?;
        let (provider, snapshot) = TosumuProvider::open(Path::new(&path))
            .map_err(|message| error("provider.tosumu.open", message))?;
        self.registry = InMemoryResourceSpaceRegistry::default();
        let (outcome, session) = match snapshot {
            Some(snapshot) => {
                if snapshot.store_id() != store
                    || snapshot.root_id() != root
                    || snapshot
                        .root_folder_id()
                        .map_err(|message| error("provider.tosumu.snapshot", message))?
                        != root_folder
                    || snapshot.case_policy() != policy
                {
                    return Err(error(
                        "provider.tosumu.identity_conflict",
                        "durable store identity, root identity, or case policy does not match the requested session",
                    ));
                }
                let restored = snapshot
                    .restore(&mut self.registry)
                    .map_err(|message| error("provider.tosumu.snapshot", message))?;
                (
                    "opened_existing",
                    Session {
                        store: restored.store,
                        root: restored.root,
                        root_folder: restored.root_folder,
                        next_folder: restored.next_folder,
                    },
                )
            }
            None => {
                self.registry
                    .create_new(ResourceStoreDescriptor::new(store, display_name), policy)
                    .map_err(resource_error)?;
                self.registry
                    .space_mut(store)
                    .map_err(resource_error)?
                    .create_root(
                        ResourceRootDescriptor::new(root, root_name),
                        root_folder,
                        ResourceMetadata::default(),
                    )
                    .map_err(resource_error)?;
                (
                    "created",
                    Session {
                        store,
                        root,
                        root_folder,
                        next_folder: root_folder.as_u128().saturating_add(1),
                    },
                )
            }
        };
        self.session = Some(session);
        self.provider = ProviderState::Tosumu(provider);
        self.persist_active_state()?;
        Ok(json!({
            "mode": "tosumu", "outcome": outcome,
            "store_id": session.store.as_u128().to_string(),
            "root_id": session.root.as_u128().to_string(),
            "root_folder_id": session.root_folder.as_u128().to_string(),
            "case_policy": case_policy_name(policy),
        }))
    }

    fn list_folders(&self, arguments: &Value) -> Result<Value, BridgeError> {
        let session = self.session()?;
        let parent = FolderId::from_u128(number(
            arguments,
            "parent_folder_id",
            session.root_folder.as_u128(),
        )?);
        let visibility = visibility_query(optional_string(arguments, "visibility").as_deref())?;
        let folders = self
            .registry
            .space(session.store)
            .map_err(resource_error)?
            .list_folders(parent, visibility)
            .map_err(resource_error)?;
        Ok(json!({ "folders": folders.iter().map(folder_json).collect::<Vec<_>>() }))
    }

    fn create_folder(&mut self, arguments: &Value) -> Result<Value, BridgeError> {
        let session = self.session()?;
        let parent = FolderId::from_u128(number(
            arguments,
            "parent_folder_id",
            session.root_folder.as_u128(),
        )?);
        let id = FolderId::from_u128(session.next_folder);
        let name = required_string(arguments, "name")?;
        let visibility = visibility(optional_string(arguments, "visibility").as_deref())?;
        self.session.as_mut().expect("session exists").next_folder =
            session.next_folder.saturating_add(1);
        let space = self
            .registry
            .space_mut(session.store)
            .map_err(resource_error)?;
        let parsed_name = space.resource_name(&name).map_err(resource_error)?;
        space
            .create_folder(
                id,
                parent,
                parsed_name,
                ResourceMetadata {
                    visibility,
                    ..Default::default()
                },
            )
            .map_err(resource_error)?;
        Ok(json!({ "folder": folder_json(space.folder(id).expect("created folder exists")) }))
    }

    fn put_resource(&mut self, arguments: &Value) -> Result<Value, BridgeError> {
        let session = self.session()?;
        let parent = FolderId::from_u128(number(
            arguments,
            "parent_folder_id",
            session.root_folder.as_u128(),
        )?);
        let name = required_string(arguments, "name")?;
        let bytes = STANDARD
            .decode(required_string(arguments, "bytes_base64")?)
            .map_err(|_| {
                error(
                    "command.invalid_argument",
                    "bytes_base64 must be valid standard Base64",
                )
            })?;
        let metadata = ResourceMetadata {
            visibility: visibility(optional_string(arguments, "visibility").as_deref())?,
            media_type: optional_string(arguments, "media_type"),
            ..Default::default()
        };
        let space = self
            .registry
            .space_mut(session.store)
            .map_err(resource_error)?;
        let parsed_name = space.resource_name(&name).map_err(resource_error)?;
        let entry = match space
            .resource(parent, &parsed_name)
            .map_err(resource_error)?
        {
            Some(_) => space.replace_resource(parent, &parsed_name, bytes, metadata),
            None => space.insert_resource(parent, parsed_name, bytes, metadata),
        }
        .map_err(resource_error)?;
        Ok(json!({ "resource": resource_json(&entry) }))
    }

    fn get_resource(&self, arguments: &Value) -> Result<Value, BridgeError> {
        let session = self.session()?;
        let parent = FolderId::from_u128(number(
            arguments,
            "parent_folder_id",
            session.root_folder.as_u128(),
        )?);
        let name = required_string(arguments, "name")?;
        let space = self.registry.space(session.store).map_err(resource_error)?;
        let parsed_name = space.resource_name(&name).map_err(resource_error)?;
        let entry = space
            .resource(parent, &parsed_name)
            .map_err(resource_error)?
            .ok_or_else(|| error("resource.not_found", "resource does not exist"))?;
        Ok(
            json!({ "resource": resource_json(&entry), "bytes_base64": STANDARD.encode(entry.bytes()) }),
        )
    }

    fn list_resources(&self, arguments: &Value) -> Result<Value, BridgeError> {
        let session = self.session()?;
        let parent = FolderId::from_u128(number(
            arguments,
            "parent_folder_id",
            session.root_folder.as_u128(),
        )?);
        let query = visibility_query(optional_string(arguments, "visibility").as_deref())?;
        let entries = self
            .registry
            .space(session.store)
            .map_err(resource_error)?
            .list_resources(parent, query)
            .map_err(resource_error)?;
        Ok(json!({ "resources": entries.iter().map(resource_json).collect::<Vec<_>>() }))
    }

    fn set_resource_visibility(&mut self, arguments: &Value) -> Result<Value, BridgeError> {
        let session = self.session()?;
        let parent = FolderId::from_u128(number(
            arguments,
            "parent_folder_id",
            session.root_folder.as_u128(),
        )?);
        let name = required_string(arguments, "name")?;
        let visibility = visibility(optional_string(arguments, "visibility").as_deref())?;
        let space = self
            .registry
            .space_mut(session.store)
            .map_err(resource_error)?;
        let parsed_name = space.resource_name(&name).map_err(resource_error)?;
        let entry = space
            .set_resource_visibility(parent, &parsed_name, visibility)
            .map_err(resource_error)?;
        Ok(json!({ "resource": resource_json(&entry) }))
    }

    fn move_resource(&mut self, arguments: &Value) -> Result<Value, BridgeError> {
        let session = self.session()?;
        let source_parent = FolderId::from_u128(number(
            arguments,
            "source_parent_folder_id",
            session.root_folder.as_u128(),
        )?);
        let destination_parent = FolderId::from_u128(number(
            arguments,
            "destination_parent_folder_id",
            session.root_folder.as_u128(),
        )?);
        let source_name = required_string(arguments, "source_name")?;
        let destination_name = required_string(arguments, "destination_name")?;
        let space = self
            .registry
            .space_mut(session.store)
            .map_err(resource_error)?;
        let source = space.resource_name(&source_name).map_err(resource_error)?;
        let destination = space
            .resource_name(&destination_name)
            .map_err(resource_error)?;
        let entry = space
            .move_resource(source_parent, &source, destination_parent, destination)
            .map_err(resource_error)?;
        Ok(json!({ "resource": resource_json(&entry) }))
    }

    fn transform_resource_compression(&mut self, arguments: &Value) -> Result<Value, BridgeError> {
        let session = self.session()?;
        let source_folder = FolderId::from_u128(number(
            arguments,
            "source_folder_id",
            session.root_folder.as_u128(),
        )?);
        let destination_folder = FolderId::from_u128(number(
            arguments,
            "destination_folder_id",
            source_folder.as_u128(),
        )?);
        let source_name = required_string(arguments, "source_name")?;
        let destination_name = required_string(arguments, "destination_name")?;
        let codec = compression_codec(required_string(arguments, "codec")?.as_str())?;
        let transform = match required_string(arguments, "operation")?.as_str() {
            "encode" => ResourceCompressionTransform::Encode {
                codec,
                goal: compression_goal(optional_string(arguments, "goal").as_deref())?,
            },
            "decode" => ResourceCompressionTransform::Decode {
                codec,
                limits: DecodeLimits::new(
                    number(arguments, "max_input_bytes", 16 * 1024 * 1024)? as u64,
                    number(arguments, "max_output_bytes", 64 * 1024 * 1024)? as u64,
                )
                .with_expansion_ratio(number(
                    arguments,
                    "max_expansion_ratio",
                    100,
                )? as u32),
            },
            _ => {
                return Err(error(
                    "command.invalid_argument",
                    "operation must be encode or decode",
                ));
            }
        };
        let collision = match optional_string(arguments, "collision")
            .as_deref()
            .unwrap_or("reject")
        {
            "reject" => ResourceCollisionPolicy::Reject,
            "replace" => ResourceCollisionPolicy::Replace,
            _ => {
                return Err(error(
                    "command.invalid_argument",
                    "collision must be reject or replace",
                ));
            }
        };
        let metadata = ResourceMetadata {
            visibility: visibility(optional_string(arguments, "visibility").as_deref())?,
            media_type: optional_string(arguments, "media_type"),
            ..Default::default()
        };
        let space = self
            .registry
            .space_mut(session.store)
            .map_err(resource_error)?;
        let source_name = space.resource_name(&source_name).map_err(resource_error)?;
        let destination_name = space
            .resource_name(&destination_name)
            .map_err(resource_error)?;
        let result = transform_resource(
            space,
            ResourceCompressionRequest {
                source_folder,
                source_name,
                destination_folder,
                destination_name,
                transform,
                collision,
                metadata,
            },
            &FlateCompressionProvider,
        )
        .map_err(|value| error("resource_space.compression_rejected", value.to_string()))?;
        Ok(json!({
            "resource": resource_json(result.entry()),
            "observation": compression_observation_json(result.observation()),
        }))
    }

    fn inspect_archive_resource(&self, arguments: &Value) -> Result<Value, BridgeError> {
        let session = self.session()?;
        let source_folder = FolderId::from_u128(number(
            arguments,
            "source_folder_id",
            session.root_folder.as_u128(),
        )?);
        let source_name = required_string(arguments, "source_name")?;
        let format = archive_format(required_string(arguments, "format")?.as_str())?;
        let limits = ArchiveReadLimits::new(
            checked_u64(arguments, "max_archive_bytes", 16 * 1024 * 1024)?,
            checked_u32(arguments, "max_entries", 1_024)?,
            checked_u64(arguments, "max_entry_bytes", 16 * 1024 * 1024)?,
            checked_u64(arguments, "max_total_output_bytes", 64 * 1024 * 1024)?,
            checked_u32(arguments, "max_path_bytes", 4_096)?,
        );
        let space = self.registry.space(session.store).map_err(resource_error)?;
        let source_name = space.resource_name(&source_name).map_err(resource_error)?;
        let request = InspectArchiveResourceRequest {
            source_folder,
            source_name,
            format,
            limits,
        };
        let inspection = match format {
            ArchiveFormat::Zip => inspect_resource_archive(space, request, &ZipArchiveProvider),
            ArchiveFormat::Tar => inspect_resource_archive(space, request, &TarArchiveProvider),
            ArchiveFormat::SevenZip => {
                inspect_resource_archive(space, request, &SevenZipArchiveProvider)
            }
        }
        .map_err(|value| error("resource_space.archive_rejected", value.to_string()))?;
        Ok(json!({ "observation": archive_inspection_json(&inspection) }))
    }

    fn import_archive_subtree(&mut self, arguments: &Value) -> Result<Value, BridgeError> {
        let session = self.session()?;
        let source_folder = FolderId::from_u128(number(
            arguments,
            "source_folder_id",
            session.root_folder.as_u128(),
        )?);
        let source_name = required_string(arguments, "source_name")?;
        let format = archive_format(required_string(arguments, "format")?.as_str())?;
        let destination_parent = FolderId::from_u128(number(
            arguments,
            "destination_parent_id",
            session.root_folder.as_u128(),
        )?);
        let destination_root_name = required_string(arguments, "destination_root_name")?;
        let limits = ArchiveReadLimits::new(
            checked_u64(arguments, "max_archive_bytes", 16 * 1024 * 1024)?,
            checked_u32(arguments, "max_entries", 1_024)?,
            checked_u64(arguments, "max_entry_bytes", 16 * 1024 * 1024)?,
            checked_u64(arguments, "max_total_output_bytes", 64 * 1024 * 1024)?,
            checked_u32(arguments, "max_path_bytes", 4_096)?,
        );
        let metadata = ResourceMetadata {
            visibility: visibility(optional_string(arguments, "visibility").as_deref())?,
            media_type: optional_string(arguments, "media_type"),
            ..Default::default()
        };
        let space = self
            .registry
            .space_mut(session.store)
            .map_err(resource_error)?;
        let source_name = space.resource_name(&source_name).map_err(resource_error)?;
        let destination_root_name = space
            .resource_name(&destination_root_name)
            .map_err(resource_error)?;
        let request = ImportArchiveSubtreeRequest {
            source_folder,
            source_name,
            format,
            limits,
            destination_parent,
            destination_root_name,
            first_folder_id: FolderId::from_u128(session.next_folder),
            metadata,
        };
        let observation = match format {
            ArchiveFormat::Zip => {
                import_archive_subtree(space, request.clone(), &ZipArchiveProvider)
            }
            ArchiveFormat::Tar => {
                import_archive_subtree(space, request.clone(), &TarArchiveProvider)
            }
            ArchiveFormat::SevenZip => {
                import_archive_subtree(space, request, &SevenZipArchiveProvider)
            }
        }
        .map_err(|value| error("resource_space.archive_rejected", value.to_string()))?;

        let next_folder = session
            .next_folder
            .checked_add(u128::from(observation.folders()))
            .ok_or_else(|| {
                error(
                    "resource_space.archive_rejected",
                    "folder identifier range exhausted",
                )
            })?;
        self.session.as_mut().expect("session exists").next_folder = next_folder;
        Ok(json!({ "observation": archive_subtree_import_json(&observation) }))
    }

    fn summary(&self) -> Result<Value, BridgeError> {
        let session = self.session()?;
        let summary = self
            .registry
            .space(session.store)
            .map_err(resource_error)?
            .summary();
        Ok(
            json!({ "store_id": session.store.as_u128().to_string(), "root_id": session.root.as_u128().to_string(),
            "roots": summary.roots(), "folders": summary.folders(), "resources": summary.resources(), "retained_bytes": summary.retained_bytes() }),
        )
    }

    fn provider_inspect(&self) -> Result<Value, BridgeError> {
        let session = self.session()?;
        match &self.provider {
            ProviderState::InMemory => Ok(json!({
                "provider": "in_memory", "durable": false,
                "store_id": session.store.as_u128().to_string(),
                "note": "This session uses the deterministic in-memory provider."
            })),
            ProviderState::Tosumu(_) => Ok(json!({
                "provider": "tosumu", "durable": true,
                "store_id": session.store.as_u128().to_string(),
                "note": "Tosumu retains a versioned consumer-local Resource Space snapshot through its public key/value provider.",
            })),
        }
    }

    fn persist_active_state(&mut self) -> Result<(), BridgeError> {
        if matches!(&self.provider, ProviderState::InMemory) {
            return Ok(());
        }
        let session = self.session()?;
        let snapshot = PersistedResourceSpace::capture(
            &self.registry,
            session.store,
            session.root,
            session.root_folder,
            session.next_folder,
        )
        .map_err(|message| error("provider.tosumu.snapshot", message))?;
        match &mut self.provider {
            ProviderState::InMemory => Ok(()),
            ProviderState::Tosumu(provider) => provider
                .save(&snapshot)
                .map_err(|message| error("provider.tosumu.persist", message)),
        }
    }

    fn session(&self) -> Result<Session, BridgeError> {
        self.session.ok_or_else(|| {
            error(
                "session.required",
                "session.create_or_open must succeed before resource commands run",
            )
        })
    }
}

fn response(
    request_id: String,
    ok: bool,
    result: Option<Value>,
    error_value: Option<BridgeError>,
) -> BridgeResponse {
    BridgeResponse {
        schema: BRIDGE_SCHEMA_VERSION,
        request_id,
        ok,
        result,
        error: error_value,
    }
}

fn is_mutating_command(command: &str) -> bool {
    matches!(
        command,
        "folder.create"
            | "resource.put"
            | "resource.set_visibility"
            | "resource.move"
            | "resource.transform_compression"
            | "resource.import_archive_subtree"
    )
}
fn error(kind: &'static str, message: impl Into<String>) -> BridgeError {
    BridgeError {
        kind,
        message: message.into(),
    }
}
fn optional_string(arguments: &Value, field: &str) -> Option<String> {
    arguments
        .get(field)?
        .as_str()
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}
fn required_string(arguments: &Value, field: &str) -> Result<String, BridgeError> {
    optional_string(arguments, field).ok_or_else(|| {
        error(
            "command.invalid_argument",
            format!("{field} must be a non-empty string"),
        )
    })
}
fn number(arguments: &Value, field: &str, default: u128) -> Result<u128, BridgeError> {
    optional_string(arguments, field).map_or(Ok(default), |value| {
        value.parse().map_err(|_| {
            error(
                "command.invalid_argument",
                format!("{field} must be a decimal u128 string"),
            )
        })
    })
}
fn checked_u64(arguments: &Value, field: &str, default: u64) -> Result<u64, BridgeError> {
    let value = number(arguments, field, default as u128)?;
    u64::try_from(value).map_err(|_| {
        error(
            "command.invalid_argument",
            format!("{field} must fit in an unsigned 64-bit integer"),
        )
    })
}
fn checked_u32(arguments: &Value, field: &str, default: u32) -> Result<u32, BridgeError> {
    let value = number(arguments, field, default as u128)?;
    u32::try_from(value).map_err(|_| {
        error(
            "command.invalid_argument",
            format!("{field} must fit in an unsigned 32-bit integer"),
        )
    })
}
fn case_policy(value: Option<&str>) -> Result<AddressCasePolicy, BridgeError> {
    match value.unwrap_or("sensitive") {
        "sensitive" => Ok(AddressCasePolicy::Sensitive),
        "insensitive" => Ok(AddressCasePolicy::Insensitive),
        _ => Err(error(
            "command.invalid_argument",
            "case_policy must be sensitive or insensitive",
        )),
    }
}
fn case_policy_name(value: AddressCasePolicy) -> &'static str {
    match value {
        AddressCasePolicy::Sensitive => "sensitive",
        AddressCasePolicy::Insensitive => "insensitive",
    }
}
fn visibility(value: Option<&str>) -> Result<ResourceVisibility, BridgeError> {
    match value.unwrap_or("visible") {
        "visible" => Ok(ResourceVisibility::Visible),
        "hidden" => Ok(ResourceVisibility::Hidden),
        _ => Err(error(
            "command.invalid_argument",
            "visibility must be visible or hidden",
        )),
    }
}
fn visibility_query(value: Option<&str>) -> Result<VisibilityQuery, BridgeError> {
    match value.unwrap_or("all") {
        "visible" => Ok(VisibilityQuery::VisibleOnly),
        "hidden" => Ok(VisibilityQuery::HiddenOnly),
        "all" => Ok(VisibilityQuery::All),
        _ => Err(error(
            "command.invalid_argument",
            "visibility must be visible, hidden, or all",
        )),
    }
}
fn compression_codec(value: &str) -> Result<CompressionCodec, BridgeError> {
    match value {
        "gzip" => Ok(CompressionCodec::Gzip),
        "deflate" => Ok(CompressionCodec::Deflate),
        "brotli" => Ok(CompressionCodec::Brotli),
        _ => Err(error(
            "command.invalid_argument",
            "codec must be gzip, deflate, or brotli",
        )),
    }
}
fn compression_goal(value: Option<&str>) -> Result<CompressionGoal, BridgeError> {
    match value.unwrap_or("balanced") {
        "fast" => Ok(CompressionGoal::Fast),
        "balanced" => Ok(CompressionGoal::Balanced),
        "small" => Ok(CompressionGoal::Small),
        _ => Err(error(
            "command.invalid_argument",
            "goal must be fast, balanced, or small",
        )),
    }
}
fn archive_format(value: &str) -> Result<ArchiveFormat, BridgeError> {
    match value {
        "zip" => Ok(ArchiveFormat::Zip),
        "tar" => Ok(ArchiveFormat::Tar),
        "7z" => Ok(ArchiveFormat::SevenZip),
        _ => Err(error(
            "command.invalid_argument",
            "format must be zip, tar, or 7z",
        )),
    }
}
fn archive_format_name(value: ArchiveFormat) -> &'static str {
    match value {
        ArchiveFormat::Zip => "zip",
        ArchiveFormat::Tar => "tar",
        ArchiveFormat::SevenZip => "7z",
    }
}
fn resource_error(value: impl std::fmt::Display) -> BridgeError {
    error("resource_space.rejected", value.to_string())
}
fn visibility_name(value: ResourceVisibility) -> &'static str {
    match value {
        ResourceVisibility::Visible => "visible",
        ResourceVisibility::Hidden => "hidden",
    }
}
fn folder_json(folder: &FolderEntry) -> Value {
    json!({ "id": folder.id().as_u128().to_string(), "parent_id": folder.parent().map(|id| id.as_u128().to_string()), "root_id": folder.root().as_u128().to_string(), "name": folder.name().map(|name| name.as_str()), "visibility": visibility_name(folder.metadata().visibility), "is_root": folder.is_root_folder() })
}
fn resource_json(entry: &ResourceEntry) -> Value {
    let fingerprint = entry.content_fingerprint();
    let fingerprint = fingerprint
        .digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    json!({
        "store_id": entry.key().store().as_u128().to_string(),
        "root_id": entry.key().root().as_u128().to_string(),
        "address": entry.key().address().to_string(),
        "parent_folder_id": entry.parent().as_u128().to_string(),
        "name": entry.name().as_str(),
        "byte_length": entry.byte_len(),
        "visibility": visibility_name(entry.metadata().visibility),
        "media_type": entry.metadata().media_type.clone(),
        "content_fingerprint": format!("blake3:{fingerprint}"),
    })
}
fn compression_codec_name(value: CompressionCodec) -> &'static str {
    match value {
        CompressionCodec::Gzip => "gzip",
        CompressionCodec::Brotli => "brotli",
        CompressionCodec::Deflate => "deflate",
    }
}
fn transform_mutation_name(value: ResourceTransformMutation) -> &'static str {
    match value {
        ResourceTransformMutation::Inserted => "inserted",
        ResourceTransformMutation::Replaced => "replaced",
    }
}
fn fingerprint_json(value: &resource_space::ContentFingerprint) -> String {
    let digest = value
        .digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("blake3:{digest}")
}
fn compression_observation_json(value: &ResourceCompressionObservation) -> Value {
    json!({
        "source_address": value.source().address().to_string(),
        "source_fingerprint": fingerprint_json(value.source_fingerprint()),
        "result_address": value.result().address().to_string(),
        "result_fingerprint": fingerprint_json(value.result_fingerprint()),
        "codec": compression_codec_name(value.compression().codec),
        "input_bytes": value.compression().input_bytes,
        "output_bytes": value.compression().output_bytes,
        "mutation": transform_mutation_name(value.mutation()),
    })
}
fn archive_inspection_json(value: &ResourceArchiveInspection) -> Value {
    let manifest = value.manifest();
    json!({
        "source_address": value.source().address().to_string(),
        "source_fingerprint": fingerprint_json(value.source_fingerprint()),
        "format": archive_format_name(manifest.format),
        "archive_bytes": manifest.archive_bytes,
        "total_uncompressed_bytes": manifest.total_uncompressed_bytes,
        "entries": manifest.entries,
    })
}
fn archive_subtree_import_json(value: &ArchiveSubtreeImportObservation) -> Value {
    json!({
        "source_address": value.source().address().to_string(),
        "source_fingerprint": fingerprint_json(value.source_fingerprint()),
        "destination_root_id": value.destination_root().as_u128().to_string(),
        "folders": value.folders(),
        "resources": value.resources(),
        "retained_bytes": value.retained_bytes(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use archive_provider::{
        ArchiveCompression, ArchiveWriteEntry, ArchiveWriteLimits, ArchiveWriter,
    };

    fn request(command: &str, arguments: Value) -> BridgeRequest {
        BridgeRequest {
            schema: BRIDGE_SCHEMA_VERSION,
            request_id: "test".to_owned(),
            command: command.to_owned(),
            arguments,
        }
    }
    #[test]
    fn session_preserves_resource_space_visibility_and_bytes() {
        let mut bridge = ResourceBridge::default();
        assert!(
            bridge
                .execute(request("session.create_or_open", json!({})))
                .ok
        );
        assert!(
            bridge
                .execute(request(
                    "resource.put",
                    json!({ "name": "note.txt", "bytes_base64": STANDARD.encode(b"hello") })
                ))
                .ok
        );
        assert!(
            bridge
                .execute(request(
                    "resource.set_visibility",
                    json!({ "name": "note.txt", "visibility": "hidden" })
                ))
                .ok
        );
        let visible = bridge.execute(request("resource.list", json!({ "visibility": "visible" })));
        assert_eq!(
            visible.result.unwrap()["resources"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        let fetched = bridge.execute(request("resource.get", json!({ "name": "note.txt" })));
        assert_eq!(
            fetched.result.unwrap()["bytes_base64"],
            STANDARD.encode(b"hello")
        );
    }
    #[test]
    fn commands_require_a_session() {
        let mut bridge = ResourceBridge::default();
        let response = bridge.execute(request("resource.list", json!({})));
        assert!(!response.ok);
        assert_eq!(response.error.unwrap().kind, "session.required");
    }

    #[test]
    fn compression_command_preserves_source_and_round_trips_gzip_bytes() {
        let mut bridge = ResourceBridge::default();
        assert!(
            bridge
                .execute(request("session.create_or_open", json!({})))
                .ok
        );
        assert!(bridge
            .execute(request(
                "resource.put",
                json!({
                    "name": "report.txt",
                    "bytes_base64": STANDARD.encode(b"compression bridge fixture compression bridge fixture"),
                    "media_type": "text/plain",
                }),
            ))
            .ok);
        let encoded = bridge.execute(request(
            "resource.transform_compression",
            json!({
                "source_name": "report.txt",
                "destination_name": "report.txt.gz",
                "operation": "encode",
                "codec": "gzip",
                "goal": "balanced",
                "media_type": "application/gzip",
            }),
        ));
        assert!(encoded.ok);
        let encoded = encoded.result.expect("encoded result");
        assert_eq!(encoded["observation"]["codec"], "gzip");
        assert_eq!(encoded["observation"]["mutation"], "inserted");

        let decoded = bridge.execute(request(
            "resource.transform_compression",
            json!({
                "source_name": "report.txt.gz",
                "destination_name": "report-copy.txt",
                "operation": "decode",
                "codec": "gzip",
                "media_type": "text/plain",
            }),
        ));
        assert!(decoded.ok);
        let fetched = bridge.execute(request(
            "resource.get",
            json!({ "name": "report-copy.txt" }),
        ));
        assert_eq!(
            fetched.result.expect("decoded resource")["bytes_base64"],
            STANDARD.encode(b"compression bridge fixture compression bridge fixture")
        );
        let source = bridge.execute(request("resource.get", json!({ "name": "report.txt" })));
        assert_eq!(
            source.result.expect("source resource")["bytes_base64"],
            STANDARD.encode(b"compression bridge fixture compression bridge fixture")
        );
    }

    #[test]
    fn archive_inspection_returns_a_provider_neutral_zip_manifest() {
        let fixture = ZipArchiveProvider
            .write_archive(
                ArchiveFormat::Zip,
                &[
                    ArchiveWriteEntry::directory("docs/"),
                    ArchiveWriteEntry::file(
                        "docs/readme.txt",
                        b"archive bridge fixture".to_vec(),
                        ArchiveCompression::Deflate,
                    ),
                ],
                ArchiveWriteLimits::default(),
            )
            .expect("write deterministic zip fixture")
            .bytes;
        let mut bridge = ResourceBridge::default();
        assert!(
            bridge
                .execute(request("session.create_or_open", json!({})))
                .ok
        );
        assert!(
            bridge
                .execute(request(
                    "resource.put",
                    json!({
                        "name": "fixture.zip",
                        "bytes_base64": STANDARD.encode(fixture),
                        "media_type": "application/zip",
                    }),
                ))
                .ok
        );

        let response = bridge.execute(request(
            "resource.inspect_archive",
            json!({ "source_name": "fixture.zip", "format": "zip" }),
        ));
        assert!(response.ok);
        let observation = response.result.expect("archive observation")["observation"].clone();
        assert_eq!(observation["format"], "zip");
        assert_eq!(observation["entries"].as_array().unwrap().len(), 2);
        assert_eq!(
            observation["entries"][1]["normalized_name"],
            "docs/readme.txt"
        );
        assert_eq!(observation["total_uncompressed_bytes"], 22);
    }

    #[test]
    fn archive_subtree_import_materializes_entries_and_advances_folder_ids() {
        let fixture = ZipArchiveProvider
            .write_archive(
                ArchiveFormat::Zip,
                &[
                    ArchiveWriteEntry::directory("docs/"),
                    ArchiveWriteEntry::file(
                        "docs/readme.txt",
                        b"archive bridge fixture".to_vec(),
                        ArchiveCompression::Deflate,
                    ),
                ],
                ArchiveWriteLimits::default(),
            )
            .expect("write deterministic zip fixture")
            .bytes;
        let mut bridge = ResourceBridge::default();
        assert!(
            bridge
                .execute(request("session.create_or_open", json!({})))
                .ok
        );
        assert!(
            bridge
                .execute(request(
                    "resource.put",
                    json!({ "name": "fixture.zip", "bytes_base64": STANDARD.encode(fixture) }),
                ))
                .ok
        );

        let imported = bridge.execute(request(
            "resource.import_archive_subtree",
            json!({
                "source_name": "fixture.zip",
                "format": "zip",
                "destination_root_name": "unpacked",
            }),
        ));
        assert!(imported.ok);
        let observation = imported.result.expect("import observation")["observation"].clone();
        assert_eq!(observation["folders"], 2);
        assert_eq!(observation["resources"], 1);
        assert_eq!(observation["retained_bytes"], 22);

        let root = bridge.execute(request("folder.list", json!({})));
        let root_result = root.result.expect("root folders");
        let unpacked = root_result["folders"]
            .as_array()
            .unwrap()
            .iter()
            .find(|folder| folder["name"] == "unpacked")
            .expect("import root folder");
        let unpacked_id = unpacked["id"].as_str().expect("folder id").to_owned();
        let child = bridge.execute(request(
            "folder.list",
            json!({ "parent_folder_id": unpacked_id }),
        ));
        let child_result = child.result.expect("import child folder");
        let docs_id = child_result["folders"][0]["id"]
            .as_str()
            .expect("docs folder id")
            .to_owned();
        let resources = bridge.execute(request(
            "resource.list",
            json!({ "parent_folder_id": docs_id }),
        ));
        let resources_result = resources.result.expect("imported resources");
        assert_eq!(resources_result["resources"][0]["name"], "readme.txt");
    }

    #[test]
    fn tosumu_provider_restores_hierarchy_bytes_and_visibility() {
        let store_path = std::env::temp_dir().join(format!(
            "tokimu-resource-workbench-{}-{}.tosumu",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock is after the Unix epoch")
                .as_nanos()
        ));
        let arguments = json!({
            "provider": "tosumu",
            "store_path": store_path,
            "store_id": "91",
            "root_id": "92",
            "root_folder_id": "93",
            "case_policy": "sensitive",
        });

        let mut first = ResourceBridge::default();
        assert!(
            first
                .execute(request("session.create_or_open", arguments.clone()))
                .ok
        );
        let folder = first.execute(request("folder.create", json!({ "name": "notes" })));
        let folder_id = folder.result.expect("folder result")["folder"]["id"]
            .as_str()
            .expect("folder id")
            .to_owned();
        assert!(
            first
                .execute(request(
                    "resource.put",
                    json!({
                        "parent_folder_id": folder_id,
                        "name": "persisted.txt",
                        "bytes_base64": STANDARD.encode(b"durable bytes"),
                        "visibility": "hidden",
                        "media_type": "text/plain",
                    })
                ))
                .ok
        );
        assert!(
            first
                .execute(request(
                    "resource.move",
                    json!({
                        "source_parent_folder_id": folder_id,
                        "source_name": "persisted.txt",
                        "destination_name": "moved.txt",
                    })
                ))
                .ok
        );
        drop(first);

        let mut second = ResourceBridge::default();
        let reopened = second.execute(request("session.create_or_open", arguments));
        assert!(reopened.ok);
        assert_eq!(
            reopened.result.expect("session result")["outcome"],
            "opened_existing"
        );
        let folders = second.execute(request("folder.list", json!({})));
        assert_eq!(
            folders.result.expect("retained folders")["folders"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        let hidden = second.execute(request("resource.list", json!({ "visibility": "hidden" })));
        assert_eq!(
            hidden.result.expect("hidden resources")["resources"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        let fetched = second.execute(request("resource.get", json!({ "name": "moved.txt" })));
        let fetched = fetched.result.expect("fetched resource");
        assert_eq!(fetched["bytes_base64"], STANDARD.encode(b"durable bytes"));
        assert_eq!(fetched["resource"]["media_type"], "text/plain");
        let provider = second.execute(request("provider.inspect", json!({})));
        assert_eq!(provider.result.expect("provider result")["durable"], true);

        drop(second);
        let mut incompatible = ResourceBridge::default();
        let conflict = incompatible.execute(request(
            "session.create_or_open",
            json!({
                "provider": "tosumu",
                "store_path": store_path,
                "store_id": "91",
                "root_id": "92",
                "root_folder_id": "93",
                "case_policy": "insensitive",
            }),
        ));
        assert!(!conflict.ok);
        assert_eq!(
            conflict.error.expect("identity conflict").kind,
            "provider.tosumu.identity_conflict"
        );
        drop(incompatible);
        let _ = std::fs::remove_file(&store_path);
        let _ = std::fs::remove_file(store_path.with_extension("tosumu.wal"));
    }
}
