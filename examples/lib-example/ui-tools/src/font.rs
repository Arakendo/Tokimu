use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiFontFormat {
    Ttf,
    Otf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiFontProviderId(pub String);

impl UiFontProviderId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiFontIdentity {
    pub provider: UiFontProviderId,
    pub format: UiFontFormat,
    pub source_name: String,
}

/// Opaque identity for a resolved font resource.
///
/// The generation prevents a released provider slot from silently becoming a
/// different font. Registry ownership and invalidation remain outside this
/// example-side contract.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UiFontHandle {
    pub index: u32,
    pub generation: u32,
}

impl UiFontHandle {
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }
}

impl UiFontIdentity {
    pub fn new(
        provider: impl Into<String>,
        format: UiFontFormat,
        source_name: impl Into<String>,
    ) -> Self {
        Self {
            provider: UiFontProviderId::new(provider),
            format,
            source_name: source_name.into(),
        }
    }
}

impl UiFontFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Ttf => "ttf",
            Self::Otf => "otf",
        }
    }
}

#[derive(Clone, Debug)]
pub struct UiFontSource {
    pub provider: String,
    pub format: UiFontFormat,
    pub path: PathBuf,
    pub bytes: Vec<u8>,
}

impl UiFontSource {
    pub fn identity(&self) -> UiFontIdentity {
        UiFontIdentity::new(
            self.provider.clone(),
            self.format,
            self.path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("font")
                .to_owned(),
        )
    }
}

impl UiFontSource {
    pub fn from_prepared_corpus(provider: &str, format: UiFontFormat) -> Result<Self, String> {
        let relative = PathBuf::from("target/glyph-corpus/fonts").join(provider);
        let repository_relative = PathBuf::from("third-party/fonts").join(provider);
        let mut roots = vec![relative.clone()];
        if let Ok(current) = std::env::current_dir() {
            roots.extend(current.ancestors().map(|path| path.join(&relative)));
            roots.extend(
                current
                    .ancestors()
                    .map(|path| path.join(&repository_relative)),
            );
        }
        if let Ok(executable) = std::env::current_exe() {
            roots.extend(executable.ancestors().map(|path| path.join(&relative)));
            roots.extend(
                executable
                    .ancestors()
                    .map(|path| path.join(&repository_relative)),
            );
        }

        let mut searched = Vec::new();
        for root in roots {
            searched.push(root.display().to_string());
            if let Some(path) = find_font(&root, provider, format) {
                let bytes = fs::read(&path)
                    .map_err(|error| format!("could not read {}: {error}", path.display()))?;
                return Ok(Self {
                    provider: provider.to_owned(),
                    format,
                    path,
                    bytes,
                });
            }
        }

        Err(format!(
            "no {} font found for provider `{provider}`; searched: {}. Run prepare-glyph-corpus.ps1 first",
            format.extension(),
            searched.join(", ")
        ))
    }
}

fn find_font(root: &Path, provider: &str, format: UiFontFormat) -> Option<PathBuf> {
    let mut pending = vec![root.to_owned()];
    let mut matches = Vec::new();
    while let Some(path) = pending.pop() {
        let Ok(entries) = fs::read_dir(path) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case(format.extension()))
            {
                matches.push(path);
            }
        }
    }
    matches.sort();
    let preferred = match provider {
        "inter" => ["InterVariable.ttf", "Inter-Regular.ttf"].as_slice(),
        "jetbrains-mono" => ["JetBrainsMono-Regular.otf", "JetBrainsMono-Regular.ttf"].as_slice(),
        "noto" => ["NotoSans-VF.ttf", "NotoSans-Regular.ttf"].as_slice(),
        _ => &[],
    };
    preferred
        .iter()
        .find_map(|name| {
            matches
                .iter()
                .find(|path| path.file_name().and_then(|value| value.to_str()) == Some(*name))
                .cloned()
        })
        .or_else(|| matches.into_iter().next())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_identity_is_logical_and_provider_neutral() {
        let identity = UiFontIdentity::new("inter", UiFontFormat::Ttf, "Inter-Regular.ttf");

        assert_eq!(identity.provider.0, "inter");
        assert_eq!(identity.format, UiFontFormat::Ttf);
        assert_eq!(identity.source_name, "Inter-Regular.ttf");
    }

    #[test]
    fn font_handles_distinguish_reused_provider_slots() {
        assert_ne!(UiFontHandle::new(4, 1), UiFontHandle::new(4, 2));
        assert_eq!(UiFontHandle::new(4, 1), UiFontHandle::new(4, 1));
    }
}
