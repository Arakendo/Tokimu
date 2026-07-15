use crate::PlatformResult;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OpenXrRuntimeTarget {
    #[default]
    Any,
    SteamVr,
}

impl OpenXrRuntimeTarget {
    pub fn label(self) -> &'static str {
        match self {
            OpenXrRuntimeTarget::Any => "any",
            OpenXrRuntimeTarget::SteamVr => "SteamVR",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OpenXrSessionBackend {
    #[default]
    Unavailable,
    QuestProViaSteamVr,
}

impl OpenXrSessionBackend {
    pub fn from_config(config: OpenXrSessionConfig) -> Self {
        match config.preferred_runtime {
            OpenXrRuntimeTarget::SteamVr => OpenXrSessionBackend::QuestProViaSteamVr,
            OpenXrRuntimeTarget::Any => OpenXrSessionBackend::Unavailable,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OpenXrSessionConfig {
    pub app_name: &'static str,
    pub preferred_runtime: OpenXrRuntimeTarget,
    pub recommended_width: u32,
    pub recommended_height: u32,
}

impl OpenXrSessionConfig {
    pub fn quest_pro_via_steamvr(app_name: &'static str) -> Self {
        Self {
            app_name,
            preferred_runtime: OpenXrRuntimeTarget::SteamVr,
            recommended_width: 1832,
            recommended_height: 1920,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OpenXrEyeView {
    pub width: u32,
    pub height: u32,
    pub eye_offset_meters: f32,
}

impl OpenXrEyeView {
    pub fn new(width: u32, height: u32, eye_offset_meters: f32) -> Self {
        Self {
            width,
            height,
            eye_offset_meters,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OpenXrFrameData {
    pub left_eye: OpenXrEyeView,
    pub right_eye: OpenXrEyeView,
}

impl OpenXrFrameData {
    pub fn stereo(width: u32, height: u32, eye_offset_meters: f32) -> Self {
        Self {
            left_eye: OpenXrEyeView::new(width, height, -eye_offset_meters),
            right_eye: OpenXrEyeView::new(width, height, eye_offset_meters),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OpenXrHeadsetProofPlan {
    pub session_config: OpenXrSessionConfig,
    pub frame_data: OpenXrFrameData,
}

impl OpenXrHeadsetProofPlan {
    pub fn quest_pro_via_steamvr(app_name: &'static str) -> Self {
        Self {
            session_config: OpenXrSessionConfig::quest_pro_via_steamvr(app_name),
            frame_data: OpenXrFrameData::stereo(1832, 1920, 0.032),
        }
    }

    pub fn bridge_contract(self) -> OpenXrRenderBridgeContract {
        OpenXrRenderBridgeContract {
            left_eye: self.frame_data.left_eye,
            right_eye: self.frame_data.right_eye,
        }
    }

    pub fn readiness(self) -> OpenXrSessionReadiness {
        OpenXrSessionReadiness {
            proof_plan: self,
            bridge_contract: self.bridge_contract(),
        }
    }

    pub fn is_supported_by(self, capabilities: OpenXrSessionCapabilities) -> bool {
        self.session_config.preferred_runtime == capabilities.runtime_target
            && self.frame_data.left_eye.width <= capabilities.left_eye.width
            && self.frame_data.left_eye.height <= capabilities.left_eye.height
            && self.frame_data.right_eye.width <= capabilities.right_eye.width
            && self.frame_data.right_eye.height <= capabilities.right_eye.height
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OpenXrSessionReadiness {
    pub proof_plan: OpenXrHeadsetProofPlan,
    pub bridge_contract: OpenXrRenderBridgeContract,
}

impl OpenXrSessionReadiness {
    pub fn is_ready(self, capabilities: OpenXrSessionCapabilities) -> bool {
        self.proof_plan.is_supported_by(capabilities) && self.bridge_contract.matches(capabilities)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OpenXrSessionCapabilities {
    pub runtime_target: OpenXrRuntimeTarget,
    pub left_eye: OpenXrEyeView,
    pub right_eye: OpenXrEyeView,
}

impl OpenXrSessionCapabilities {
    pub fn unavailable() -> Self {
        Self {
            runtime_target: OpenXrRuntimeTarget::Any,
            left_eye: OpenXrEyeView::default(),
            right_eye: OpenXrEyeView::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OpenXrRenderBridgeContract {
    pub left_eye: OpenXrEyeView,
    pub right_eye: OpenXrEyeView,
}

impl OpenXrRenderBridgeContract {
    pub fn from_proof_plan(plan: OpenXrHeadsetProofPlan) -> Self {
        plan.bridge_contract()
    }

    pub fn matches(self, capabilities: OpenXrSessionCapabilities) -> bool {
        self.left_eye.width <= capabilities.left_eye.width
            && self.left_eye.height <= capabilities.left_eye.height
            && self.right_eye.width <= capabilities.right_eye.width
            && self.right_eye.height <= capabilities.right_eye.height
    }
}

pub trait OpenXrSession {
    fn start(&mut self, config: OpenXrSessionConfig) -> PlatformResult<()>;
    fn capabilities(&self) -> OpenXrSessionCapabilities;
    fn poll_frame(&mut self) -> PlatformResult<Option<OpenXrFrameData>>;
    fn submit_frame(&mut self, frame: OpenXrFrameData) -> PlatformResult<()>;
}

pub fn create_openxr_session(config: OpenXrSessionConfig) -> Box<dyn OpenXrSession> {
    match OpenXrSessionBackend::from_config(config) {
        OpenXrSessionBackend::Unavailable => Box::new(OpenXrUnavailableSession),
        OpenXrSessionBackend::QuestProViaSteamVr => Box::new(OpenXrUnavailableSession),
    }
}

pub fn create_openxr_session_for_readiness(
    readiness: OpenXrSessionReadiness,
) -> Box<dyn OpenXrSession> {
    let backend = OpenXrSessionBackend::from_config(readiness.proof_plan.session_config);

    match backend {
        OpenXrSessionBackend::Unavailable => Box::new(OpenXrUnavailableSession),
        OpenXrSessionBackend::QuestProViaSteamVr => Box::new(OpenXrUnavailableSession),
    }
}

pub fn create_openxr_session_from_readiness(
    readiness: OpenXrSessionReadiness,
) -> Box<dyn OpenXrSession> {
    create_openxr_session_for_readiness(readiness)
}

#[derive(Debug, Default)]
pub struct OpenXrUnavailableSession;

impl OpenXrSession for OpenXrUnavailableSession {
    fn start(&mut self, _config: OpenXrSessionConfig) -> PlatformResult<()> {
        Err(std::io::Error::other("OpenXR runtime integration is not implemented yet").into())
    }

    fn capabilities(&self) -> OpenXrSessionCapabilities {
        OpenXrSessionCapabilities::unavailable()
    }

    fn poll_frame(&mut self) -> PlatformResult<Option<OpenXrFrameData>> {
        Ok(None)
    }

    fn submit_frame(&mut self, _frame: OpenXrFrameData) -> PlatformResult<()> {
        Err(std::io::Error::other("OpenXR runtime integration is not implemented yet").into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_openxr_session_boundary_types() {
        let config = OpenXrSessionConfig {
            app_name: "Tokimu",
            preferred_runtime: OpenXrRuntimeTarget::Any,
            recommended_width: 1280,
            recommended_height: 720,
        };

        assert_eq!(config.app_name, "Tokimu");
        assert_eq!(config.preferred_runtime, OpenXrRuntimeTarget::Any);
        assert_eq!(config.recommended_width, 1280);
        assert_eq!(config.recommended_height, 720);
    }

    #[test]
    fn exposes_runtime_labels() {
        assert_eq!(OpenXrRuntimeTarget::Any.label(), "any");
        assert_eq!(OpenXrRuntimeTarget::SteamVr.label(), "SteamVR");
    }

    #[test]
    fn maps_quest_pro_steamvr_configs_to_the_expected_backend() {
        let config = OpenXrSessionConfig::quest_pro_via_steamvr("Tokimu");

        assert_eq!(
            OpenXrSessionBackend::from_config(config),
            OpenXrSessionBackend::QuestProViaSteamVr
        );
    }

    #[test]
    fn constructs_quest_pro_steamvr_profile() {
        let config = OpenXrSessionConfig::quest_pro_via_steamvr("Tokimu");

        assert_eq!(config.app_name, "Tokimu");
        assert_eq!(config.preferred_runtime, OpenXrRuntimeTarget::SteamVr);
        assert_eq!(config.recommended_width, 1832);
        assert_eq!(config.recommended_height, 1920);
    }

    #[test]
    fn create_session_uses_config_driven_factory() {
        let config = OpenXrSessionConfig::quest_pro_via_steamvr("Tokimu");
        let mut session = create_openxr_session(config);

        let error = session.start(config).unwrap_err();
        assert!(error.to_string().contains("not implemented yet"));
    }

    #[test]
    fn create_session_uses_readiness_driven_factory() {
        let readiness = OpenXrHeadsetProofPlan::quest_pro_via_steamvr("Tokimu").readiness();
        let mut session = create_openxr_session_for_readiness(readiness);

        let error = session
            .start(readiness.proof_plan.session_config)
            .unwrap_err();
        assert!(error.to_string().contains("not implemented yet"));
    }

    #[test]
    fn constructs_first_headset_proof_plan() {
        let plan = OpenXrHeadsetProofPlan::quest_pro_via_steamvr("Tokimu");

        assert_eq!(
            plan.session_config.preferred_runtime,
            OpenXrRuntimeTarget::SteamVr
        );
        assert_eq!(plan.frame_data.left_eye.width, 1832);
        assert_eq!(plan.frame_data.right_eye.height, 1920);
        assert!(plan.frame_data.left_eye.eye_offset_meters < 0.0);
        assert!(plan.frame_data.right_eye.eye_offset_meters > 0.0);
    }

    #[test]
    fn checks_proof_plan_against_capabilities() {
        let plan = OpenXrHeadsetProofPlan::quest_pro_via_steamvr("Tokimu");
        let supported = OpenXrSessionCapabilities {
            runtime_target: OpenXrRuntimeTarget::SteamVr,
            left_eye: OpenXrEyeView::new(1832, 1920, 0.0),
            right_eye: OpenXrEyeView::new(1832, 1920, 0.0),
        };
        let unsupported = OpenXrSessionCapabilities::unavailable();

        assert!(plan.is_supported_by(supported));
        assert!(!plan.is_supported_by(unsupported));
    }

    #[test]
    fn converts_proof_plan_into_a_bridge_contract() {
        let plan = OpenXrHeadsetProofPlan::quest_pro_via_steamvr("Tokimu");
        let contract = OpenXrRenderBridgeContract::from_proof_plan(plan);
        let supported = OpenXrSessionCapabilities {
            runtime_target: OpenXrRuntimeTarget::SteamVr,
            left_eye: OpenXrEyeView::new(1832, 1920, 0.0),
            right_eye: OpenXrEyeView::new(1832, 1920, 0.0),
        };
        let unsupported = OpenXrSessionCapabilities::unavailable();

        assert_eq!(contract.left_eye.width, 1832);
        assert_eq!(contract.right_eye.height, 1920);
        assert!(contract.matches(supported));
        assert!(!contract.matches(unsupported));
    }

    #[test]
    fn bundles_proof_plan_and_bridge_contract_into_readiness() {
        let plan = OpenXrHeadsetProofPlan::quest_pro_via_steamvr("Tokimu");
        let readiness = plan.readiness();
        let supported = OpenXrSessionCapabilities {
            runtime_target: OpenXrRuntimeTarget::SteamVr,
            left_eye: OpenXrEyeView::new(1832, 1920, 0.0),
            right_eye: OpenXrEyeView::new(1832, 1920, 0.0),
        };
        let unsupported = OpenXrSessionCapabilities::unavailable();

        assert_eq!(
            readiness.proof_plan.session_config.preferred_runtime,
            OpenXrRuntimeTarget::SteamVr
        );
        assert_eq!(readiness.bridge_contract.left_eye.width, 1832);
        assert!(readiness.is_ready(supported));
        assert!(!readiness.is_ready(unsupported));
    }
}
