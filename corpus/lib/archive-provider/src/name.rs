use crate::ArchiveError;

pub(crate) fn normalize_entry_name(
    name: &str,
    max_path_bytes: u32,
) -> Result<String, ArchiveError> {
    if name.len() > max_path_bytes as usize {
        return Err(ArchiveError::UnsafeEntryName {
            name: name.to_owned(),
            reason: format!("name exceeds {max_path_bytes} bytes"),
        });
    }
    if name.is_empty() {
        return unsafe_name(name, "name is empty");
    }
    if name.contains('\0') {
        return unsafe_name(name, "name contains NUL");
    }

    let portable = name.replace('\\', "/");
    if portable.starts_with('/') {
        return unsafe_name(name, "absolute paths are prohibited");
    }
    if portable
        .as_bytes()
        .get(1)
        .is_some_and(|separator| *separator == b':')
    {
        return unsafe_name(name, "drive prefixes are prohibited");
    }

    let is_directory = portable.ends_with('/');
    let mut parts = Vec::new();
    for part in portable.split('/') {
        match part {
            "" if is_directory => {}
            "" => return unsafe_name(name, "empty path component"),
            "." => {}
            ".." => return unsafe_name(name, "parent traversal is prohibited"),
            component if component.ends_with(':') => {
                return unsafe_name(name, "drive-like path component is prohibited")
            }
            component => parts.push(component),
        }
    }

    if parts.is_empty() {
        return unsafe_name(name, "name has no material component");
    }

    let mut normalized = parts.join("/");
    if is_directory {
        normalized.push('/');
    }
    Ok(normalized)
}

fn unsafe_name<T>(name: &str, reason: &str) -> Result<T, ArchiveError> {
    Err(ArchiveError::UnsafeEntryName {
        name: name.to_owned(),
        reason: reason.to_owned(),
    })
}
