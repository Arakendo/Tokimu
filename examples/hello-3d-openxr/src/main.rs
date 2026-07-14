use tokimu::openxr::{
    create_openxr_session_from_readiness, OpenXrHeadsetProofPlan,
};

fn main() {
    let readiness = OpenXrHeadsetProofPlan::quest_pro_via_steamvr("Tokimu Hello 3D OpenXR")
        .readiness();
    let session = create_openxr_session_from_readiness(readiness);
    let capabilities = session.capabilities();

    println!("Tokimu Hello 3D OpenXR");
    println!("preferred runtime: {}", readiness.proof_plan.session_config.preferred_runtime.label());
    println!(
        "bridge contract ready: {}",
        readiness.bridge_contract.matches(capabilities)
    );
    println!("session ready: {}", readiness.is_ready(capabilities));
}