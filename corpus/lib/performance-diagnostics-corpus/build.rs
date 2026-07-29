fn main() {
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "unknown".into());
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".into());

    println!("cargo:rustc-env=TOKIMU_CORPUS_PROFILE={profile}");
    println!("cargo:rustc-env=TOKIMU_CORPUS_TARGET={target}");
}
