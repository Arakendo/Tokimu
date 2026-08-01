use crate::{
    region::{UiCard, UiRegion},
    text::{UiTextAlign, UiTextOverflow, UiTextRole, UiTextSpec},
    UiButton, UiInteractionState, UiLabel, UiNodeKind, UiRect, UiResolvedNode, UiResolvedTree,
    UiStateChip, UiTreeDiagnosticKind,
};

use crate::theme::{UiControlRole, UiSurfaceStyle, UiTextStyle, UiTheme};
use crate::{PathBuilder, VectorPath};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiSurfaceCommand {
    pub rect: UiRect,
    pub style: UiSurfaceStyle,
    /// Optional rectangular scissor region supplied by the layout layer.
    ///
    /// Vector lowering preserves the semantic geometry and carries this
    /// metadata forward. It does not synthesize rounded corners at a clip
    /// boundary; applying the scissor remains a renderer-adapter concern.
    pub clip: Option<UiRect>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiSurfaceVectorLayerKind {
    Shadow,
    Border,
    Fill,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiSurfaceVectorLayer {
    pub kind: UiSurfaceVectorLayerKind,
    pub path: VectorPath,
    pub role: crate::UiSurfaceRole,
    pub opacity: f32,
    pub clip: Option<UiRect>,
}

/// Lowers one semantic surface into ordered vector presentation layers.
///
/// The result deliberately contains no renderer or material handles. The
/// renderer adapter decides how each semantic role is painted later.
pub fn lower_surface_to_vector(command: &UiSurfaceCommand) -> Vec<UiSurfaceVectorLayer> {
    let rect = command.rect;
    let radius = radius_value(command.style.radius);
    let mut layers = Vec::with_capacity(3);

    if matches!(
        command.style.elevation,
        crate::UiElevation::Raised | crate::UiElevation::Floating
    ) {
        layers.push(UiSurfaceVectorLayer {
            kind: UiSurfaceVectorLayerKind::Shadow,
            path: rounded_rect_path(rect, radius, [0.01, -0.01]),
            role: crate::UiSurfaceRole::Overlay,
            opacity: command.style.opacity,
            clip: command.clip,
        });
    }

    if let Some(role) = command.style.border_role {
        let border_rect = UiRect::new(
            rect.center,
            [
                rect.size[0] + command.style.border_width * 2.0,
                rect.size[1] + command.style.border_width * 2.0,
            ],
        );
        layers.push(UiSurfaceVectorLayer {
            kind: UiSurfaceVectorLayerKind::Border,
            path: rounded_rect_path(border_rect, radius, [0.0, 0.0]),
            role,
            opacity: command.style.opacity,
            clip: command.clip,
        });
    }

    layers.push(UiSurfaceVectorLayer {
        kind: UiSurfaceVectorLayerKind::Fill,
        path: rounded_rect_path(rect, radius, [0.0, 0.0]),
        role: command.style.role,
        opacity: command.style.opacity,
        clip: command.clip,
    });
    layers
}

fn rounded_rect_path(rect: UiRect, radius: f32, offset: [f32; 2]) -> VectorPath {
    let min = [
        rect.center[0] - rect.size[0] * 0.5 + offset[0],
        rect.center[1] - rect.size[1] * 0.5 + offset[1],
    ];
    PathBuilder::new()
        .rounded_rect(min, rect.size, radius)
        .build()
}

fn radius_value(radius: crate::UiRadius) -> f32 {
    match radius {
        crate::UiRadius::None => 0.0,
        crate::UiRadius::Small => 0.01,
        crate::UiRadius::Medium => 0.02,
        crate::UiRadius::Large => 0.04,
    }
}

/// Backend-neutral text draw request.
///
/// This contains semantic text and theme style only. Font rasterizers, glyph
/// atlases, meshes, and GPU handles are intentionally resolved downstream.
#[derive(Clone, Debug, PartialEq)]
pub struct UiTextCommand {
    pub spec: UiTextSpec,
    pub style: UiTextStyle,
}

impl UiTextCommand {
    pub fn new(spec: UiTextSpec, style: UiTextStyle) -> Self {
        Self { spec, style }
    }
}

/// One renderer-neutral operation in an ordered UI draw list.
///
/// The list intentionally carries semantic presentation requests rather than
/// meshes, glyph atlases, bind groups, or other backend-owned resources.
#[derive(Clone, Debug, PartialEq)]
pub enum UiDrawCommand {
    PushClip(UiRect),
    PopClip,
    Surface(UiSurfaceCommand),
    Text(UiTextCommand),
}

/// A deterministic draw operation with optional semantic-tree provenance.
#[derive(Clone, Debug, PartialEq)]
pub struct UiDrawEntry {
    pub source: Option<crate::UiNodeId>,
    pub layer: u32,
    pub order: u32,
    pub command: UiDrawCommand,
}

/// Renderer-neutral output produced by presentation lowering.
///
/// `revision` belongs to the semantic producer. Renderers can use it to
/// decide whether their own backend caches require work, but cache lifetime
/// and GPU resources remain renderer-owned.
#[derive(Clone, Debug, PartialEq)]
pub struct UiDrawList {
    pub revision: u64,
    pub diagnostics: Vec<UiDrawListDiagnostic>,
    entries: Vec<UiDrawEntry>,
}

/// Stable identity for renderer-neutral draw work.
///
/// This key deliberately excludes semantic provenance, producer revisions,
/// and diagnostics. Renderer adapters may use it to index their own caches,
/// but the key does not prescribe cache lifetime, residency, or GPU resources.
/// It is stable for the current draw schema, not a cross-version file format.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiDrawCacheKey(u64);

impl UiDrawCacheKey {
    pub fn value(self) -> u64 {
        self.0
    }
}

/// Bounded structural evidence for one renderer-neutral draw list.
///
/// Batch candidates are contiguous runs with compatible semantic styles and
/// clips. Ordered UI layers do not split a candidate by themselves because a
/// renderer may preserve that order while grouping adjacent instances. These
/// counts remain a grouping signal, not a bound or guarantee about backend
/// submits.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiDrawListStatistics {
    pub entries: u32,
    pub surfaces: u32,
    pub text: u32,
    pub clip_pushes: u32,
    pub clip_pops: u32,
    pub surface_batch_candidates: u32,
    pub text_batch_candidates: u32,
}

impl UiDrawList {
    pub fn entries(&self) -> &[UiDrawEntry] {
        &self.entries
    }

    /// Returns a deterministic fingerprint of executable presentation content.
    ///
    /// The producer revision and lowering diagnostics are intentionally
    /// excluded: two equivalent semantic trees may be rebuilt at different
    /// revisions while still requiring identical renderer-neutral work. This
    /// is regression evidence, not a cross-version serialization format.
    pub fn structural_fingerprint(&self) -> u64 {
        let mut fingerprint = DrawListFingerprint::default();
        for entry in &self.entries {
            fingerprint.write_debug(&entry.source);
            fingerprint.write_u32(entry.layer);
            fingerprint.write_u32(entry.order);
            match &entry.command {
                UiDrawCommand::PushClip(clip) => {
                    fingerprint.write_u8(0);
                    fingerprint.write_debug(clip);
                }
                UiDrawCommand::PopClip => fingerprint.write_u8(1),
                UiDrawCommand::Surface(surface) => {
                    fingerprint.write_u8(2);
                    fingerprint.write_debug(surface);
                }
                UiDrawCommand::Text(text) => {
                    fingerprint.write_u8(3);
                    fingerprint.write_debug(text);
                }
            }
        }
        fingerprint.finish()
    }

    /// Returns stable identity for executable renderer-neutral work.
    ///
    /// Unlike `structural_fingerprint`, this excludes semantic node provenance
    /// because provenance does not change the pixels a renderer must execute.
    pub fn cache_key(&self) -> UiDrawCacheKey {
        let mut fingerprint = DrawListFingerprint::default();
        for entry in &self.entries {
            fingerprint.write_u32(entry.layer);
            fingerprint.write_u32(entry.order);
            write_draw_command_fingerprint(&mut fingerprint, &entry.command);
        }
        UiDrawCacheKey(fingerprint.finish())
    }

    /// Summarizes draw work and conservative contiguous batch candidates.
    pub fn statistics(&self) -> UiDrawListStatistics {
        let mut statistics = UiDrawListStatistics::default();
        let mut previous_surface: Option<(UiSurfaceStyle, Option<UiRect>)> = None;
        let mut previous_text: Option<UiTextStyle> = None;

        for entry in &self.entries {
            statistics.entries = statistics.entries.saturating_add(1);
            match &entry.command {
                UiDrawCommand::PushClip(_) => {
                    statistics.clip_pushes = statistics.clip_pushes.saturating_add(1);
                    previous_surface = None;
                    previous_text = None;
                }
                UiDrawCommand::PopClip => {
                    statistics.clip_pops = statistics.clip_pops.saturating_add(1);
                    previous_surface = None;
                    previous_text = None;
                }
                UiDrawCommand::Surface(surface) => {
                    statistics.surfaces = statistics.surfaces.saturating_add(1);
                    let signature = (surface.style, surface.clip);
                    if previous_surface != Some(signature) {
                        statistics.surface_batch_candidates =
                            statistics.surface_batch_candidates.saturating_add(1);
                    }
                    previous_surface = Some(signature);
                    previous_text = None;
                }
                UiDrawCommand::Text(text) => {
                    statistics.text = statistics.text.saturating_add(1);
                    let signature = text.style;
                    if previous_text != Some(signature) {
                        statistics.text_batch_candidates =
                            statistics.text_batch_candidates.saturating_add(1);
                    }
                    previous_text = Some(signature);
                    previous_surface = None;
                }
            }
        }

        statistics
    }

    /// Adapts the legacy parallel command collections into one ordered output.
    ///
    /// Legacy drawers historically submitted surfaces before text. Preserve
    /// that behavior here while newer callers construct a list directly and
    /// can interleave commands deliberately.
    pub fn from_legacy_commands(
        revision: u64,
        surfaces: &[UiSurfaceCommand],
        text: &[UiTextCommand],
    ) -> Self {
        let mut builder = UiDrawListBuilder::new(revision);
        builder.record_diagnostic(UiDrawListDiagnostic {
            source: None,
            kind: UiDrawListDiagnosticKind::LegacyParallelCommandsAdapted,
        });
        for surface in surfaces {
            builder.surface(None, 0, *surface);
        }
        for command in text {
            builder.text(None, 1, command.clone());
        }
        // The adapter creates no nested clips and uses monotonic layers.
        builder
            .finish()
            .expect("legacy UI draw commands are ordered")
    }
}

fn write_draw_command_fingerprint(fingerprint: &mut DrawListFingerprint, command: &UiDrawCommand) {
    match command {
        UiDrawCommand::PushClip(clip) => {
            fingerprint.write_u8(0);
            fingerprint.write_debug(clip);
        }
        UiDrawCommand::PopClip => fingerprint.write_u8(1),
        UiDrawCommand::Surface(surface) => {
            fingerprint.write_u8(2);
            fingerprint.write_debug(surface);
        }
        UiDrawCommand::Text(text) => {
            fingerprint.write_u8(3);
            fingerprint.write_debug(text);
        }
    }
}

/// Fixed FNV-1a hashing keeps corpus comparison independent from randomized
/// standard-library hashers. The draw-list schema is represented by its stable
/// debug fields until a versioned artifact serializer is independently needed.
#[derive(Clone, Copy, Debug)]
struct DrawListFingerprint(u64);

impl Default for DrawListFingerprint {
    fn default() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
}

impl DrawListFingerprint {
    fn write_u8(&mut self, value: u8) {
        self.write_bytes(&[value]);
    }

    fn write_u32(&mut self, value: u32) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_debug(&mut self, value: &impl std::fmt::Debug) {
        self.write_bytes(format!("{value:?}").as_bytes());
        self.write_u8(0xff);
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    fn finish(self) -> u64 {
        self.0
    }
}

/// Bounded evidence emitted while semantic UI becomes executable draw commands.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiDrawListDiagnostic {
    pub source: Option<crate::UiNodeId>,
    pub kind: UiDrawListDiagnosticKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiDrawListDiagnosticKind {
    LegacyParallelCommandsAdapted,
    TextOverflow,
    TextProviderUnavailable,
    MissingGlyph { character: char },
}

/// Bounded errors reported before a renderer adapter consumes a draw list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiDrawListError {
    LayerOrder { previous: u32, next: u32 },
    ClipUnderflow { order: u32 },
    UnclosedClips { remaining: u32 },
}

/// Builds a deterministic, validated `UiDrawList`.
#[derive(Debug)]
pub struct UiDrawListBuilder {
    revision: u64,
    entries: Vec<UiDrawEntry>,
    diagnostics: Vec<UiDrawListDiagnostic>,
    next_order: u32,
    last_layer: Option<u32>,
    clip_depth: u32,
    error: Option<UiDrawListError>,
}

impl UiDrawListBuilder {
    pub fn new(revision: u64) -> Self {
        Self {
            revision,
            entries: Vec::new(),
            diagnostics: Vec::new(),
            next_order: 0,
            last_layer: None,
            clip_depth: 0,
            error: None,
        }
    }

    pub fn push_clip(&mut self, source: Option<crate::UiNodeId>, layer: u32, clip: UiRect) {
        self.push(source, layer, UiDrawCommand::PushClip(clip));
        self.clip_depth = self.clip_depth.saturating_add(1);
    }

    pub fn pop_clip(&mut self, source: Option<crate::UiNodeId>, layer: u32) {
        let order = self.next_order;
        self.push(source, layer, UiDrawCommand::PopClip);
        if self.clip_depth == 0 {
            self.error
                .get_or_insert(UiDrawListError::ClipUnderflow { order });
        } else {
            self.clip_depth -= 1;
        }
    }

    pub fn surface(
        &mut self,
        source: Option<crate::UiNodeId>,
        layer: u32,
        command: UiSurfaceCommand,
    ) {
        self.push(source, layer, UiDrawCommand::Surface(command));
    }

    pub fn text(&mut self, source: Option<crate::UiNodeId>, layer: u32, command: UiTextCommand) {
        self.push(source, layer, UiDrawCommand::Text(command));
    }

    /// Adds bounded diagnostic evidence without exposing backend concerns.
    pub fn record_diagnostic(&mut self, diagnostic: UiDrawListDiagnostic) {
        const MAX_DRAW_LIST_DIAGNOSTICS: usize = 128;
        if self.diagnostics.len() < MAX_DRAW_LIST_DIAGNOSTICS {
            self.diagnostics.push(diagnostic);
        }
    }

    pub fn finish(self) -> Result<UiDrawList, UiDrawListError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        if self.clip_depth != 0 {
            return Err(UiDrawListError::UnclosedClips {
                remaining: self.clip_depth,
            });
        }
        Ok(UiDrawList {
            revision: self.revision,
            diagnostics: self.diagnostics,
            entries: self.entries,
        })
    }

    fn push(&mut self, source: Option<crate::UiNodeId>, layer: u32, command: UiDrawCommand) {
        if let Some(previous) = self.last_layer {
            if layer < previous {
                self.error.get_or_insert(UiDrawListError::LayerOrder {
                    previous,
                    next: layer,
                });
            }
        }
        self.entries.push(UiDrawEntry {
            source,
            layer,
            order: self.next_order,
            command,
        });
        self.next_order = self.next_order.saturating_add(1);
        self.last_layer = Some(layer);
    }
}

/// Lowers a resolved semantic tree into one renderer-neutral draw artifact.
///
/// The tree remains the source of bounds, clipping, visibility, and ordering.
/// This function intentionally applies only semantic surface roles and text
/// intent; stateful button treatment, icons, and general vector content remain
/// separate lowering concerns until their contracts are equally resolved.
pub fn lower_resolved_tree_to_draw_list(
    tree: &UiResolvedTree,
    theme: &UiTheme,
    revision: u64,
) -> UiDrawList {
    let mut builder = UiDrawListBuilder::new(revision);
    for diagnostic in &tree.diagnostics {
        let kind = match diagnostic.kind {
            UiTreeDiagnosticKind::TextOverflow => Some(UiDrawListDiagnosticKind::TextOverflow),
            UiTreeDiagnosticKind::TextProviderUnavailable => {
                Some(UiDrawListDiagnosticKind::TextProviderUnavailable)
            }
            UiTreeDiagnosticKind::MissingGlyph { character } => {
                Some(UiDrawListDiagnosticKind::MissingGlyph { character })
            }
            _ => None,
        };
        if let Some(kind) = kind {
            builder.record_diagnostic(UiDrawListDiagnostic {
                source: Some(diagnostic.node),
                kind,
            });
        }
    }
    let mut next_execution_layer = 0;
    lower_resolved_node(&tree.root, theme, &mut builder, &mut next_execution_layer);
    // Tree resolution guarantees pre-order layer assignment and balanced
    // recursive clipping, so draw-list validation should only fail if this
    // lowering implementation regresses.
    builder
        .finish()
        .expect("resolved UI trees lower into a valid ordered draw list")
}

fn lower_resolved_node(
    node: &UiResolvedNode,
    theme: &UiTheme,
    builder: &mut UiDrawListBuilder,
    next_execution_layer: &mut u32,
) {
    if !node.visible {
        return;
    }

    if !matches!(node.kind, UiNodeKind::Text(_)) {
        builder.surface(
            Some(node.provenance),
            take_execution_layer(next_execution_layer),
            UiSurfaceCommand {
                rect: node.bounds,
                style: theme.surface(node.role),
                clip: node.clip,
            },
        );
    }
    if let Some(spec) = &node.text {
        builder.text(
            Some(node.provenance),
            take_execution_layer(next_execution_layer),
            UiTextCommand::new(spec.clone(), theme.text(spec.role)),
        );
    }

    if node.clips_children {
        builder.push_clip(
            Some(node.provenance),
            take_execution_layer(next_execution_layer),
            node.clip.unwrap_or(node.bounds),
        );
    }
    for child in &node.children {
        lower_resolved_node(child, theme, builder, next_execution_layer);
    }
    if node.clips_children {
        builder.pop_clip(
            Some(node.provenance),
            take_execution_layer(next_execution_layer),
        );
    }
}

fn take_execution_layer(next_execution_layer: &mut u32) -> u32 {
    let layer = *next_execution_layer;
    *next_execution_layer = next_execution_layer.saturating_add(1);
    layer
}

pub struct UiDrawer<'a> {
    pub surfaces: &'a mut Vec<UiSurfaceCommand>,
    pub text: &'a mut Vec<UiTextCommand>,
    pub theme: &'a UiTheme,
    clip: Option<UiRect>,
}

impl<'a> UiDrawer<'a> {
    pub fn new(
        surfaces: &'a mut Vec<UiSurfaceCommand>,
        text: &'a mut Vec<UiTextCommand>,
        theme: &'a UiTheme,
    ) -> Self {
        Self {
            surfaces,
            text,
            theme,
            clip: None,
        }
    }

    pub fn set_clip(&mut self, clip: Option<UiRect>) {
        self.clip = clip;
    }

    fn clipped_rect(&self, rect: UiRect) -> Option<UiRect> {
        self.clip.map_or(Some(rect), |clip| rect.intersection(clip))
    }

    pub fn surface(&mut self, region: &UiRegion) {
        if self.clipped_rect(region.rect).is_some() {
            self.surfaces.push(UiSurfaceCommand {
                rect: region.rect,
                style: self.theme.surface(region.role),
                clip: self.clip,
            });
        }
    }

    pub fn label(&mut self, label: &UiLabel, role: UiTextRole) {
        let spec = UiTextSpec::new(
            label.text,
            UiRect::new([label.position[0], label.position[1]], [0.0, 0.0]),
            role,
        )
        .with_alignment(label.anchor.into(), UiTextAlign::Center)
        .with_overflow(UiTextOverflow::Clip);
        if let Some(rect) = self.clipped_rect(spec.rect) {
            self.text.push(UiTextCommand {
                spec: UiTextSpec { rect, ..spec },
                style: self.theme.text(role),
            });
        }
    }

    pub fn chip(&mut self, chip: &UiStateChip, role: UiTextRole) {
        self.surface(&chip.region());
        let spec = UiTextSpec::new(chip.label, chip.rect, role);
        if let Some(rect) = self.clipped_rect(spec.rect) {
            self.text.push(UiTextCommand {
                spec: UiTextSpec { rect, ..spec },
                style: self.theme.text(role),
            });
        }
    }

    pub fn button(&mut self, button: &UiButton, state: UiInteractionState, role: UiControlRole) {
        if let Some(rect) = self.clipped_rect(button.rect) {
            self.surfaces.push(UiSurfaceCommand {
                rect: button.rect,
                style: self.theme.control(role, state),
                clip: self.clip,
            });
            let spec = UiTextSpec::new(button.label, rect, UiTextRole::Button);
            self.text.push(UiTextCommand {
                spec,
                style: self.theme.text(UiTextRole::Button),
            });
        }
    }

    pub fn card(&mut self, card: &UiCard) {
        if self.clipped_rect(card.region.rect).is_some() {
            self.surfaces.push(UiSurfaceCommand {
                rect: card.region.rect,
                style: self.theme.card(card.role),
                clip: self.clip,
            });
        }
        self.surface(&card.header);
        self.surface(&card.body_region);
        self.surface(&card.footer);

        let title = UiTextSpec::new(
            card.title,
            // Keep the visual header band narrow, but give glyphs enough
            // vertical bounds to avoid clipping bitmap rows.
            UiRect::new(
                card.header.rect.center,
                [
                    card.header.rect.size[0] - card.padding.value() * 2.0,
                    card.region.rect.size[1] * 0.34,
                ],
            ),
            UiTextRole::Heading,
        );
        if let Some(rect) = self.clipped_rect(title.rect) {
            self.text.push(UiTextCommand {
                spec: UiTextSpec { rect, ..title },
                style: self.theme.text(UiTextRole::Heading),
            });
        }

        let body = UiTextSpec::new(
            card.body,
            UiRect::new(
                card.body_region.rect.center,
                [
                    card.body_region.rect.size[0] - card.padding.value() * 2.0,
                    card.region.rect.size[1] * 0.34,
                ],
            ),
            UiTextRole::Body,
        )
        .with_alignment(UiTextAlign::Start, UiTextAlign::Center);
        if let Some(rect) = self.clipped_rect(body.rect) {
            self.text.push(UiTextCommand {
                spec: UiTextSpec { rect, ..body },
                style: self.theme.text(UiTextRole::Body),
            });
        }
    }

    pub fn workspace(&mut self, region: &UiRegion) {
        self.surface(region);
    }

    pub fn toolbar(&mut self, region: &UiRegion) {
        self.surface(region);
    }

    pub fn divider(&mut self, region: &UiRegion) {
        self.surface(region);
    }

    pub fn button_strip(
        &mut self,
        button: &UiButton,
        state: UiInteractionState,
        role: UiControlRole,
    ) {
        self.button(button, state, role);
    }
}
