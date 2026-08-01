use crate::{
    UiButton, UiButtonId, UiCard, UiCardRole, UiInsets, UiLayoutFit, UiRect, UiRegion,
    UiRegionKind, UiSurfaceRole, UiTextDiagnostic, UiTextDiagnosticKind, UiTextFit, UiTextMeasure,
    UiTextMetricsProvider, UiTextRole, UiTextSpec,
};

/// Provider-neutral identity for a semantic UI node.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct UiNodeId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNodeKind {
    Region(UiRegionKind),
    Card(UiCardRole),
    Button(UiButtonId),
    Text(UiTextRole),
    /// A semantic editing region. Platform IME integration and the mutable
    /// text value remain with the consuming application.
    TextInput,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiNodeContent {
    None,
    Text(String),
}

/// Provider-neutral meaning exposed to accessibility and inspection adapters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiSemanticRole {
    Region,
    Group,
    Button,
    Text,
    TextInput,
}

/// Focus-aware semantic observation of one resolved node.
///
/// Platform accessibility trees may adapt this record, but platform roles,
/// handles, and event mechanisms do not belong in `ui-tools`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiResolvedSemantics {
    pub id: UiNodeId,
    pub role: UiSemanticRole,
    pub label: Option<String>,
    pub value: Option<String>,
    pub visible: bool,
    pub enabled: bool,
    pub selected: bool,
    pub focusable: bool,
    pub focused: bool,
}

/// Declares whether a semantic node participates in pointer target resolution.
///
/// This deliberately identifies an interaction capability rather than a
/// widget implementation. A button opts in today, while future controls can
/// expose the same target contract without pretending to be buttons.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UiNodeInteraction {
    #[default]
    Passive,
    Activatable,
    /// Accepts normalized text-edit operations while focused.
    Editable,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiNodeLayout {
    Explicit(UiRect),
    Fill,
    Inset(UiInsets),
    /// Centers the node in its parent at its preferred constrained size.
    ///
    /// This is deliberately distinct from `Fill`: consumers can express a
    /// natural size without weakening the meaning of a fill allocation.
    Fit,
}

/// Semantic stacking class used by drawing, hit testing, and focus traversal.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum UiNodeStacking {
    #[default]
    Normal,
    Overlay,
    Modal,
}

/// Application-owned reason for requesting dismissal of the active modal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiModalDismissReason {
    Escape,
    Backdrop,
}

/// Provider-neutral dismissal request. The application decides whether and
/// how its state changes in response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiModalDismissal {
    pub modal: UiNodeId,
    pub reason: UiModalDismissReason,
}

/// Provider-neutral readable-size request for a semantic node.
///
/// Resolution never silently stretches an ancestor to satisfy this request. If
/// a node receives less space than requested, its resolved layout reports
/// `UiLayoutFit::Overflow` and emits a bounded diagnostic for the node. Maximum
/// constraints clamp the node and report `UiLayoutFit::Adjusted` rather than
/// allowing oversized geometry to escape its semantic allocation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiNodeConstraints {
    pub min_size: [f32; 2],
    pub preferred_size: Option<[f32; 2]>,
    pub max_size: [f32; 2],
}

impl UiNodeConstraints {
    pub const fn minimum(min_size: [f32; 2]) -> Self {
        Self {
            min_size,
            preferred_size: None,
            max_size: [f32::INFINITY, f32::INFINITY],
        }
    }

    pub const fn preferred(preferred_size: [f32; 2]) -> Self {
        Self {
            min_size: [0.0, 0.0],
            preferred_size: Some(preferred_size),
            max_size: [f32::INFINITY, f32::INFINITY],
        }
    }

    pub const fn bounded(min_size: [f32; 2], preferred_size: [f32; 2], max_size: [f32; 2]) -> Self {
        Self {
            min_size,
            preferred_size: Some(preferred_size),
            max_size,
        }
    }
}

impl Default for UiNodeConstraints {
    fn default() -> Self {
        Self {
            min_size: [0.0, 0.0],
            preferred_size: None,
            max_size: [f32::INFINITY, f32::INFINITY],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiNodeSpec {
    pub id: UiNodeId,
    pub parent: Option<UiNodeId>,
    pub kind: UiNodeKind,
    pub role: UiSurfaceRole,
    pub content: UiNodeContent,
    pub semantic_label: Option<String>,
    pub semantic_value: Option<String>,
    pub selected: bool,
    /// Optional text intent owned by this node. Its rectangle is resolved from
    /// `layout`, so consumers cannot accidentally keep a parallel text box.
    pub text: Option<UiTextSpec>,
    pub layout: UiNodeLayout,
    pub constraints: UiNodeConstraints,
    pub interaction: UiNodeInteraction,
    pub visible: bool,
    pub enabled: bool,
    pub clips_children: bool,
    pub stacking: UiNodeStacking,
    /// Whether the application admits Escape/backdrop dismissal for this
    /// modal. Ignored for non-modal stacking classes.
    pub dismissible: bool,
    /// Translation inherited by descendants while this node remains fixed.
    ///
    /// This is the provider-neutral seam used by scrolling: the viewport owns
    /// clipping, while its content moves through normal tree resolution.
    pub child_translation: [f32; 2],
    pub children: Vec<Self>,
}

impl UiNodeSpec {
    pub fn region(id: UiNodeId, region: UiRegion) -> Self {
        Self::new(
            id,
            UiNodeKind::Region(region.kind),
            region.role,
            UiNodeLayout::Explicit(region.rect),
        )
    }

    pub fn card(id: UiNodeId, card: UiCard) -> Self {
        Self::new(
            id,
            UiNodeKind::Card(card.role),
            card.surface_role,
            UiNodeLayout::Explicit(card.region.rect),
        )
        .with_content(UiNodeContent::Text(card.title.to_owned()))
    }

    pub fn button(id: UiNodeId, button: UiButton) -> Self {
        Self::new(
            id,
            UiNodeKind::Button(button.id),
            UiSurfaceRole::Raised,
            UiNodeLayout::Explicit(button.rect),
        )
        .with_content(UiNodeContent::Text(button.label.to_owned()))
        .with_interaction(UiNodeInteraction::Activatable)
        .with_enabled(button.enabled)
    }

    pub fn text(id: UiNodeId, text: &UiTextSpec) -> Self {
        Self::new(
            id,
            UiNodeKind::Text(text.role),
            UiSurfaceRole::Region,
            UiNodeLayout::Explicit(text.rect),
        )
        .with_content(UiNodeContent::Text(text.text.clone()))
        .with_text(text.clone())
    }

    pub fn new(id: UiNodeId, kind: UiNodeKind, role: UiSurfaceRole, layout: UiNodeLayout) -> Self {
        let interaction = if matches!(kind, UiNodeKind::Button(_)) {
            UiNodeInteraction::Activatable
        } else {
            UiNodeInteraction::Passive
        };
        Self {
            id,
            parent: None,
            kind,
            role,
            content: UiNodeContent::None,
            semantic_label: None,
            semantic_value: None,
            selected: false,
            text: None,
            layout,
            constraints: UiNodeConstraints::default(),
            interaction,
            visible: true,
            enabled: true,
            clips_children: false,
            stacking: UiNodeStacking::Normal,
            dismissible: false,
            child_translation: [0.0, 0.0],
            children: Vec::new(),
        }
    }

    pub fn with_parent(mut self, parent: UiNodeId) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn with_content(mut self, content: UiNodeContent) -> Self {
        self.content = content;
        self
    }

    pub fn with_semantic_label(mut self, label: impl Into<String>) -> Self {
        self.semantic_label = Some(label.into());
        self
    }

    pub fn with_semantic_value(mut self, value: impl Into<String>) -> Self {
        self.semantic_value = Some(value.into());
        self
    }

    pub const fn with_selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Attaches provider-neutral text behavior to this node. Resolution updates
    /// the text rectangle from the final node bounds.
    pub fn with_text(mut self, text: UiTextSpec) -> Self {
        self.text = Some(text);
        self
    }

    /// Replaces the node's spatial intent while preserving its semantic kind
    /// and provider-neutral content.
    pub fn with_layout(mut self, layout: UiNodeLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Declares the smallest readable region this node can accept.
    pub fn with_constraints(mut self, constraints: UiNodeConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    /// Opts this node into the shared pointer-target contract.
    pub fn with_interaction(mut self, interaction: UiNodeInteraction) -> Self {
        self.interaction = interaction;
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_child(mut self, child: Self) -> Self {
        self.children.push(child);
        self
    }

    pub fn with_children(mut self, children: impl IntoIterator<Item = Self>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn clips_children(mut self) -> Self {
        self.clips_children = true;
        self
    }

    /// Translates descendant content without moving this node's own bounds.
    pub fn with_child_translation(mut self, translation: [f32; 2]) -> Self {
        self.child_translation = translation;
        self
    }

    pub fn as_overlay(mut self) -> Self {
        self.stacking = UiNodeStacking::Overlay;
        self
    }

    pub fn as_modal(mut self, dismissible: bool) -> Self {
        self.stacking = UiNodeStacking::Modal;
        self.dismissible = dismissible;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiFitStatus {
    Fits,
    Clipped,
    Empty,
}

/// Bounded evidence emitted while a semantic tree becomes resolved geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct UiTreeDiagnostic {
    pub node: UiNodeId,
    pub kind: UiTreeDiagnosticKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiTreeDiagnosticKind {
    Clipped,
    Empty,
    Hidden,
    BelowMinimumSize,
    ImpossibleLayout,
    TextOverflow,
    TextProviderUnavailable,
    MissingGlyph { character: char },
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiResolvedNode {
    pub id: UiNodeId,
    pub kind: UiNodeKind,
    pub role: UiSurfaceRole,
    pub content: UiNodeContent,
    pub semantic_label: Option<String>,
    pub semantic_value: Option<String>,
    pub selected: bool,
    /// Text intent with the final resolved bounds, when this node owns text.
    pub text: Option<UiTextSpec>,
    /// Provider-neutral measurement when this tree was resolved with a text
    /// metrics provider. Plain headless resolution deliberately leaves this
    /// empty rather than selecting an implicit font implementation.
    pub text_measure: Option<UiTextMeasure>,
    /// Pre-overflow-policy fit result for `text_measure` in `bounds`.
    pub text_fit: Option<UiTextFit>,
    /// The semantic source node which produced this resolved node.
    pub provenance: UiNodeId,
    pub bounds: UiRect,
    /// Whether this node's requested readable geometry fit its final bounds.
    pub layout_fit: UiLayoutFit,
    pub clip: Option<UiRect>,
    pub visible: bool,
    pub enabled: bool,
    pub interaction: UiNodeInteraction,
    pub stacking: UiNodeStacking,
    pub dismissible: bool,
    /// Whether this node establishes a clipping scope for its descendants.
    pub clips_children: bool,
    pub layer: usize,
    pub fit: UiFitStatus,
    pub children: Vec<Self>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiResolvedTree {
    pub viewport: UiRect,
    pub root: UiResolvedNode,
    pub diagnostics: Vec<UiTreeDiagnostic>,
}

impl UiResolvedTree {
    /// Returns any resolved node by semantic identity.
    ///
    /// This is a read-only composition seam for consumers that need to anchor
    /// domain-specific presentation inside shared layout regions. It exposes
    /// final provider-neutral geometry, not renderer resources or mutable UI
    /// state.
    pub fn node(&self, id: UiNodeId) -> Option<&UiResolvedNode> {
        find_node(&self.root, id)
    }

    /// Returns the topmost admitted interactive node at `point`.
    ///
    /// Hit testing intentionally consumes resolved bounds and inherited clips,
    /// so an interaction adapter cannot drift from the geometry used by
    /// presentation lowering. More control kinds can opt in as their semantic
    /// interaction contracts are admitted.
    pub fn hit_test(&self, point: [f32; 2]) -> Option<&UiResolvedNode> {
        hit_test_node(self.interaction_root(), point)
    }

    /// Returns an admitted interactive node by identity.
    ///
    /// Pointer capture uses this lookup to cancel safely when a newly resolved
    /// tree removes or disables the control that previously received a press.
    pub fn interactive_node(&self, id: UiNodeId) -> Option<&UiResolvedNode> {
        find_node(self.interaction_root(), id).filter(|node| node.is_interactive())
    }

    /// Returns an admitted editable node by identity.
    pub fn editable_node(&self, id: UiNodeId) -> Option<&UiResolvedNode> {
        find_node(self.interaction_root(), id).filter(|node| node.is_editable())
    }

    /// Returns interactive nodes in the resolved visual traversal order.
    pub fn interactive_node_ids(&self) -> Vec<UiNodeId> {
        let mut ids = Vec::new();
        collect_interactive_node_ids(self.interaction_root(), &mut ids);
        ids
    }

    /// Returns the topmost modal scope, if any. Later modal siblings and nested
    /// modal scopes take precedence consistently with visual traversal.
    pub fn active_modal(&self) -> Option<&UiResolvedNode> {
        find_topmost_modal(&self.root)
    }

    /// Resolves a dismissal request without changing application state.
    pub fn modal_dismissal(&self, reason: UiModalDismissReason) -> Option<UiModalDismissal> {
        self.active_modal()
            .filter(|modal| modal.dismissible)
            .map(|modal| UiModalDismissal {
                modal: modal.id,
                reason,
            })
    }

    /// Returns inspectable semantics in resolved visual order.
    ///
    /// When a modal is active, background semantics are excluded consistently
    /// with pointer and focus routing.
    pub fn semantic_nodes(&self, focus: &UiResolvedFocus) -> Vec<UiResolvedSemantics> {
        let mut semantics = Vec::new();
        collect_semantics(self.interaction_root(), focus.focused(), &mut semantics);
        semantics
    }

    fn interaction_root(&self) -> &UiResolvedNode {
        self.active_modal().unwrap_or(&self.root)
    }
}

impl UiResolvedNode {
    pub const fn semantic_role(&self) -> UiSemanticRole {
        match self.kind {
            UiNodeKind::Region(_) => UiSemanticRole::Region,
            UiNodeKind::Card(_) => UiSemanticRole::Group,
            UiNodeKind::Button(_) => UiSemanticRole::Button,
            UiNodeKind::Text(_) => UiSemanticRole::Text,
            UiNodeKind::TextInput => UiSemanticRole::TextInput,
        }
    }

    /// Whether the current semantic kind has an admitted activation contract.
    pub fn is_interactive(&self) -> bool {
        !matches!(self.interaction, UiNodeInteraction::Passive) && self.visible && self.enabled
    }

    /// Whether this node can receive normalized text-edit operations.
    pub fn is_editable(&self) -> bool {
        matches!(self.interaction, UiNodeInteraction::Editable) && self.visible && self.enabled
    }
}

/// The renderer-neutral phase of a pointer interaction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPointerPhase {
    Move,
    Press,
    Release,
}

/// A pointer event expressed in the same coordinate space as `UiResolvedTree`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiPointerEvent {
    pub position: [f32; 2],
    pub phase: UiPointerPhase,
}

impl UiPointerEvent {
    pub const fn new(position: [f32; 2], phase: UiPointerPhase) -> Self {
        Self { position, phase }
    }
}

/// The resolved target state after routing one pointer event.
///
/// This intentionally reports identities rather than invoking callbacks. The
/// consuming application owns commands, state mutation, and presentation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiPointerResolution {
    pub hover: Option<UiNodeId>,
    pub target: Option<UiNodeId>,
    pub captured: Option<UiNodeId>,
    pub activated: Option<UiNodeId>,
}

/// Deterministic pointer target routing over a resolved semantic UI tree.
///
/// Press captures the current hit target. Move and release continue to target
/// that capture even when the pointer leaves its bounds. Release activates only
/// when the pointer returns to the captured target. A re-resolved tree can
/// cancel capture by removing or disabling the captured node.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UiPointerRouter {
    hover: Option<UiNodeId>,
    captured: Option<UiNodeId>,
}

/// A normalized edit request targeting the currently focused editable node.
///
/// This carries semantic editing intent only. Text state, IME composition, and
/// application command handling remain outside the resolved UI tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiTextInputEvent {
    pub operation: crate::UiTextInputOperation,
}

impl UiTextInputEvent {
    pub const fn new(operation: crate::UiTextInputOperation) -> Self {
        Self { operation }
    }
}

/// The resolved text-edit target for one normalized input operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiTextInputResolution {
    pub target: Option<UiNodeId>,
    pub operation: crate::UiTextInputOperation,
}

/// Routes normalized text editing through the same resolved focus contract used
/// by pointer and keyboard interaction.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UiTextInputRouter;

/// Focus identity and traversal over a resolved semantic tree.
///
/// This is intentionally independent of legacy button-specific focus helpers.
/// It resolves only a focus target; keyboard event normalization and semantic
/// command dispatch remain application responsibilities.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UiResolvedFocus {
    focused: Option<UiNodeId>,
}

impl UiResolvedFocus {
    pub const fn focused(&self) -> Option<UiNodeId> {
        self.focused
    }

    /// Selects an interactive node, clearing focus when the identity is absent,
    /// hidden, clipped, or disabled in the supplied resolved tree.
    pub fn set_focus(&mut self, tree: &UiResolvedTree, candidate: Option<UiNodeId>) {
        self.focused = candidate.filter(|id| tree.interactive_node(*id).is_some());
    }

    /// Reconciles a retained focus identity with a newly resolved tree.
    pub fn reconcile(&mut self, tree: &UiResolvedTree) -> Option<UiNodeId> {
        self.set_focus(tree, self.focused);
        self.focused
    }

    /// Moves focus through admitted interactive nodes in stable pre-order.
    pub fn move_focus(
        &mut self,
        tree: &UiResolvedTree,
        direction: crate::UiFocusDirection,
    ) -> Option<UiNodeId> {
        let focusable = tree.interactive_node_ids();
        if focusable.is_empty() {
            self.focused = None;
            return None;
        }

        let current_index = self
            .focused
            .and_then(|id| focusable.iter().position(|candidate| *candidate == id));
        let next_index = match (current_index, direction) {
            (Some(index), crate::UiFocusDirection::Forward) => (index + 1) % focusable.len(),
            (Some(index), crate::UiFocusDirection::Backward) => {
                (index + focusable.len() - 1) % focusable.len()
            }
            (None, crate::UiFocusDirection::Forward) => 0,
            (None, crate::UiFocusDirection::Backward) => focusable.len() - 1,
        };
        self.focused = Some(focusable[next_index]);
        self.focused
    }

    /// Resolves a normalized activation key to the currently focused node.
    ///
    /// The caller owns platform key normalization and the command associated
    /// with the returned node identity.
    pub fn activate(
        &mut self,
        tree: &UiResolvedTree,
        key: crate::UiActivationKey,
    ) -> Option<UiNodeId> {
        self.reconcile(tree);
        matches!(
            key,
            crate::UiActivationKey::Enter | crate::UiActivationKey::Space
        )
        .then(|| {
            self.focused.filter(|id| {
                tree.interactive_node(*id)
                    .is_some_and(|node| matches!(node.interaction, UiNodeInteraction::Activatable))
            })
        })
        .flatten()
    }
}

impl UiPointerRouter {
    pub const fn hover(&self) -> Option<UiNodeId> {
        self.hover
    }

    pub const fn captured(&self) -> Option<UiNodeId> {
        self.captured
    }

    /// Resolves the presentation state for one node from shared interaction
    /// state. Application selection remains an explicit semantic input.
    ///
    /// The precedence is intentional: a disabled node cannot present as
    /// interactive, an active capture presents as pressed, and transient
    /// pointer feedback wins over retained focus or selection.
    pub fn interaction_state(
        &self,
        tree: &UiResolvedTree,
        focus: &UiResolvedFocus,
        node_id: UiNodeId,
        selected: bool,
    ) -> crate::UiInteractionState {
        let Some(node) = find_node(&tree.root, node_id) else {
            return crate::UiInteractionState::Disabled;
        };
        if !node.visible || !node.enabled {
            return crate::UiInteractionState::Disabled;
        }
        if self.captured == Some(node_id) {
            return crate::UiInteractionState::Pressed;
        }
        if self.hover == Some(node_id) {
            return crate::UiInteractionState::Hovered;
        }
        if focus.focused() == Some(node_id) {
            return crate::UiInteractionState::Focused;
        }
        if selected {
            return crate::UiInteractionState::Selected;
        }
        crate::UiInteractionState::Idle
    }

    pub fn route(&mut self, tree: &UiResolvedTree, event: UiPointerEvent) -> UiPointerResolution {
        if self
            .captured
            .is_some_and(|id| tree.interactive_node(id).is_none())
        {
            self.captured = None;
        }

        let hit = tree.hit_test(event.position).map(|node| node.id);
        self.hover = hit;

        let (target, activated) = match event.phase {
            UiPointerPhase::Move => (self.captured.or(hit), None),
            UiPointerPhase::Press => {
                self.captured = hit;
                (hit, None)
            }
            UiPointerPhase::Release => {
                let target = self.captured.take();
                let activated = target.filter(|id| Some(*id) == hit);
                (target, activated)
            }
        };

        UiPointerResolution {
            hover: self.hover,
            target,
            captured: self.captured,
            activated,
        }
    }
}

impl UiTextInputRouter {
    /// Returns the focused editable node for the normalized operation, or no
    /// target after focus loss, removal, disablement, or a non-editable focus.
    pub fn route(
        &self,
        tree: &UiResolvedTree,
        focus: &mut UiResolvedFocus,
        event: UiTextInputEvent,
    ) -> UiTextInputResolution {
        focus.reconcile(tree);
        UiTextInputResolution {
            target: focus
                .focused()
                .filter(|id| tree.editable_node(*id).is_some()),
            operation: event.operation,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiTreeError {
    DuplicateId(UiNodeId),
    InvalidParent {
        node: UiNodeId,
        expected: Option<UiNodeId>,
        actual: Option<UiNodeId>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiTree {
    pub root: UiNodeSpec,
}

impl UiTree {
    pub const fn new(root: UiNodeSpec) -> Self {
        Self { root }
    }

    /// Resolves a semantic tree without a window, provider, or renderer.
    pub fn resolve(&self, viewport: UiRect) -> Result<UiResolvedTree, UiTreeError> {
        self.resolve_with_optional_text_metrics(viewport, None)
    }

    /// Resolves the semantic tree and attaches provider-neutral text evidence.
    ///
    /// The provider supplies measurements only; it does not control semantic
    /// roles, resolved geometry, renderer commands, or fallback policy.
    pub fn resolve_with_text_metrics(
        &self,
        viewport: UiRect,
        text_metrics: &dyn UiTextMetricsProvider,
    ) -> Result<UiResolvedTree, UiTreeError> {
        self.resolve_with_optional_text_metrics(viewport, Some(text_metrics))
    }

    fn resolve_with_optional_text_metrics(
        &self,
        viewport: UiRect,
        text_metrics: Option<&dyn UiTextMetricsProvider>,
    ) -> Result<UiResolvedTree, UiTreeError> {
        let mut ids = Vec::new();
        let mut next_layer = 0;
        let mut diagnostics = Vec::new();
        let mut context = UiResolveContext {
            parent_bounds: viewport,
            parent_clip: Some(viewport),
            parent_visible: true,
            explicit_translation: [0.0, 0.0],
            ids: &mut ids,
            next_layer: &mut next_layer,
            diagnostics: &mut diagnostics,
            text_metrics,
        };
        let root = resolve_node(&self.root, None, &mut context)?;

        Ok(UiResolvedTree {
            viewport,
            root,
            diagnostics,
        })
    }
}

struct UiResolveContext<'a> {
    parent_bounds: UiRect,
    parent_clip: Option<UiRect>,
    parent_visible: bool,
    /// Accumulated translation for explicitly positioned descendant content.
    explicit_translation: [f32; 2],
    ids: &'a mut Vec<UiNodeId>,
    next_layer: &'a mut usize,
    diagnostics: &'a mut Vec<UiTreeDiagnostic>,
    text_metrics: Option<&'a dyn UiTextMetricsProvider>,
}

fn resolve_node(
    spec: &UiNodeSpec,
    expected_parent: Option<UiNodeId>,
    context: &mut UiResolveContext<'_>,
) -> Result<UiResolvedNode, UiTreeError> {
    if spec.parent != expected_parent {
        return Err(UiTreeError::InvalidParent {
            node: spec.id,
            expected: expected_parent,
            actual: spec.parent,
        });
    }
    if context.ids.contains(&spec.id) {
        return Err(UiTreeError::DuplicateId(spec.id));
    }
    context.ids.push(spec.id);

    let (bounds, size_adjusted, capacity_overflow) = resolve_node_bounds(
        spec.layout,
        context.parent_bounds,
        context.explicit_translation,
        spec.constraints,
    );
    let layout_fit = layout_fit(bounds, spec.constraints, size_adjusted, capacity_overflow);
    match layout_fit {
        UiLayoutFit::Overflow => {
            push_diagnostic(
                context.diagnostics,
                spec.id,
                UiTreeDiagnosticKind::BelowMinimumSize,
            );
        }
        UiLayoutFit::Impossible => {
            push_diagnostic(
                context.diagnostics,
                spec.id,
                UiTreeDiagnosticKind::ImpossibleLayout,
            );
        }
        UiLayoutFit::Exact | UiLayoutFit::Adjusted => {}
    }
    let parent_clip = context.parent_clip;
    let clip = parent_clip.and_then(|clip| bounds.intersection(clip));
    let fit = match (parent_clip, clip) {
        (Some(parent_clip), Some(_)) if rect_contains_with_tolerance(parent_clip, bounds) => {
            UiFitStatus::Fits
        }
        (_, None) => UiFitStatus::Empty,
        (_, Some(_)) => UiFitStatus::Clipped,
    };
    let visible = context.parent_visible && spec.visible && !matches!(fit, UiFitStatus::Empty);
    let diagnostic_kind = match (fit, visible) {
        (UiFitStatus::Clipped, true) => Some(UiTreeDiagnosticKind::Clipped),
        (UiFitStatus::Empty, _) => Some(UiTreeDiagnosticKind::Empty),
        (_, false) => Some(UiTreeDiagnosticKind::Hidden),
        _ => None,
    };
    if let Some(kind) = diagnostic_kind {
        push_diagnostic(context.diagnostics, spec.id, kind);
    }
    let layer = *context.next_layer;
    *context.next_layer += 1;

    let child_clip = if spec.clips_children {
        clip
    } else {
        context.parent_clip
    };
    let mut child_context = UiResolveContext {
        parent_bounds: bounds,
        parent_clip: child_clip,
        parent_visible: visible,
        explicit_translation: [
            context.explicit_translation[0] + spec.child_translation[0],
            context.explicit_translation[1] + spec.child_translation[1],
        ],
        ids: context.ids,
        next_layer: context.next_layer,
        diagnostics: context.diagnostics,
        text_metrics: context.text_metrics,
    };
    let mut children = Vec::with_capacity(spec.children.len());
    for stacking in [
        UiNodeStacking::Normal,
        UiNodeStacking::Overlay,
        UiNodeStacking::Modal,
    ] {
        for child in spec
            .children
            .iter()
            .filter(|child| child.stacking == stacking)
        {
            children.push(resolve_node(child, Some(spec.id), &mut child_context)?);
        }
    }

    let text = spec.text.clone().map(|text| UiTextSpec {
        rect: bounds,
        ..text
    });
    let (text_measure, text_fit) = resolve_text_measurement(
        spec.id,
        text.as_ref(),
        context.text_metrics,
        context.diagnostics,
    );

    Ok(UiResolvedNode {
        id: spec.id,
        kind: spec.kind,
        role: spec.role,
        content: spec.content.clone(),
        semantic_label: spec.semantic_label.clone().or_else(|| match &spec.content {
            UiNodeContent::None => None,
            UiNodeContent::Text(text) => Some(text.clone()),
        }),
        semantic_value: spec.semantic_value.clone(),
        selected: spec.selected,
        text,
        text_measure,
        text_fit,
        provenance: spec.id,
        bounds,
        layout_fit,
        clip,
        visible,
        enabled: spec.enabled,
        interaction: spec.interaction,
        stacking: spec.stacking,
        dismissible: spec.dismissible,
        clips_children: spec.clips_children,
        layer,
        fit,
        children,
    })
}

fn rect_contains_with_tolerance(container: UiRect, candidate: UiRect) -> bool {
    let scale = container
        .center
        .iter()
        .chain(container.size.iter())
        .chain(candidate.center.iter())
        .chain(candidate.size.iter())
        .fold(1.0_f32, |scale, value| scale.max(value.abs()));
    let tolerance = scale * 1.0e-5;
    let container_left = container.center[0] - container.size[0] * 0.5;
    let container_right = container.center[0] + container.size[0] * 0.5;
    let container_bottom = container.center[1] - container.size[1] * 0.5;
    let container_top = container.center[1] + container.size[1] * 0.5;
    let candidate_left = candidate.center[0] - candidate.size[0] * 0.5;
    let candidate_right = candidate.center[0] + candidate.size[0] * 0.5;
    let candidate_bottom = candidate.center[1] - candidate.size[1] * 0.5;
    let candidate_top = candidate.center[1] + candidate.size[1] * 0.5;

    candidate_left >= container_left - tolerance
        && candidate_right <= container_right + tolerance
        && candidate_bottom >= container_bottom - tolerance
        && candidate_top <= container_top + tolerance
}

fn collect_semantics(
    node: &UiResolvedNode,
    focused: Option<UiNodeId>,
    output: &mut Vec<UiResolvedSemantics>,
) {
    output.push(UiResolvedSemantics {
        id: node.id,
        role: node.semantic_role(),
        label: node.semantic_label.clone(),
        value: node.semantic_value.clone(),
        visible: node.visible,
        enabled: node.enabled,
        selected: node.selected,
        focusable: node.is_interactive(),
        focused: focused == Some(node.id),
    });
    for child in &node.children {
        collect_semantics(child, focused, output);
    }
}

fn resolve_text_measurement(
    node: UiNodeId,
    text: Option<&UiTextSpec>,
    text_metrics: Option<&dyn UiTextMetricsProvider>,
    diagnostics: &mut Vec<UiTreeDiagnostic>,
) -> (Option<UiTextMeasure>, Option<UiTextFit>) {
    let (Some(text), Some(provider)) = (text, text_metrics) else {
        return (None, None);
    };

    match provider.measure(&text.text) {
        Ok(measure) => {
            let fit = measure.fit_in(text.rect);
            if !fit.fits() {
                push_diagnostic(diagnostics, node, UiTreeDiagnosticKind::TextOverflow);
            }
            for diagnostic in &measure.diagnostics {
                push_text_diagnostic(diagnostics, node, diagnostic);
            }
            (Some(measure), Some(fit))
        }
        Err(diagnostic) => {
            push_text_diagnostic(diagnostics, node, &diagnostic);
            (None, None)
        }
    }
}

fn push_text_diagnostic(
    diagnostics: &mut Vec<UiTreeDiagnostic>,
    node: UiNodeId,
    diagnostic: &UiTextDiagnostic,
) {
    let kind = match diagnostic.kind {
        UiTextDiagnosticKind::MissingGlyph { character } => {
            UiTreeDiagnosticKind::MissingGlyph { character }
        }
        UiTextDiagnosticKind::ProviderUnavailable => UiTreeDiagnosticKind::TextProviderUnavailable,
    };
    push_diagnostic(diagnostics, node, kind);
}

fn layout_fit(
    bounds: UiRect,
    constraints: UiNodeConstraints,
    size_adjusted: bool,
    capacity_overflow: bool,
) -> UiLayoutFit {
    if !bounds
        .center
        .iter()
        .chain(bounds.size.iter())
        .all(|value| value.is_finite())
        || bounds.size[0] <= 0.0
        || bounds.size[1] <= 0.0
    {
        return UiLayoutFit::Impossible;
    }

    let minimum = normalized_constraints(constraints).min_size;
    if capacity_overflow || bounds.size[0] < minimum[0] || bounds.size[1] < minimum[1] {
        UiLayoutFit::Overflow
    } else if size_adjusted {
        UiLayoutFit::Adjusted
    } else {
        UiLayoutFit::Exact
    }
}

fn normalized_constraints(constraints: UiNodeConstraints) -> UiNodeConstraints {
    let min_size = constraints.min_size.map(normalize_minimum);
    let max_size = [
        normalize_maximum(constraints.max_size[0], min_size[0]),
        normalize_maximum(constraints.max_size[1], min_size[1]),
    ];
    let preferred_size = constraints.preferred_size.map(|preferred| {
        [
            normalize_preferred(preferred[0], min_size[0], max_size[0]),
            normalize_preferred(preferred[1], min_size[1], max_size[1]),
        ]
    });

    UiNodeConstraints {
        min_size,
        preferred_size,
        max_size,
    }
}

fn resolve_node_bounds(
    layout: UiNodeLayout,
    parent_bounds: UiRect,
    explicit_translation: [f32; 2],
    constraints: UiNodeConstraints,
) -> (UiRect, bool, bool) {
    let constraints = normalized_constraints(constraints);
    if matches!(layout, UiNodeLayout::Fit) {
        let preferred = constraints.preferred_size.unwrap_or(parent_bounds.size);
        let capacity_overflow = parent_bounds.size[0] < constraints.min_size[0]
            || parent_bounds.size[1] < constraints.min_size[1];
        let size = [
            if parent_bounds.size[0] < constraints.min_size[0] {
                constraints.min_size[0]
            } else {
                preferred[0].min(parent_bounds.size[0])
            },
            if parent_bounds.size[1] < constraints.min_size[1] {
                constraints.min_size[1]
            } else {
                preferred[1].min(parent_bounds.size[1])
            },
        ];
        return (
            UiRect::new(parent_bounds.center, size),
            size != preferred,
            capacity_overflow,
        );
    }

    let bounds = match layout {
        UiNodeLayout::Explicit(bounds) => bounds.translated(explicit_translation),
        UiNodeLayout::Fill => parent_bounds,
        UiNodeLayout::Inset(insets) => parent_bounds.inset_by(insets),
        UiNodeLayout::Fit => unreachable!("fit layout returned above"),
    };
    if !bounds.size.iter().all(|value| value.is_finite()) {
        return (bounds, false, false);
    }
    let size = [
        bounds.size[0].min(constraints.max_size[0]),
        bounds.size[1].min(constraints.max_size[1]),
    ];
    let adjusted = size != bounds.size;
    (UiRect::new(bounds.center, size), adjusted, false)
}

fn normalize_minimum(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn normalize_maximum(value: f32, minimum: f32) -> f32 {
    if value.is_nan() {
        f32::INFINITY
    } else {
        value.max(minimum)
    }
}

fn normalize_preferred(value: f32, minimum: f32, maximum: f32) -> f32 {
    if value.is_finite() {
        value.clamp(minimum, maximum)
    } else {
        minimum
    }
}

fn hit_test_node(node: &UiResolvedNode, point: [f32; 2]) -> Option<&UiResolvedNode> {
    // Later siblings own higher pre-order layers, so visit them first.
    for child in node.children.iter().rev() {
        if let Some(hit) = hit_test_node(child, point) {
            return Some(hit);
        }
    }

    let within_clip = node.clip.is_none_or(|clip| clip.contains(point));
    if node.is_interactive() && within_clip && node.bounds.contains(point) {
        Some(node)
    } else {
        None
    }
}

fn find_node(node: &UiResolvedNode, id: UiNodeId) -> Option<&UiResolvedNode> {
    if node.id == id {
        return Some(node);
    }

    node.children.iter().find_map(|child| find_node(child, id))
}

fn find_topmost_modal(node: &UiResolvedNode) -> Option<&UiResolvedNode> {
    node.children
        .iter()
        .rev()
        .find_map(find_topmost_modal)
        .or_else(|| {
            (node.visible && matches!(node.stacking, UiNodeStacking::Modal)).then_some(node)
        })
}

fn collect_interactive_node_ids(node: &UiResolvedNode, ids: &mut Vec<UiNodeId>) {
    if node.is_interactive() {
        ids.push(node.id);
    }
    for child in &node.children {
        collect_interactive_node_ids(child, ids);
    }
}

fn push_diagnostic(
    diagnostics: &mut Vec<UiTreeDiagnostic>,
    node: UiNodeId,
    kind: UiTreeDiagnosticKind,
) {
    const MAX_TREE_DIAGNOSTICS: usize = 128;
    if diagnostics.len() < MAX_TREE_DIAGNOSTICS {
        diagnostics.push(UiTreeDiagnostic { node, kind });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct TestTextMetrics {
        result: Result<UiTextMeasure, UiTextDiagnostic>,
    }

    impl UiTextMetricsProvider for TestTextMetrics {
        fn measure(&self, _text: &str) -> Result<UiTextMeasure, UiTextDiagnostic> {
            self.result.clone()
        }
    }

    fn root(children: Vec<UiNodeSpec>) -> UiTree {
        UiTree::new(
            UiNodeSpec::new(
                UiNodeId(1),
                UiNodeKind::Region(UiRegionKind::Workspace),
                UiSurfaceRole::Region,
                UiNodeLayout::Fill,
            )
            .clips_children()
            .with_children(children),
        )
    }

    #[test]
    fn equivalent_semantic_trees_resolve_identically() {
        let child = UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Inset(UiInsets::uniform(0.1)),
        )
        .with_parent(UiNodeId(1));
        let viewport = UiRect::new([0.0, 0.0], [2.0, 1.0]);

        assert_eq!(
            root(vec![child.clone()]).resolve(viewport),
            root(vec![child]).resolve(viewport)
        );
    }

    #[test]
    fn resolution_updates_text_bounds_without_losing_text_policy() {
        let root_id = UiNodeId(1);
        let text = UiTextSpec::new(
            "a deliberately long status label",
            UiRect::new([99.0, 99.0], [99.0, 99.0]),
            UiTextRole::Status,
        )
        .with_overflow(crate::UiTextOverflow::Ellipsis)
        .with_alignment(crate::UiTextAlign::End, crate::UiTextAlign::Center);
        let child = UiNodeSpec::text(UiNodeId(2), &text)
            .with_parent(root_id)
            .with_layout(UiNodeLayout::Fill)
            .with_content(UiNodeContent::Text(text.text.clone()));
        let viewport = UiRect::new([0.0, 0.0], [2.0, 1.0]);

        let resolved = root(vec![child]).resolve(viewport).unwrap();
        let text = resolved.root.children[0].text.as_ref().unwrap();

        assert_eq!(text.rect, viewport);
        assert_eq!(text.overflow, crate::UiTextOverflow::Ellipsis);
        assert_eq!(text.align_x, crate::UiTextAlign::End);
    }

    #[test]
    fn measured_resolution_attaches_fit_and_missing_glyph_evidence_to_its_text_node() {
        let text = UiTextSpec::new(
            "MISSING",
            UiRect::new([0.0, 0.0], [0.1, 0.08]),
            UiTextRole::Status,
        );
        let child = UiNodeSpec::text(UiNodeId(2), &text)
            .with_parent(UiNodeId(1))
            .with_layout(UiNodeLayout::Fill);
        let metrics = TestTextMetrics {
            result: Ok(UiTextMeasure {
                advance: 0.4,
                ascent: 0.05,
                descent: 0.01,
                line_gap: 0.012,
                visible_bounds: Some(UiRect::new([0.03, 0.02], [0.32, 0.06])),
                diagnostics: vec![UiTextDiagnostic {
                    kind: UiTextDiagnosticKind::MissingGlyph { character: 'X' },
                }],
            }),
        };

        let resolved = root(vec![child])
            .resolve_with_text_metrics(UiRect::new([0.0, 0.0], [0.1, 0.08]), &metrics)
            .unwrap();
        let child = &resolved.root.children[0];

        assert_eq!(
            child.text_fit,
            Some(UiTextFit {
                horizontal_overflow: true,
                vertical_overflow: false,
            })
        );
        assert_eq!(
            child.text_measure.as_ref(),
            Some(&UiTextMeasure {
                advance: 0.4,
                ascent: 0.05,
                descent: 0.01,
                line_gap: 0.012,
                visible_bounds: Some(UiRect::new([0.03, 0.02], [0.32, 0.06])),
                diagnostics: vec![UiTextDiagnostic {
                    kind: UiTextDiagnosticKind::MissingGlyph { character: 'X' },
                }],
            })
        );
        assert!(resolved.diagnostics.contains(&UiTreeDiagnostic {
            node: UiNodeId(2),
            kind: UiTreeDiagnosticKind::TextOverflow,
        }));
        assert!(resolved.diagnostics.contains(&UiTreeDiagnostic {
            node: UiNodeId(2),
            kind: UiTreeDiagnosticKind::MissingGlyph { character: 'X' },
        }));
    }

    #[test]
    fn measured_resolution_reports_provider_failure_without_implicit_fallback() {
        let text = UiTextSpec::new(
            "STATUS",
            UiRect::new([0.0, 0.0], [0.2, 0.08]),
            UiTextRole::Status,
        );
        let child = UiNodeSpec::text(UiNodeId(2), &text)
            .with_parent(UiNodeId(1))
            .with_layout(UiNodeLayout::Fill);
        let metrics = TestTextMetrics {
            result: Err(UiTextDiagnostic {
                kind: UiTextDiagnosticKind::ProviderUnavailable,
            }),
        };

        let resolved = root(vec![child])
            .resolve_with_text_metrics(UiRect::new([0.0, 0.0], [0.2, 0.08]), &metrics)
            .unwrap();
        let child = &resolved.root.children[0];

        assert!(child.text_measure.is_none());
        assert!(child.text_fit.is_none());
        assert!(resolved.diagnostics.contains(&UiTreeDiagnostic {
            node: UiNodeId(2),
            kind: UiTreeDiagnosticKind::TextProviderUnavailable,
        }));
    }

    #[test]
    fn resolution_reports_an_explicit_minimum_size_violation() {
        let child = UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Fill,
        )
        .with_parent(UiNodeId(1))
        .with_constraints(UiNodeConstraints::minimum([3.0, 2.0]));
        let resolved = root(vec![child])
            .resolve(UiRect::new([0.0, 0.0], [2.0, 1.0]))
            .unwrap();

        assert_eq!(resolved.root.children[0].layout_fit, UiLayoutFit::Overflow);
        assert!(resolved.diagnostics.contains(&UiTreeDiagnostic {
            node: UiNodeId(2),
            kind: UiTreeDiagnosticKind::BelowMinimumSize,
        }));
    }

    #[test]
    fn fit_layout_uses_centered_preferred_size_without_weakening_fill() {
        let fit = UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Fit,
        )
        .with_parent(UiNodeId(1))
        .with_constraints(UiNodeConstraints::preferred([1.2, 0.6]));
        let fill = UiNodeSpec::new(
            UiNodeId(3),
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Fill,
        )
        .with_parent(UiNodeId(1))
        .with_constraints(UiNodeConstraints::preferred([1.2, 0.6]));

        let resolved = root(vec![fit, fill])
            .resolve(UiRect::new([0.25, -0.5], [4.0, 2.0]))
            .unwrap();

        assert_eq!(
            resolved.root.children[0].bounds,
            UiRect::new([0.25, -0.5], [1.2, 0.6])
        );
        assert_eq!(resolved.root.children[0].layout_fit, UiLayoutFit::Exact);
        assert_eq!(resolved.root.children[1].bounds, resolved.viewport);
    }

    #[test]
    fn maximum_size_clamps_geometry_and_reports_adjustment() {
        let child = UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Fill,
        )
        .with_parent(UiNodeId(1))
        .with_constraints(UiNodeConstraints::bounded(
            [0.5, 0.25],
            [1.0, 0.5],
            [2.0, 1.0],
        ));

        let resolved = root(vec![child])
            .resolve(UiRect::new([0.0, 0.0], [4.0, 3.0]))
            .unwrap();
        let child = &resolved.root.children[0];

        assert_eq!(child.bounds, UiRect::new([0.0, 0.0], [2.0, 1.0]));
        assert_eq!(child.layout_fit, UiLayoutFit::Adjusted);
    }

    #[test]
    fn fit_layout_preserves_minimum_and_reports_insufficient_capacity() {
        let child = UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Fit,
        )
        .with_parent(UiNodeId(1))
        .with_constraints(UiNodeConstraints::bounded(
            [1.5, 1.0],
            [2.0, 1.5],
            [3.0, 2.0],
        ));

        let resolved = root(vec![child])
            .resolve(UiRect::new([0.0, 0.0], [1.0, 0.75]))
            .unwrap();
        let child = &resolved.root.children[0];

        assert_eq!(child.bounds.size, [1.5, 1.0]);
        assert_eq!(child.layout_fit, UiLayoutFit::Overflow);
        assert!(resolved.diagnostics.contains(&UiTreeDiagnostic {
            node: UiNodeId(2),
            kind: UiTreeDiagnosticKind::BelowMinimumSize,
        }));
    }

    #[test]
    fn resolution_reports_impossible_geometry_without_hiding_the_cause() {
        let child = UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Explicit(UiRect::new([0.0, 0.0], [0.0, 1.0])),
        )
        .with_parent(UiNodeId(1));
        let resolved = root(vec![child])
            .resolve(UiRect::new([0.0, 0.0], [2.0, 1.0]))
            .unwrap();

        assert_eq!(
            resolved.root.children[0].layout_fit,
            UiLayoutFit::Impossible
        );
        assert!(resolved.diagnostics.contains(&UiTreeDiagnostic {
            node: UiNodeId(2),
            kind: UiTreeDiagnosticKind::ImpossibleLayout,
        }));
    }

    #[test]
    fn hit_testing_uses_resolved_order_enabled_state_and_clipping() {
        let lower_button = UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Button(UiButtonId(2)),
            UiSurfaceRole::Raised,
            UiNodeLayout::Explicit(UiRect::new([0.0, 0.0], [1.0, 1.0])),
        )
        .with_parent(UiNodeId(1));
        let disabled_top_button = UiNodeSpec::new(
            UiNodeId(3),
            UiNodeKind::Button(UiButtonId(3)),
            UiSurfaceRole::Raised,
            UiNodeLayout::Explicit(UiRect::new([0.0, 0.0], [1.0, 1.0])),
        )
        .with_parent(UiNodeId(1))
        .with_enabled(false);
        let clipped_button = UiNodeSpec::new(
            UiNodeId(4),
            UiNodeKind::Button(UiButtonId(4)),
            UiSurfaceRole::Raised,
            UiNodeLayout::Explicit(UiRect::new([2.0, 0.0], [1.0, 1.0])),
        )
        .with_parent(UiNodeId(1));
        let resolved = root(vec![lower_button, disabled_top_button, clipped_button])
            .resolve(UiRect::new([0.0, 0.0], [1.0, 1.0]))
            .unwrap();

        assert_eq!(
            resolved.hit_test([0.0, 0.0]).map(|node| node.id),
            Some(UiNodeId(2))
        );
        assert_eq!(resolved.hit_test([2.0, 0.0]), None);
    }

    #[test]
    fn non_button_nodes_can_opt_into_the_shared_interaction_contract() {
        let interactive_card = UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Card(UiCardRole::Preview),
            UiSurfaceRole::Card,
            UiNodeLayout::Explicit(UiRect::new([0.0, 0.0], [0.8, 0.8])),
        )
        .with_parent(UiNodeId(1))
        .with_interaction(UiNodeInteraction::Activatable);
        let resolved = root(vec![interactive_card])
            .resolve(UiRect::new([0.0, 0.0], [1.0, 1.0]))
            .unwrap();

        let hit = resolved.hit_test([0.0, 0.0]).unwrap();
        assert_eq!(hit.id, UiNodeId(2));
        assert_eq!(hit.interaction, UiNodeInteraction::Activatable);
        assert!(hit.is_interactive());
    }

    #[test]
    fn pointer_capture_targets_the_press_node_until_release() {
        let button = UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Button(UiButtonId(2)),
            UiSurfaceRole::Raised,
            UiNodeLayout::Explicit(UiRect::new([0.0, 0.0], [0.5, 0.5])),
        )
        .with_parent(UiNodeId(1));
        let tree = root(vec![button])
            .resolve(UiRect::new([0.0, 0.0], [2.0, 1.0]))
            .unwrap();
        let mut router = UiPointerRouter::default();

        let press = router.route(
            &tree,
            UiPointerEvent::new([0.0, 0.0], UiPointerPhase::Press),
        );
        assert_eq!(press.target, Some(UiNodeId(2)));
        assert_eq!(press.captured, Some(UiNodeId(2)));

        let moved = router.route(&tree, UiPointerEvent::new([0.9, 0.0], UiPointerPhase::Move));
        assert_eq!(moved.hover, None);
        assert_eq!(moved.target, Some(UiNodeId(2)));
        assert_eq!(moved.captured, Some(UiNodeId(2)));

        let released = router.route(
            &tree,
            UiPointerEvent::new([0.9, 0.0], UiPointerPhase::Release),
        );
        assert_eq!(released.target, Some(UiNodeId(2)));
        assert_eq!(released.captured, None);
        assert_eq!(released.activated, None);
    }

    #[test]
    fn pointer_release_activates_only_the_matching_capture_target() {
        let button = UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Button(UiButtonId(2)),
            UiSurfaceRole::Raised,
            UiNodeLayout::Explicit(UiRect::new([0.0, 0.0], [0.5, 0.5])),
        )
        .with_parent(UiNodeId(1));
        let tree = root(vec![button])
            .resolve(UiRect::new([0.0, 0.0], [2.0, 1.0]))
            .unwrap();
        let mut router = UiPointerRouter::default();

        router.route(
            &tree,
            UiPointerEvent::new([0.0, 0.0], UiPointerPhase::Press),
        );
        let released = router.route(
            &tree,
            UiPointerEvent::new([0.0, 0.0], UiPointerPhase::Release),
        );

        assert_eq!(released.activated, Some(UiNodeId(2)));
        assert_eq!(router.captured(), None);
    }

    #[test]
    fn pointer_capture_cancels_when_a_resolved_tree_disables_the_node() {
        let active_button = UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Button(UiButtonId(2)),
            UiSurfaceRole::Raised,
            UiNodeLayout::Explicit(UiRect::new([0.0, 0.0], [0.5, 0.5])),
        )
        .with_parent(UiNodeId(1));
        let disabled_button = active_button.clone().with_enabled(false);
        let viewport = UiRect::new([0.0, 0.0], [2.0, 1.0]);
        let active_tree = root(vec![active_button]).resolve(viewport).unwrap();
        let disabled_tree = root(vec![disabled_button]).resolve(viewport).unwrap();
        let mut router = UiPointerRouter::default();

        router.route(
            &active_tree,
            UiPointerEvent::new([0.0, 0.0], UiPointerPhase::Press),
        );
        let released = router.route(
            &disabled_tree,
            UiPointerEvent::new([0.0, 0.0], UiPointerPhase::Release),
        );

        assert_eq!(released.target, None);
        assert_eq!(released.activated, None);
        assert_eq!(router.captured(), None);
    }

    #[test]
    fn resolved_focus_traverses_interactive_nodes_in_stable_tree_order() {
        let first = UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Button(UiButtonId(2)),
            UiSurfaceRole::Raised,
            UiNodeLayout::Explicit(UiRect::new([-0.3, 0.0], [0.2, 0.2])),
        )
        .with_parent(UiNodeId(1));
        let second = UiNodeSpec::new(
            UiNodeId(3),
            UiNodeKind::Region(UiRegionKind::Card),
            UiSurfaceRole::Raised,
            UiNodeLayout::Explicit(UiRect::new([0.0, 0.0], [0.2, 0.2])),
        )
        .with_parent(UiNodeId(1))
        .with_interaction(UiNodeInteraction::Activatable);
        let disabled = UiNodeSpec::new(
            UiNodeId(4),
            UiNodeKind::Button(UiButtonId(4)),
            UiSurfaceRole::Raised,
            UiNodeLayout::Explicit(UiRect::new([0.3, 0.0], [0.2, 0.2])),
        )
        .with_parent(UiNodeId(1))
        .with_enabled(false);
        let tree = root(vec![first, second, disabled])
            .resolve(UiRect::new([0.0, 0.0], [1.0, 1.0]))
            .unwrap();
        let mut focus = UiResolvedFocus::default();

        assert_eq!(tree.interactive_node_ids(), vec![UiNodeId(2), UiNodeId(3)]);
        assert_eq!(
            focus.move_focus(&tree, crate::UiFocusDirection::Forward),
            Some(UiNodeId(2))
        );
        assert_eq!(
            focus.move_focus(&tree, crate::UiFocusDirection::Forward),
            Some(UiNodeId(3))
        );
        assert_eq!(
            focus.move_focus(&tree, crate::UiFocusDirection::Backward),
            Some(UiNodeId(2))
        );
    }

    #[test]
    fn resolved_focus_clears_when_the_focused_node_disappears() {
        let button = UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Button(UiButtonId(2)),
            UiSurfaceRole::Raised,
            UiNodeLayout::Explicit(UiRect::new([0.0, 0.0], [0.5, 0.5])),
        )
        .with_parent(UiNodeId(1));
        let viewport = UiRect::new([0.0, 0.0], [1.0, 1.0]);
        let active_tree = root(vec![button]).resolve(viewport).unwrap();
        let removed_tree = root(vec![]).resolve(viewport).unwrap();
        let mut focus = UiResolvedFocus::default();

        focus.set_focus(&active_tree, Some(UiNodeId(2)));
        assert_eq!(focus.focused(), Some(UiNodeId(2)));
        assert_eq!(focus.reconcile(&removed_tree), None);
        assert_eq!(focus.focused(), None);
    }

    #[test]
    fn resolved_focus_activates_only_admitted_keys_for_the_current_node() {
        let button = UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Button(UiButtonId(2)),
            UiSurfaceRole::Raised,
            UiNodeLayout::Explicit(UiRect::new([0.0, 0.0], [0.5, 0.5])),
        )
        .with_parent(UiNodeId(1));
        let tree = root(vec![button])
            .resolve(UiRect::new([0.0, 0.0], [1.0, 1.0]))
            .unwrap();
        let mut focus = UiResolvedFocus::default();

        focus.set_focus(&tree, Some(UiNodeId(2)));
        assert_eq!(
            focus.activate(&tree, crate::UiActivationKey::Enter),
            Some(UiNodeId(2))
        );
        assert_eq!(
            focus.activate(&tree, crate::UiActivationKey::Space),
            Some(UiNodeId(2))
        );
    }

    #[test]
    fn text_input_router_targets_only_the_focused_editable_node() {
        let editable = UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::TextInput,
            UiSurfaceRole::Raised,
            UiNodeLayout::Explicit(UiRect::new([0.0, 0.0], [0.5, 0.5])),
        )
        .with_parent(UiNodeId(1))
        .with_interaction(UiNodeInteraction::Editable);
        let button = UiNodeSpec::new(
            UiNodeId(3),
            UiNodeKind::Button(UiButtonId(3)),
            UiSurfaceRole::Raised,
            UiNodeLayout::Explicit(UiRect::new([0.3, 0.0], [0.2, 0.2])),
        )
        .with_parent(UiNodeId(1));
        let tree = root(vec![editable, button])
            .resolve(UiRect::new([0.0, 0.0], [1.0, 1.0]))
            .unwrap();
        let mut focus = UiResolvedFocus::default();
        let router = UiTextInputRouter;
        let event = UiTextInputEvent::new(crate::UiTextInputOperation::Insert('7'));

        focus.set_focus(&tree, Some(UiNodeId(2)));
        assert_eq!(
            router.route(&tree, &mut focus, event),
            UiTextInputResolution {
                target: Some(UiNodeId(2)),
                operation: crate::UiTextInputOperation::Insert('7'),
            }
        );

        focus.set_focus(&tree, Some(UiNodeId(3)));
        assert_eq!(router.route(&tree, &mut focus, event).target, None);
        assert_eq!(
            focus.activate(&tree, crate::UiActivationKey::Enter),
            Some(UiNodeId(3))
        );
    }

    #[test]
    fn text_input_router_clears_a_stale_editable_focus() {
        let editable = UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::TextInput,
            UiSurfaceRole::Raised,
            UiNodeLayout::Explicit(UiRect::new([0.0, 0.0], [0.5, 0.5])),
        )
        .with_parent(UiNodeId(1))
        .with_interaction(UiNodeInteraction::Editable);
        let viewport = UiRect::new([0.0, 0.0], [1.0, 1.0]);
        let active = root(vec![editable]).resolve(viewport).unwrap();
        let removed = root(vec![]).resolve(viewport).unwrap();
        let mut focus = UiResolvedFocus::default();
        let router = UiTextInputRouter;

        focus.set_focus(&active, Some(UiNodeId(2)));
        assert_eq!(
            router
                .route(
                    &removed,
                    &mut focus,
                    UiTextInputEvent::new(crate::UiTextInputOperation::DeleteBackward),
                )
                .target,
            None
        );
        assert_eq!(focus.focused(), None);
    }

    #[test]
    fn resolved_interaction_state_uses_shared_precedence() {
        let enabled = UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Button(UiButtonId(2)),
            UiSurfaceRole::Panel,
            UiNodeLayout::Explicit(UiRect::new([0.0, 0.0], [0.5, 1.0])),
        )
        .with_parent(UiNodeId(1));
        let disabled = UiNodeSpec::new(
            UiNodeId(3),
            UiNodeKind::Button(UiButtonId(3)),
            UiSurfaceRole::Panel,
            UiNodeLayout::Explicit(UiRect::new([0.5, 0.0], [1.0, 1.0])),
        )
        .with_parent(UiNodeId(1))
        .with_enabled(false);
        let tree = root(vec![enabled, disabled])
            .resolve(UiRect::new([0.0, 0.0], [1.0, 1.0]))
            .expect("tree resolves");

        let mut pointer = UiPointerRouter::default();
        let mut focus = UiResolvedFocus::default();
        focus.set_focus(&tree, Some(UiNodeId(2)));

        assert_eq!(
            pointer.interaction_state(&tree, &focus, UiNodeId(2), true),
            crate::UiInteractionState::Focused
        );
        pointer.route(
            &tree,
            UiPointerEvent::new([0.25, 0.5], UiPointerPhase::Move),
        );
        assert_eq!(
            pointer.interaction_state(&tree, &focus, UiNodeId(2), true),
            crate::UiInteractionState::Hovered
        );
        pointer.route(
            &tree,
            UiPointerEvent::new([0.25, 0.5], UiPointerPhase::Press),
        );
        assert_eq!(
            pointer.interaction_state(&tree, &focus, UiNodeId(2), true),
            crate::UiInteractionState::Pressed
        );
        assert_eq!(
            pointer.interaction_state(&tree, &focus, UiNodeId(3), false),
            crate::UiInteractionState::Disabled
        );
    }

    #[test]
    fn resolution_preserves_child_and_layer_order() {
        let tree = root(vec![
            UiNodeSpec::new(
                UiNodeId(2),
                UiNodeKind::Text(UiTextRole::Body),
                UiSurfaceRole::Region,
                UiNodeLayout::Fill,
            )
            .with_parent(UiNodeId(1)),
            UiNodeSpec::new(
                UiNodeId(3),
                UiNodeKind::Text(UiTextRole::Status),
                UiSurfaceRole::Overlay,
                UiNodeLayout::Fill,
            )
            .with_parent(UiNodeId(1)),
        ]);

        let resolved = tree.resolve(UiRect::new([0.0, 0.0], [2.0, 1.0])).unwrap();
        assert_eq!(resolved.root.layer, 0);
        assert_eq!(resolved.root.children[0].id, UiNodeId(2));
        assert_eq!(resolved.root.children[0].layer, 1);
        assert_eq!(resolved.root.children[1].id, UiNodeId(3));
        assert_eq!(resolved.root.children[1].layer, 2);
    }

    #[test]
    fn equivalent_rebuilds_preserve_resolved_identity_and_structure() {
        let build = || {
            root(vec![
                UiNodeSpec::new(
                    UiNodeId(2),
                    UiNodeKind::Region(UiRegionKind::Panel),
                    UiSurfaceRole::Panel,
                    UiNodeLayout::Fill,
                )
                .with_parent(UiNodeId(1))
                .with_semantic_label("stable panel"),
                UiNodeSpec::new(
                    UiNodeId(3),
                    UiNodeKind::Text(UiTextRole::Body),
                    UiSurfaceRole::Region,
                    UiNodeLayout::Fill,
                )
                .with_parent(UiNodeId(1))
                .with_content(UiNodeContent::Text("stable content".to_owned())),
            ])
        };
        let viewport = UiRect::new([0.0, 0.0], [2.0, 1.0]);

        let first = build().resolve(viewport).unwrap();
        let second = build().resolve(viewport).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.root.provenance, UiNodeId(1));
        assert_eq!(first.root.children[0].provenance, UiNodeId(2));
        assert_eq!(first.root.children[1].provenance, UiNodeId(3));
    }

    #[test]
    fn duplicate_identity_is_rejected() {
        let tree = root(vec![
            UiNodeSpec::new(
                UiNodeId(2),
                UiNodeKind::Region(UiRegionKind::Panel),
                UiSurfaceRole::Panel,
                UiNodeLayout::Fill,
            )
            .with_parent(UiNodeId(1)),
            UiNodeSpec::new(
                UiNodeId(2),
                UiNodeKind::Region(UiRegionKind::Card),
                UiSurfaceRole::Card,
                UiNodeLayout::Fill,
            )
            .with_parent(UiNodeId(1)),
        ]);

        assert_eq!(
            tree.resolve(UiRect::new([0.0, 0.0], [2.0, 1.0])),
            Err(UiTreeError::DuplicateId(UiNodeId(2)))
        );
    }

    #[test]
    fn invalid_parentage_is_rejected_explicitly() {
        let tree = root(vec![UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Fill,
        )
        .with_parent(UiNodeId(99))]);

        assert_eq!(
            tree.resolve(UiRect::new([0.0, 0.0], [2.0, 1.0])),
            Err(UiTreeError::InvalidParent {
                node: UiNodeId(2),
                expected: Some(UiNodeId(1)),
                actual: Some(UiNodeId(99)),
            })
        );
    }

    #[test]
    fn clipping_and_visibility_are_headless_resolution_results() {
        let tree = root(vec![UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Region(UiRegionKind::Card),
            UiSurfaceRole::Card,
            UiNodeLayout::Explicit(UiRect::new([0.9, 0.0], [0.6, 0.4])),
        )
        .with_parent(UiNodeId(1))]);

        let resolved = tree.resolve(UiRect::new([0.0, 0.0], [2.0, 1.0])).unwrap();
        let child = &resolved.root.children[0];
        assert_eq!(child.fit, UiFitStatus::Clipped);
        assert!(child.visible);
        let clip = child.clip.unwrap();
        assert!((clip.center[0] - 0.8).abs() < 0.00001);
        assert!((clip.size[0] - 0.4).abs() < 0.00001);
        assert!((clip.size[1] - 0.4).abs() < 0.00001);
        assert_eq!(
            resolved.diagnostics,
            vec![UiTreeDiagnostic {
                node: UiNodeId(2),
                kind: UiTreeDiagnosticKind::Clipped,
            }]
        );
    }

    #[test]
    fn contained_decimal_layout_does_not_emit_false_clipping() {
        let viewport = UiRect::new([0.0, 0.0], [12.8, 7.2]);
        let child_bounds = UiRect::new([0.0, 2.94], [12.2, 0.576]);
        let tree = root(vec![UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Panel,
            UiNodeLayout::Explicit(child_bounds),
        )
        .with_parent(UiNodeId(1))]);

        let resolved = tree.resolve(viewport).unwrap();

        assert_eq!(resolved.root.children[0].fit, UiFitStatus::Fits);
        assert!(resolved.diagnostics.is_empty());
    }

    #[test]
    fn disabled_button_remains_visible_but_not_enabled() {
        let button = UiButton::new(UiButtonId(7), "Save", UiRect::new([0.0, 0.0], [0.4, 0.1]))
            .with_enabled(false);
        let tree = root(vec![
            UiNodeSpec::button(UiNodeId(2), button).with_parent(UiNodeId(1))
        ]);

        let resolved = tree.resolve(UiRect::new([0.0, 0.0], [2.0, 1.0])).unwrap();
        let button = &resolved.root.children[0];
        assert!(button.visible);
        assert!(!button.enabled);
    }

    #[test]
    fn overlay_stacking_is_deterministic_independent_of_declaration_order() {
        let overlay = UiNodeSpec::new(
            UiNodeId(3),
            UiNodeKind::Card(UiCardRole::Preview),
            UiSurfaceRole::Overlay,
            UiNodeLayout::Explicit(UiRect::new([0.0, 0.0], [0.6, 0.6])),
        )
        .with_parent(UiNodeId(1))
        .with_interaction(UiNodeInteraction::Activatable)
        .as_overlay();
        let background = UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Button(UiButtonId(2)),
            UiSurfaceRole::Raised,
            UiNodeLayout::Explicit(UiRect::new([0.0, 0.0], [0.8, 0.8])),
        )
        .with_parent(UiNodeId(1));

        let resolved = root(vec![overlay, background])
            .resolve(UiRect::new([0.0, 0.0], [1.0, 1.0]))
            .unwrap();

        assert_eq!(resolved.root.children[0].id, UiNodeId(2));
        assert_eq!(resolved.root.children[1].id, UiNodeId(3));
        assert_eq!(
            resolved.hit_test([0.0, 0.0]).map(|node| node.id),
            Some(UiNodeId(3))
        );
    }

    #[test]
    fn active_modal_confines_pointer_focus_and_dismissal() {
        let background = UiNodeSpec::new(
            UiNodeId(2),
            UiNodeKind::Button(UiButtonId(2)),
            UiSurfaceRole::Raised,
            UiNodeLayout::Explicit(UiRect::new([-0.35, 0.0], [0.2, 0.2])),
        )
        .with_parent(UiNodeId(1));
        let modal_id = UiNodeId(3);
        let modal_button_id = UiNodeId(4);
        let modal = UiNodeSpec::new(
            modal_id,
            UiNodeKind::Region(UiRegionKind::Panel),
            UiSurfaceRole::Overlay,
            UiNodeLayout::Explicit(UiRect::new([0.25, 0.0], [0.5, 0.6])),
        )
        .with_parent(UiNodeId(1))
        .as_modal(true)
        .with_child(
            UiNodeSpec::new(
                modal_button_id,
                UiNodeKind::Button(UiButtonId(4)),
                UiSurfaceRole::Raised,
                UiNodeLayout::Explicit(UiRect::new([0.25, 0.0], [0.2, 0.2])),
            )
            .with_parent(modal_id),
        );
        let resolved = root(vec![modal, background])
            .resolve(UiRect::new([0.0, 0.0], [1.0, 1.0]))
            .unwrap();

        assert_eq!(resolved.active_modal().map(|node| node.id), Some(modal_id));
        assert_eq!(resolved.hit_test([-0.35, 0.0]), None);
        assert_eq!(
            resolved.hit_test([0.25, 0.0]).map(|node| node.id),
            Some(modal_button_id)
        );
        assert_eq!(resolved.interactive_node_ids(), vec![modal_button_id]);

        let mut focus = UiResolvedFocus::default();
        focus.set_focus(&resolved, Some(UiNodeId(2)));
        assert_eq!(focus.focused(), None);
        assert_eq!(
            focus.move_focus(&resolved, crate::UiFocusDirection::Forward),
            Some(modal_button_id)
        );
        assert_eq!(
            resolved.modal_dismissal(UiModalDismissReason::Escape),
            Some(UiModalDismissal {
                modal: modal_id,
                reason: UiModalDismissReason::Escape,
            })
        );
    }

    #[test]
    fn resolver_diagnostics_are_bounded() {
        let children = (2..200)
            .map(|id| {
                UiNodeSpec::new(
                    UiNodeId(id),
                    UiNodeKind::Region(UiRegionKind::Card),
                    UiSurfaceRole::Card,
                    UiNodeLayout::Explicit(UiRect::new([10.0, 10.0], [0.1, 0.1])),
                )
                .with_parent(UiNodeId(1))
            })
            .collect::<Vec<_>>();
        let resolved = root(children)
            .resolve(UiRect::new([0.0, 0.0], [2.0, 1.0]))
            .unwrap();

        assert_eq!(resolved.diagnostics.len(), 128);
        assert!(resolved
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.kind == UiTreeDiagnosticKind::Empty));
    }
}
