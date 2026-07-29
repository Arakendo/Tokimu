#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UiIconProviderId(pub String);

impl UiIconProviderId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

/// Opaque identity for a resolved icon resource.
///
/// A generation prevents a released provider slot from silently becoming a
/// different icon. Providers own allocation and invalidation of these values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiIconHandle {
    pub index: u32,
    pub generation: u32,
}

impl UiIconHandle {
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UiIconId {
    pub provider: UiIconProviderId,
    pub name: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiIconMetrics {
    pub size: [f32; 2],
    pub baseline: f32,
}

/// Provider output before renderer-specific tessellation.
///
/// Paths remain simple vector geometry here; SVG parsing, flattening, stroke
/// joins, and mesh generation belong to the provider or its vector backend.
#[derive(Clone, Debug, PartialEq)]
pub struct UiIconVectorAsset {
    pub paths: Vec<Vec<[f32; 2]>>,
    pub view_box: [f32; 4],
}

/// Contract implemented by Lucide, project-local, or future icon providers.
pub trait UiIconVectorProvider {
    fn provider(&self) -> &UiIconProviderId;
    fn resolve_vector(&self, id: &UiIconId) -> Result<UiIconVectorAsset, UiIconDiagnostic>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiIconTint {
    Inherit,
    Accent,
    Muted,
    Disabled,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiIconSpec {
    pub id: UiIconId,
    pub rect: UiRect,
    pub tint: UiIconTint,
}

impl UiIconSpec {
    pub fn new(id: UiIconId, rect: UiRect) -> Self {
        Self {
            id,
            rect,
            tint: UiIconTint::Inherit,
        }
    }

    pub fn with_tint(mut self, tint: UiIconTint) -> Self {
        self.tint = tint;
        self
    }

    pub fn metrics(&self) -> UiIconMetrics {
        UiIconMetrics {
            size: self.rect.size,
            baseline: self.rect.center[1],
        }
    }
}

impl UiIconId {
    pub fn new(provider: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            provider: UiIconProviderId::new(provider),
            name: name.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiIconDiagnosticKind {
    MissingIcon,
    ProviderUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiIconDiagnostic {
    pub icon: UiIconId,
    pub kind: UiIconDiagnosticKind,
}

impl UiIconDiagnostic {
    pub fn missing(icon: UiIconId) -> Self {
        Self {
            icon,
            kind: UiIconDiagnosticKind::MissingIcon,
        }
    }

    pub fn provider_unavailable(icon: UiIconId) -> Self {
        Self {
            icon,
            kind: UiIconDiagnosticKind::ProviderUnavailable,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiIconResolution {
    Resolved(UiIconId),
    Fallback {
        requested: UiIconId,
        fallback: UiIconId,
    },
    Missing {
        icon: UiIconId,
        diagnostic: UiIconDiagnostic,
    },
}

impl UiIconResolution {
    pub fn requested(&self) -> &UiIconId {
        match self {
            Self::Resolved(icon) => icon,
            Self::Fallback { requested, .. } => requested,
            Self::Missing { icon, .. } => icon,
        }
    }

    pub fn is_renderable(&self) -> bool {
        matches!(self, Self::Resolved(_) | Self::Fallback { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_identity_keeps_provider_and_name_separate() {
        let icon = UiIconId::new("lucide", "save");

        assert_eq!(icon.provider.0, "lucide");
        assert_eq!(icon.name, "save");
    }

    #[test]
    fn icon_failure_is_explicit_and_inspectable() {
        let requested = UiIconId::new("lucide", "missing");
        let fallback = UiIconId::new("project", "missing-placeholder");
        let resolution = UiIconResolution::Fallback {
            requested: requested.clone(),
            fallback,
        };

        assert_eq!(resolution.requested(), &requested);
        assert!(resolution.is_renderable());
        let missing = UiIconResolution::Missing {
            icon: requested.clone(),
            diagnostic: UiIconDiagnostic {
                icon: requested,
                kind: UiIconDiagnosticKind::MissingIcon,
            },
        };
        assert!(!missing.is_renderable());
        assert!(matches!(
            missing,
            UiIconResolution::Missing {
                diagnostic: UiIconDiagnostic {
                    kind: UiIconDiagnosticKind::MissingIcon,
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn icon_spec_keeps_size_and_tint_semantic() {
        let spec = UiIconSpec::new(
            UiIconId::new("lucide", "save"),
            UiRect::new([0.0, 0.0], [0.12, 0.12]),
        )
        .with_tint(UiIconTint::Accent);

        assert_eq!(spec.metrics().size, [0.12, 0.12]);
        assert_eq!(spec.metrics().baseline, 0.0);
        assert_eq!(spec.tint, UiIconTint::Accent);
    }

    #[test]
    fn project_icon_provider_uses_the_same_resolution_contract() {
        let project_icon = UiIconId::new("project", "application-mark");
        let resolution = UiIconResolution::Resolved(project_icon.clone());

        assert_eq!(resolution.requested(), &project_icon);
        assert!(resolution.is_renderable());
    }

    #[test]
    fn icon_handles_distinguish_reused_provider_slots() {
        assert_ne!(UiIconHandle::new(2, 1), UiIconHandle::new(2, 2));
        assert_eq!(UiIconHandle::new(2, 1), UiIconHandle::new(2, 1));
    }

    #[test]
    fn icon_diagnostics_have_explicit_failure_constructors() {
        let icon = UiIconId::new("lucide", "save");
        assert_eq!(
            UiIconDiagnostic::missing(icon.clone()).kind,
            UiIconDiagnosticKind::MissingIcon
        );
        assert_eq!(
            UiIconDiagnostic::provider_unavailable(icon).kind,
            UiIconDiagnosticKind::ProviderUnavailable
        );
    }

    #[test]
    fn vector_provider_returns_geometry_without_renderer_objects() {
        struct FixtureProvider(UiIconProviderId);

        impl UiIconVectorProvider for FixtureProvider {
            fn provider(&self) -> &UiIconProviderId {
                &self.0
            }

            fn resolve_vector(&self, id: &UiIconId) -> Result<UiIconVectorAsset, UiIconDiagnostic> {
                if id.name == "square" {
                    Ok(UiIconVectorAsset {
                        paths: vec![vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]],
                        view_box: [0.0, 0.0, 1.0, 1.0],
                    })
                } else {
                    Err(UiIconDiagnostic::missing(id.clone()))
                }
            }
        }

        let provider = FixtureProvider(UiIconProviderId::new("project"));
        let id = UiIconId::new("project", "square");
        let asset = provider.resolve_vector(&id).unwrap();

        assert_eq!(provider.provider().0, "project");
        assert_eq!(asset.paths[0].len(), 4);
        assert_eq!(asset.view_box, [0.0, 0.0, 1.0, 1.0]);
    }
}
use crate::UiRect;
