use crate::constants;

#[derive(Debug)]
pub struct SupportedDefinition {
    pub content_types: Vec<String>,
    pub name_contains: Vec<String>,
    pub major_version_required: i32,
}


pub fn is_supported_platform() -> bool {
    constants::SUPPORTED_PLATFORMS.contains(&std::env::consts::OS.to_string())
}

pub fn supported_definition() -> SupportedDefinition {
    match std::env::consts::OS {
        "macos" => macos_supported_definition(),
        "linux" => linux_supported_definition(),
        &_ => macos_supported_definition(),
    }
}

fn macos_supported_definition() -> SupportedDefinition {
    SupportedDefinition {
        name_contains: vec![
            "-macos-".to_string(),
            "-macos10.10-".to_string(),
            "-Darwin-".to_string(),
        ],
        content_types: vec!["application/gzip".to_string()],
        major_version_required: 3,
    }
}

fn linux_supported_definition() -> SupportedDefinition {
    SupportedDefinition {
        name_contains: vec!["-linux-x86_64".to_string(), "-Linux-x86_64".to_string()],
        content_types: vec!["application/gzip".to_string()],
        major_version_required: 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported_platform_on_known_os() {
        let result = is_supported_platform();
        let os = std::env::consts::OS;
        let expected = os == "linux" || os == "macos";
        assert_eq!(result, expected);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_supported_definition_linux_name_contains() {
        let def = supported_definition();
        assert!(def.name_contains.contains(&"-linux-x86_64".to_string()));
        assert!(def.name_contains.contains(&"-Linux-x86_64".to_string()));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_supported_definition_linux_content_types() {
        let def = supported_definition();
        assert!(def.content_types.contains(&"application/gzip".to_string()));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_supported_definition_linux_major_version_required() {
        let def = supported_definition();
        assert_eq!(def.major_version_required, 3);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_supported_definition_macos_name_contains() {
        let def = supported_definition();
        assert!(def.name_contains.contains(&"-macos-".to_string()));
        assert!(def.name_contains.contains(&"-Darwin-".to_string()));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_supported_definition_macos_content_types() {
        let def = supported_definition();
        assert!(def.content_types.contains(&"application/gzip".to_string()));
    }
}
